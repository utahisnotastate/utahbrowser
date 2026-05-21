//! Watches configured code paths and writes LLM optimization proposals (never auto-applies).

use crate::config::{AppConfig, EvolutionConfig};
use crate::ipc::IpcEvent;
use crate::truth::ollama::OllamaClient;
use anyhow::Result;
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::{Path, PathBuf};
use std::sync::mpsc;
use std::sync::{Arc, Mutex};
use std::time::{Duration, Instant};
use tracing::{debug, info, warn};

/// Proposals queued while browser holds instance lock (lock-free for install dir).
static EVOLUTION_QUEUE: Mutex<Vec<(String, String)>> = Mutex::new(Vec::new());

fn browser_is_active() -> bool {
    crate::paths::instance_lock_path().is_file()
}

fn flush_evolution_queue(proposals_dir: &Path) {
    let mut batch = Vec::new();
    if let Ok(mut q) = EVOLUTION_QUEUE.lock() {
        batch = std::mem::take(&mut *q);
    }
    for (path, summary) in batch {
        let stamp = chrono_lite_timestamp();
        let out = proposals_dir.join(format!("{stamp}_{}", sanitize_filename(&path)));
        if std::fs::write(&out, &summary).is_ok() {
            crate::sentinel::log_evolution(&format!("flushed queued proposal {}", out.display()));
        }
    }
}

/// Callback invoked when a new evolution proposal is written.
pub type EventCallback = Box<dyn Fn(IpcEvent) + Send + Sync>;

/// Start the evolution daemon on a background thread.
pub fn spawn_evolution_daemon(
    config: AppConfig,
    runtime: Arc<tokio::runtime::Runtime>,
    on_event: EventCallback,
) {
    if !config.evolution.enabled {
        info!("evolution daemon disabled");
        return;
    }
    crate::sentinel::log_evolution("evolution daemon thread spawned");
    let proposals_dir = config.proposals_dir();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(Duration::from_secs(10));
            if !browser_is_active() {
                flush_evolution_queue(&proposals_dir);
            }
        }
    });
    std::thread::spawn(move || {
        if let Err(e) = run_watcher(&config, runtime, on_event) {
            let msg = format!("evolution daemon stopped: {e:#}");
            crate::sentinel::log_evolution(&msg);
            tracing::error!("{msg}");
        }
    });
}

fn run_watcher(
    config: &AppConfig,
    runtime: Arc<tokio::runtime::Runtime>,
    on_event: EventCallback,
) -> Result<()> {
    let proposals_dir = config.proposals_dir();
    std::fs::create_dir_all(&proposals_dir)?;
    crate::sentinel::log_evolution(&format!(
        "watching {} path(s); proposals → {}",
        config.evolution.watch_paths.len(),
        proposals_dir.display()
    ));

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        NotifyConfig::default(),
    )?;

    let install = crate::paths::install_root();
    let mut watched = 0u32;
    for path in &config.evolution.watch_paths {
        let resolved = resolve_watch_path(path, &install);
        if !resolved.exists() {
            warn!("evolution watch path missing: {}", resolved.display());
            continue;
        }
        if should_ignore_path(&resolved) {
            warn!("evolution skip unsafe watch path: {}", resolved.display());
            continue;
        }
        watcher.watch(&resolved, RecursiveMode::Recursive)?;
        info!("evolution watching {}", resolved.display());
        watched += 1;
    }

    if watched == 0 {
        warn!("evolution enabled but no safe watch paths — daemon idle");
    }

    let ollama = OllamaClient::new(config.ollama.clone());
    let debounce = Duration::from_millis(config.evolution.debounce_ms.max(1000));
    let mut pending: Option<(String, Instant)> = None;

    loop {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(500)) {
            if matches!(
                event.kind,
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
            ) {
                if let Some(path) = event.paths.first() {
                    if is_interesting_source(path, &proposals_dir) {
                        pending = Some((
                            path.to_string_lossy().into_owned(),
                            Instant::now(),
                        ));
                    }
                }
            }
        }

        if let Some((path, started)) = &pending {
            if started.elapsed() >= debounce {
                let path = path.clone();
                pending = None;
                match runtime.block_on(generate_proposal(
                    &ollama,
                    &path,
                    &config.evolution,
                    &proposals_dir,
                )) {
                    Ok(Some(proposal)) => {
                        on_event(IpcEvent::EvolutionProposal {
                            path: path.clone(),
                            summary: proposal.summary,
                        });
                        info!("evolution proposal for {}", path);
                    }
                    Ok(None) => {
                        debug!("evolution skipped (no proposal): {}", path);
                    }
                    Err(e) => {
                        warn!("evolution proposal failed for {}: {e:#}", path);
                    }
                }
            }
        }
    }
}

fn resolve_watch_path(path: &Path, install: &Path) -> PathBuf {
    if path.is_absolute() {
        path.to_path_buf()
    } else {
        install.join(path)
    }
}

/// Paths that must never be watched (build output, logs, webview cache, proposal spam).
fn should_ignore_path(path: &Path) -> bool {
    let s = path.to_string_lossy().to_ascii_lowercase();
    const SKIP: &[&str] = &[
        "\\dist\\",
        "/dist/",
        "\\target\\",
        "/target/",
        "\\.git\\",
        "/.git/",
        "\\node_modules\\",
        "/node_modules/",
        "\\.webview2",
        "/.webview2",
        "\\evolution\\proposals",
        "/evolution/proposals",
        "\\logs\\",
        "/logs/",
        "browser.log",
        "recovery.json",
    ];
    SKIP.iter().any(|needle| s.contains(needle))
}

fn is_interesting_source(path: &Path, proposals_dir: &Path) -> bool {
    if should_ignore_path(path) {
        return false;
    }
    if path.starts_with(proposals_dir) {
        return false;
    }
    if !is_code_file(path) {
        return false;
    }
    // Only Rust sources and UI assets — not generated logs or markdown proposals.
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs" | "toml" | "css" | "html" | "js")
    )
}

fn is_code_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs" | "toml" | "css" | "html" | "js" | "md")
    )
}

struct Proposal {
    summary: String,
}

/// Returns `None` when Ollama is offline or path should not produce a file.
async fn generate_proposal(
    ollama: &OllamaClient,
    path: &str,
    _evolution: &EvolutionConfig,
    proposals_dir: &Path,
) -> Result<Option<Proposal>> {
    if should_ignore_path(Path::new(path)) {
        return Ok(None);
    }

    let content = std::fs::read_to_string(path).unwrap_or_default();
    if content.is_empty() {
        return Ok(None);
    }

    let snippet: String = content.chars().take(4000).collect();
    let system = "You are the Utah Browser evolution daemon. Suggest small, safe performance \
        or clarity improvements. Do not invent files. Output markdown with a 1-line summary first.";
    let user = format!("File: {path}\n\n```\n{snippet}\n```");

    let summary = match ollama.complete(system, &user).await {
        Ok(s) => s,
        Err(e) => {
            debug!("evolution: Ollama unavailable for {}: {e:#}", path);
            return Ok(None);
        }
    };

    let first_line = summary
        .lines()
        .find(|l| !l.trim().is_empty())
        .unwrap_or("Optimization proposal")
        .to_string();

    if browser_is_active() {
        if let Ok(mut q) = EVOLUTION_QUEUE.lock() {
            q.push((path.to_string(), summary.clone()));
            crate::sentinel::log_evolution(&format!("queued proposal for {path} (browser active)"));
        }
        return Ok(Some(Proposal { summary: first_line }));
    }

    let stamp = chrono_lite_timestamp();
    let out = proposals_dir.join(format!("{stamp}_{}", sanitize_filename(path)));
    std::fs::write(&out, &summary)?;
    crate::sentinel::log_evolution(&format!("wrote proposal {}", out.display()));

    Ok(Some(Proposal { summary: first_line }))
}

fn sanitize_filename(path: &str) -> String {
    path.replace(['/', '\\', ':'], "_")
}

fn chrono_lite_timestamp() -> String {
    use std::time::{SystemTime, UNIX_EPOCH};
    let secs = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs();
    format!("{secs}")
}
