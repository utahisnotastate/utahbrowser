//! Ghost-Link bridge — reads sensory daemon output from the local vault.

use crate::browser::storage_bridge;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GhostPrefetchHint {
    pub ts: Option<String>,
    pub prefetch: Option<bool>,
    pub summary: Option<String>,
    pub suggested_url: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct SensoryTheme {
    pub mode: String,
    pub accent: String,
    pub contrast: String,
    #[serde(default)]
    pub audio_rms: f32,
}

#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct GhostEvent {
    pub ts: String,
    pub trigger: String,
    pub entropy: f32,
    pub summary: Option<String>,
}

/// Reads `prefetch.json` / `events.jsonl` written by the Python Ghost-Link daemon.
#[derive(Debug)]
pub struct GhostLinkBridge {
    prefetch_path: PathBuf,
    events_path: PathBuf,
    theme_path: PathBuf,
    log_path: PathBuf,
}

impl GhostLinkBridge {
    pub fn new() -> Self {
        Self {
            prefetch_path: storage_bridge::ghost_link_prefetch(),
            events_path: storage_bridge::ghost_link_events(),
            theme_path: storage_bridge::ghost_link_theme(),
            log_path: storage_bridge::ghost_link_telemetry(),
        }
    }

    pub fn read_theme(&self) -> Result<Option<SensoryTheme>> {
        if !self.theme_path.is_file() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&self.theme_path)?;
        Ok(serde_json::from_str(&raw).ok())
    }

    pub fn is_active(&self) -> bool {
        self.prefetch_path.exists()
            || self.events_path.exists()
            || self.log_path.exists()
    }

    pub fn read_prefetch_hint(&self) -> Result<Option<GhostPrefetchHint>> {
        if !self.prefetch_path.is_file() {
            return Ok(None);
        }
        let raw = fs::read_to_string(&self.prefetch_path)
            .with_context(|| format!("read {}", self.prefetch_path.display()))?;
        let hint: GhostPrefetchHint = serde_json::from_str(&raw)?;
        Ok(Some(hint))
    }

    pub fn consume_prefetch_url(&self) -> Result<Option<String>> {
        let Some(hint) = self.read_prefetch_hint()? else {
            return Ok(None);
        };
        if hint.prefetch == Some(false) {
            return Ok(None);
        }
        Ok(hint.suggested_url.filter(|u| !u.is_empty()))
    }

    pub fn recent_events(&self, max: usize) -> Result<Vec<GhostEvent>> {
        if !self.events_path.is_file() {
            return Ok(Vec::new());
        }
        let file = fs::File::open(&self.events_path)?;
        let len = file.metadata()?.len();

        // 32KB buffer for tailing — avoid reading gigabytes of JSONL
        let buf_size = 32768;
        let offset = len.saturating_sub(buf_size as u64);

        use std::io::{Read, Seek, SeekFrom};
        let mut f = file;
        f.seek(SeekFrom::Start(offset))?;
        let mut buf = Vec::new();
        f.read_to_end(&mut buf)?;

        let content = String::from_utf8_lossy(&buf);
        // If we're not at the start, the first line is likely partial.
        let mut lines: Vec<&str> = content.lines().collect();
        if offset > 0 && !lines.is_empty() {
            lines.remove(0);
        }

        let mut events = Vec::new();
        for line in lines.iter().rev().take(max).rev() {
            if let Ok(ev) = serde_json::from_str::<GhostEvent>(line) {
                events.push(ev);
            }
        }
        Ok(events)
    }

    pub fn status_summary(&self) -> String {
        if !self.is_active() {
            return "Ghost-Link offline — run scripts/install_ghost_link.ps1".into();
        }
        let events = self.recent_events(1).ok().and_then(|v| v.into_iter().next());
        if let Some(ev) = events {
            return format!(
                "Ghost-Link active — last trigger `{}` entropy {:.2}",
                ev.trigger, ev.entropy
            );
        }
        "Ghost-Link active — awaiting sensory trigger".into()
    }
}

impl Default for GhostLinkBridge {
    fn default() -> Self {
        Self::new()
    }
}
