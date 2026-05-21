//! Sentinel-Core boot — shell ready signal + delayed background daemons (no install-dir contention).

use crate::config::AppConfig;
use crate::diagnostics::RecoveryState;
use crate::evolution;
use crate::ipc::IpcEvent;
use crate::paths;
use std::fs::OpenOptions;
use std::io::Write;
use std::sync::Arc;
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use tracing::info;

/// Grace period after shell ready before evolution/audio may touch disk.
pub const DAEMON_GRACE_SECS: u64 = 5;

/// Remove stale ready flag from a prior crash.
pub fn clear_ready_signal() {
    let path = paths::sovereign_ready_signal_path();
    let _ = std::fs::remove_file(&path);
}

/// Written when WebView shell initialized successfully (Sentinel traffic controller).
pub fn signal_shell_ready(boot_mode: &str) {
    let _ = paths::ensure_sovereign_dirs();
    let path = paths::sovereign_ready_signal_path();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let body = format!("ready\nboot_mode={boot_mode}\nunix={ts}\n");
    if std::fs::write(&path, &body).is_ok() {
        crate::diagnostics::log_step(&format!(
            "sentinel: shell ready signal ({boot_mode}) → {}",
            path.display()
        ));
    }
}

pub fn log_evolution(msg: &str) {
    let _ = paths::ensure_sovereign_dirs();
    let path = paths::evolution_log_path();
    let ts = SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let line = format!("[{ts}] {msg}\n");
    if let Ok(mut f) = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
    {
        let _ = f.write_all(line.as_bytes());
        let _ = f.flush();
    }
    tracing::debug!("evolution: {msg}");
}

/// Evolution and audio start only after shell.ready exists + grace window (non-blocking).
pub fn spawn_delayed_background_services(
    config: AppConfig,
    runtime: Arc<tokio::runtime::Runtime>,
    recovery: RecoveryState,
) {
    if recovery.should_use_safe_mode() {
        crate::diagnostics::log_step("sentinel: safe mode — background daemons deferred/disabled");
        return;
    }

    std::thread::spawn(move || {
        let ready_path = paths::sovereign_ready_signal_path();
        let deadline = Instant::now() + Duration::from_secs(120);
        while Instant::now() < deadline {
            if ready_path.is_file() {
                break;
            }
            std::thread::sleep(Duration::from_millis(200));
        }
        if !ready_path.is_file() {
            log_evolution("daemon start skipped — shell.ready never appeared");
            return;
        }

        crate::diagnostics::log_step(&format!(
            "sentinel: waiting {DAEMON_GRACE_SECS}s before background daemons"
        ));
        std::thread::sleep(Duration::from_secs(DAEMON_GRACE_SECS));

        if config.audio.enabled {
            crate::diagnostics::log_step("sentinel: starting audio listener");
            crate::audio::spawn_audio_listener(config.audio.clone());
        }

        if config.evolution.enabled {
            log_evolution("starting evolution watcher (sovereign data path)");
            crate::diagnostics::log_step("sentinel: starting evolution daemon");
            let proxy = {
                Box::new(move |ev: IpcEvent| {
                    if let IpcEvent::EvolutionProposal { path, summary } = ev {
                        log_evolution(&format!("proposal for {path}: {summary}"));
                        info!("evolution proposal for {path}: {summary}");
                    }
                }) as evolution::EventCallback
            };
            evolution::spawn_evolution_daemon(config, runtime, proxy);
        } else {
            log_evolution("evolution disabled in config");
        }
    });
}
