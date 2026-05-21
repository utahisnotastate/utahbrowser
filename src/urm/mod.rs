//! Utah Unified Reality Manifold (URM) — reads Nexus Orchestrator state from vault.

use crate::browser::storage_bridge;
use anyhow::{Context, Result};
use serde::Deserialize;
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize)]
pub struct NexusState {
    pub status: String,
    pub coherence: f32,
    #[serde(default)]
    pub hardware_id: String,
    #[serde(default)]
    pub shard: Option<serde_json::Value>,
    #[serde(default)]
    pub sensory_active: bool,
    #[serde(default)]
    pub snapshots: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct BrowserOverlay {
    pub message: String,
    pub severity: String,
    pub visible: bool,
}

#[derive(Debug, Clone, Deserialize)]
pub struct MutagenesisProposal {
    pub summary: String,
    #[serde(default)]
    pub target_file: Option<String>,
    #[serde(default)]
    pub patch_hint: Option<String>,
}

/// Bridge to the Python Nexus Orchestrator JSON artifacts.
#[derive(Debug, Default)]
pub struct UrmBridge;

impl UrmBridge {
    pub fn nexus_state_path() -> PathBuf {
        storage_bridge::urm_nexus_state()
    }

    pub fn overlay_path() -> PathBuf {
        storage_bridge::urm_browser_overlay()
    }

    pub fn is_active(&self) -> bool {
        Self::nexus_state_path().is_file()
    }

    pub fn read_state(&self) -> Result<Option<NexusState>> {
        let path = Self::nexus_state_path();
        if !path.is_file() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&path).with_context(|| format!("read {}", path.display()))?;
        Ok(Some(serde_json::from_str(&raw)?))
    }

    pub fn read_overlay(&self) -> Result<Option<BrowserOverlay>> {
        let path = Self::overlay_path();
        if !path.is_file() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&raw)?))
    }

    pub fn read_mutagenesis_latest(&self) -> Result<Option<MutagenesisProposal>> {
        let path = storage_bridge::urm_mutagenesis_latest();
        if !path.is_file() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&path)?;
        Ok(Some(serde_json::from_str(&raw)?))
    }

    pub fn status_line(&self) -> String {
        match self.read_state() {
            Ok(Some(s)) => format!(
                "URM Nexus {} — coherence {:.0}%",
                s.status,
                s.coherence * 100.0
            ),
            Ok(None) => "URM offline — run scripts/install_urm.ps1".into(),
            Err(e) => format!("URM read error: {e:#}"),
        }
    }
}
