//! Native folder picker + ingestion daemon spawn (non-blocking).

use super::zones::SemanticBindingStore;
use anyhow::{Context, Result};
use std::path::Path;
use std::process::Command;

/// Invoke OS folder picker and bind a new cognitive zone.
pub fn pick_and_bind(store: &mut SemanticBindingStore) -> Result<Option<super::zones::KnowledgeZone>> {
    let folder = rfd::FileDialog::new()
        .set_title("Map Cognitive Zone")
        .pick_folder();
    let Some(path) = folder else {
        return Ok(None);
    };
    let zone = store.bind_zone(path, None)?;
    spawn_ingestion_daemon(&zone.path)?;
    Ok(Some(zone))
}

pub fn spawn_ingestion_daemon(path: &Path) -> Result<()> {
    let repo = crate::paths::install_root();
    let script = repo.join("scripts/ingestion_daemon.py");
    if !script.is_file() {
        tracing::warn!("ingestion_daemon.py missing — Rust ingest will run via IPC only");
        return Ok(());
    }
    let path_str = path.to_string_lossy().to_string();
    Command::new(python_executable())
        .arg(&script)
        .arg("--add-path")
        .arg(&path_str)
        .stdout(std::process::Stdio::null())
        .stderr(std::process::Stdio::null())
        .spawn()
        .with_context(|| format!("spawn ingestion daemon for {}", path.display()))?;
    tracing::info!("[ZEO-CORE] Ingestion daemon signaled for {}", path.display());
    Ok(())
}

fn python_executable() -> String {
    std::env::var("UTAH_PYTHON").unwrap_or_else(|_| {
        if cfg!(windows) {
            "python".into()
        } else {
            "python3".into()
        }
    })
}
