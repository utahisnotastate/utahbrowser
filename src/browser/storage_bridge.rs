//! Local vault paths — all persistence routes through the sovereign data directory.

use anyhow::{Context, Result};
use std::fs;
use std::path::{Path, PathBuf};

/// Root of the Utah Browser local vault (`~/.utah_browser` or platform equivalent).
pub fn vault_root() -> PathBuf {
    crate::paths::sovereign_data_root()
}

pub fn vault_dir(name: &str) -> PathBuf {
    vault_root().join(name)
}

pub fn tab_cache_dir() -> PathBuf {
    vault_dir("cache").join("tabs")
}

pub fn extensions_dir() -> PathBuf {
    vault_dir("extensions")
}

pub fn logs_dir() -> PathBuf {
    vault_dir("logs")
}

pub fn ghost_link_dir() -> PathBuf {
    vault_dir("ghost-link")
}

pub fn ghost_link_logs() -> PathBuf {
    ghost_link_dir().join("logs")
}

pub fn ghost_link_events() -> PathBuf {
    ghost_link_dir().join("out").join("events.jsonl")
}

pub fn ghost_link_prefetch() -> PathBuf {
    ghost_link_dir().join("out").join("prefetch.json")
}

pub fn ghost_link_theme() -> PathBuf {
    ghost_link_dir().join("out").join("theme.json")
}

pub fn ghost_link_telemetry() -> PathBuf {
    ghost_link_logs().join("telemetry.log")
}

pub fn zones_manifest_path() -> PathBuf {
    vault_dir("vault").join("zones.json")
}

pub fn ingestion_watch_path() -> PathBuf {
    vault_dir("vault").join("ingestion_watch.json")
}

pub fn bookmarks_index_path() -> PathBuf {
    vault_dir("vault").join("bookmarks.json")
}

pub fn urm_root() -> PathBuf {
    if cfg!(windows) && std::env::var("URM_USE_PROGRAMDATA").is_ok() {
        std::env::var("PROGRAMDATA")
            .map(|p| PathBuf::from(p).join("Utah_URM"))
            .unwrap_or_else(|_| vault_dir("urm"))
    } else {
        vault_dir("urm")
    }
}

pub fn urm_nexus_state() -> PathBuf {
    urm_root().join("nexus").join("state.json")
}

pub fn urm_browser_overlay() -> PathBuf {
    urm_root().join("nexus").join("overlay.json")
}

pub fn urm_snapshots_dir() -> PathBuf {
    urm_root().join("snapshots")
}

pub fn urm_mutagenesis_latest() -> PathBuf {
    urm_root().join("mutagenesis").join("latest.json")
}

pub fn tab_cache_path(tab_id: u32) -> PathBuf {
    tab_cache_dir().join(format!("tab_{tab_id}.bin"))
}

/// Create vault layout used by the Zero-Click Kernel and browser core.
pub fn ensure_vault() -> Result<()> {
    for dir in [
        vault_root(),
        vault_dir("vault"),
        tab_cache_dir(),
        extensions_dir(),
        logs_dir(),
        ghost_link_dir(),
        ghost_link_logs(),
        ghost_link_dir().join("out"),
        urm_root(),
        urm_root().join("nexus"),
        urm_root().join("snapshots"),
        urm_root().join("mutagenesis"),
        urm_root().join("swarm"),
        urm_root().join("logs"),
    ] {
        fs::create_dir_all(&dir)
            .with_context(|| format!("create vault dir {}", dir.display()))?;
    }
    Ok(())
}

/// Legacy path used before Phase 4 (migrated on load if present).
pub fn legacy_data_dir() -> PathBuf {
    dirs::data_local_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join("utah-browser")
}

pub fn migrate_legacy_bookmarks_if_needed(target: &Path) -> Result<()> {
    let legacy = legacy_data_dir().join("bookmarks.json");
    if legacy.is_file() && !target.is_file() {
        if let Some(parent) = target.parent() {
            fs::create_dir_all(parent)?;
        }
        fs::copy(&legacy, target)?;
        tracing::info!(
            "migrated bookmarks from {} to {}",
            legacy.display(),
            target.display()
        );
    }
    Ok(())
}
