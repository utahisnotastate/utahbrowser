//! Semantic memory anchors — URL + scroll + session snapshot for exact resume.

use crate::browser::storage_bridge;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MemoryAnchor {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub intention: String,
    pub scroll_x: f32,
    pub scroll_y: f32,
    pub tab_title: String,
    pub created_unix: u64,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct AnchorFile {
    next_id: u32,
    items: Vec<MemoryAnchor>,
}

#[derive(Debug)]
pub struct MemoryAnchorStore {
    path: std::path::PathBuf,
    data: AnchorFile,
}

impl MemoryAnchorStore {
    pub fn empty() -> Self {
        let path = storage_bridge::vault_dir("memory_anchors.json");
        Self {
            path,
            data: AnchorFile::default(),
        }
    }

    pub fn load() -> Result<Self> {
        storage_bridge::ensure_vault()?;
        let path = storage_bridge::vault_dir("memory_anchors.json");
        let data = if path.is_file() {
            let raw = fs::read_to_string(&path)?;
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            AnchorFile::default()
        };
        Ok(Self { path, data })
    }

    pub fn list(&self) -> &[MemoryAnchor] {
        &self.data.items
    }

    pub fn create(
        &mut self,
        title: String,
        url: String,
        intention: String,
        scroll_x: f32,
        scroll_y: f32,
        tab_title: String,
    ) -> MemoryAnchor {
        let id = self.data.next_id;
        self.data.next_id += 1;
        let anchor = MemoryAnchor {
            id,
            title: if title.trim().is_empty() {
                tab_title.clone()
            } else {
                title
            },
            url,
            intention,
            scroll_x,
            scroll_y,
            tab_title,
            created_unix: unix_now(),
        };
        self.data.items.push(anchor.clone());
        let _ = self.save();
        anchor
    }

    fn save(&self) -> Result<()> {
        let raw = serde_json::to_string_pretty(&self.data)?;
        fs::write(&self.path, raw).with_context(|| format!("write {}", self.path.display()))
    }
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
