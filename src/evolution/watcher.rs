//! Watches configured code paths and writes LLM optimization proposals (never auto-applies).

use crate::config::{AppConfig, EvolutionConfig};
use crate::ipc::IpcEvent;
use crate::truth::ollama::OllamaClient;
use anyhow::Result;
use notify::{Config as NotifyConfig, EventKind, RecommendedWatcher, RecursiveMode, Watcher};
use std::path::Path;
use std::sync::mpsc;
use std::sync::Arc;
use std::time::{Duration, Instant};
use tracing::{info, warn};

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
    std::thread::spawn(move || {
        if let Err(e) = run_watcher(&config, runtime, on_event) {
            tracing::error!("evolution daemon stopped: {e:#}");
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

    let (tx, rx) = mpsc::channel();
    let mut watcher = RecommendedWatcher::new(
        move |res| {
            if let Ok(event) = res {
                let _ = tx.send(event);
            }
        },
        NotifyConfig::default(),
    )?;

    for path in &config.evolution.watch_paths {
        if path.exists() {
            watcher.watch(path, RecursiveMode::Recursive)?;
            info!("evolution watching {}", path.display());
        } else {
            warn!("evolution watch path missing: {}", path.display());
        }
    }

    let ollama = OllamaClient::new(config.ollama.clone());
    let debounce = Duration::from_millis(config.evolution.debounce_ms);
    let mut pending: Option<(String, Instant)> = None;

    loop {
        if let Ok(event) = rx.recv_timeout(Duration::from_millis(500)) {
            if matches!(
                event.kind,
                EventKind::Modify(_) | EventKind::Create(_) | EventKind::Remove(_)
            ) {
                if let Some(path) = event.paths.first() {
                    if is_code_file(path) {
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
                if let Ok(proposal) = runtime.block_on(generate_proposal(
                    &ollama,
                    &path,
                    &config.evolution,
                    &proposals_dir,
                )) {
                    on_event(IpcEvent::EvolutionProposal {
                        path: path.clone(),
                        summary: proposal.summary.clone(),
                    });
                    info!("evolution proposal written for {}", path);
                }
            }
        }
    }
}

fn is_code_file(path: &Path) -> bool {
    matches!(
        path.extension().and_then(|e| e.to_str()),
        Some("rs" | "toml" | "css" | "html" | "md")
    )
}

struct Proposal {
    summary: String,
}

async fn generate_proposal(
    ollama: &OllamaClient,
    path: &str,
    _evolution: &EvolutionConfig,
    proposals_dir: &Path,
) -> Result<Proposal> {
    let content = std::fs::read_to_string(path).unwrap_or_default();
    let snippet: String = content.chars().take(4000).collect();
    let system = "You are the Utah Browser evolution daemon. Suggest small, safe performance \
        or clarity improvements. Do not invent files. Output markdown with a 1-line summary first.";
    let user = format!("File: {path}\n\n```\n{snippet}\n```");
    let summary = ollama
        .complete(system, &user)
        .await
        .unwrap_or_else(|e| format!("Proposal skipped (Ollama unavailable): {e}"));

    let stamp = chrono_lite_timestamp();
    let out = proposals_dir.join(format!("{stamp}_{}", sanitize_filename(path)));
    std::fs::write(&out, &summary)?;

    let first_line = summary.lines().next().unwrap_or("Optimization proposal").to_string();
    Ok(Proposal { summary: first_line })
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

// ollama.complete is async - need to fix watcher to use tokio or block_on
