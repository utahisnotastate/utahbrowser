//! Ensures local services are reachable before Truth Engine operations.

use crate::config::AppConfig;
use crate::truth::qdrant::QdrantClient;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};
use std::time::Duration;
use tokio::time::sleep;

/// Resolve `scripts/Ensure-Qdrant.ps1` next to the executable or project root.
fn ensure_qdrant_script() -> Option<PathBuf> {
    let script = crate::paths::scripts_dir().join("Ensure-Qdrant.ps1");
    if script.is_file() {
        return Some(script);
    }
    None
}

fn project_root_from_script(script: &Path) -> Option<PathBuf> {
    script.parent()?.parent().map(|p| p.to_path_buf())
}

#[cfg(windows)]
fn spawn_ensure_qdrant_script() -> Result<()> {
    let script = ensure_qdrant_script()
        .context("Ensure-Qdrant.ps1 not found — run install.ps1 or Launch-UtahBrowser.ps1")?;
    let root = project_root_from_script(&script).unwrap_or_else(crate::paths::install_root);

    tracing::info!("Qdrant offline — running Ensure-Qdrant.ps1");
    let status = std::process::Command::new("powershell")
        .args([
            "-NoProfile",
            "-ExecutionPolicy",
            "Bypass",
            "-File",
        ])
        .arg(&script)
        .arg("-ProjectRoot")
        .arg(&root)
        .status()
        .context("spawn Ensure-Qdrant.ps1")?;

    if !status.success() {
        tracing::warn!("Ensure-Qdrant.ps1 exited with {}", status);
    }
    Ok(())
}

#[cfg(not(windows))]
fn spawn_ensure_qdrant_script() -> Result<()> {
    tracing::warn!("Automatic Qdrant start is only implemented on Windows");
    Ok(())
}

/// Ping Qdrant, auto-start via PowerShell on Windows if needed, then ensure collection exists.
pub async fn ensure_qdrant_ready(config: &AppConfig, qdrant: &QdrantClient) -> Result<()> {
    for attempt in 0..4 {
        if qdrant.ping().await.unwrap_or(false) {
            qdrant.ensure_collection().await?;
            return Ok(());
        }

        if attempt == 0 {
            let _ = spawn_ensure_qdrant_script();
        }

        sleep(Duration::from_secs(3)).await;
    }

    anyhow::bail!(
        "Qdrant is not reachable at {}. Re-run .\\scripts\\install.ps1 or \
         .\\scripts\\Ensure-Qdrant.ps1 (installs native Qdrant without Docker)",
        config.qdrant.url
    )
}
