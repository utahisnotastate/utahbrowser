//! State-compressor tab manager — inactive tabs serialize to the vault cache (MMap-ready files).

use crate::browser::storage_bridge;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use std::path::PathBuf;
use std::time::Instant;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TabState {
    pub url: String,
    pub title: String,
    pub scroll_pos: (f32, f32),
    /// Reserved for future virtual-DOM snapshot capture from the content WebView.
    pub dom_snapshot: Vec<u8>,
    #[serde(skip, default = "Instant::now")]
    pub last_accessed: Instant,
}

#[derive(Debug, Clone, Serialize)]
pub struct TabInfo {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub suspended: bool,
}

/// Single-process tab graph with disk-backed suspension for memory sovereignty.
#[derive(Debug)]
pub struct TabManager {
    active: HashMap<u32, TabState>,
    suspended: HashMap<u32, PathBuf>,
    metadata: HashMap<u32, (String, String)>,
    order: Vec<u32>,
    active_id: u32,
    next_id: u32,
    home_url: String,
    suspend_on_switch: bool,
}

impl TabManager {
    pub fn new(home_url: String, suspend_on_switch: bool) -> Result<Self> {
        storage_bridge::ensure_vault()?;
        let mut mgr = Self {
            active: HashMap::new(),
            suspended: HashMap::new(),
            metadata: HashMap::new(),
            order: Vec::new(),
            active_id: 0,
            next_id: 1,
            home_url: home_url.clone(),
            suspend_on_switch,
        };
        mgr.new_tab(Some(home_url));
        Ok(mgr)
    }

    pub fn home_url(&self) -> &str {
        &self.home_url
    }

    pub fn new_tab(&mut self, url: Option<String>) -> u32 {
        let url = url.unwrap_or_else(|| self.home_url.clone());
        let id = self.next_id;
        self.next_id += 1;
        let title = title_from_url(&url);
        self.active.insert(
            id,
            TabState {
                url: url.clone(),
                title: title.clone(),
                scroll_pos: (0.0, 0.0),
                dom_snapshot: Vec::new(),
                last_accessed: Instant::now(),
            },
        );
        self.metadata.insert(id, (title, url));
        self.order.push(id);
        self.active_id = id;
        id
    }

    pub fn close_tab(&mut self, id: u32) -> Result<bool> {
        if self.order.len() <= 1 {
            return Ok(false);
        }
        self.metadata.remove(&id);
        if self.active.remove(&id).is_some() {
            let _ = fs::remove_file(storage_bridge::tab_cache_path(id));
        } else if self.suspended.remove(&id).is_some() {
            let _ = fs::remove_file(storage_bridge::tab_cache_path(id));
        } else {
            return Ok(false);
        }
        self.order.retain(|&t| t != id);
        if self.active_id == id {
            self.active_id = *self.order.last().unwrap_or(&id);
            if !self.active.contains_key(&self.active_id) {
                self.resume_tab(self.active_id)?;
            }
        }
        Ok(true)
    }

    pub fn switch_tab(&mut self, id: u32) -> Result<Option<String>> {
        if !self.order.contains(&id) {
            return Ok(None);
        }
        if self.suspend_on_switch && self.active_id != id {
            self.suspend_tab(self.active_id)?;
        }
        if self.suspended.contains_key(&id) {
            self.resume_tab(id)?;
        }
        if let Some(state) = self.active.get_mut(&id) {
            state.last_accessed = Instant::now();
        }
        self.active_id = id;
        Ok(self.active_url())
    }

    pub fn suspend_tab(&mut self, tab_id: u32) -> Result<()> {
        let Some(state) = self.active.remove(&tab_id) else {
            return Ok(());
        };
        let path = storage_bridge::tab_cache_path(tab_id);
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let bytes = bincode::serialize(&state).context("serialize tab state")?;
        fs::write(&path, &bytes).with_context(|| format!("write tab cache {}", path.display()))?;
        self.suspended.insert(tab_id, path);
        tracing::info!("[UTAH_KERNEL] Tab {tab_id} suspended to disk.");
        Ok(())
    }

    pub fn resume_tab(&mut self, tab_id: u32) -> Result<()> {
        let path = if let Some(p) = self.suspended.remove(&tab_id) {
            p
        } else {
            let p = storage_bridge::tab_cache_path(tab_id);
            if p.is_file() {
                p
            } else {
                anyhow::bail!("tab {tab_id} is not suspended");
            }
        };
        let bytes = fs::read(&path).with_context(|| format!("read tab cache {}", path.display()))?;
        let mut state: TabState = bincode::deserialize(&bytes).context("deserialize tab state")?;
        state.last_accessed = Instant::now();
        self.active.insert(tab_id, state);
        if !self.order.contains(&tab_id) {
            self.order.push(tab_id);
        }
        tracing::info!("[UTAH_KERNEL] Tab {tab_id} resumed from vault cache.");
        Ok(())
    }

    pub fn active_id(&self) -> u32 {
        self.active_id
    }

    pub fn active_url(&self) -> Option<String> {
        self.active
            .get(&self.active_id)
            .map(|t| t.url.clone())
            .or_else(|| {
                self.suspended
                    .get(&self.active_id)
                    .and_then(|p| fs::read(p).ok())
                    .and_then(|b| bincode::deserialize::<TabState>(&b).ok())
                    .map(|t| t.url)
            })
    }

    pub fn active_title(&self) -> Option<String> {
        self.get_title(self.active_id)
    }

    pub fn get_title(&self, id: u32) -> Option<String> {
        self.metadata.get(&id).map(|(t, _)| t.clone())
    }

    pub fn get_title_for_url(&self, url: &str) -> Option<String> {
        self.metadata
            .values()
            .find(|(_, u)| u == url)
            .map(|(t, _)| t.clone())
    }

    pub fn navigate_active(&mut self, url: String) -> String {
        if let Some(state) = self.active.get_mut(&self.active_id) {
            let prev = title_from_url(&state.url);
            state.url = url.clone();
            if state.title.is_empty() || state.title == prev {
                state.title = title_from_url(&url);
            }
            self.metadata.insert(self.active_id, (state.title.clone(), state.url.clone()));
        }
        url
    }

    pub fn set_active_title(&mut self, title: String) {
        if let Some(state) = self.active.get_mut(&self.active_id) {
            if !title.trim().is_empty() {
                state.title = title;
                self.metadata.insert(self.active_id, (state.title.clone(), state.url.clone()));
            }
        }
    }

    pub fn set_active_url(&mut self, url: String) {
        if let Some(state) = self.active.get_mut(&self.active_id) {
            let prev = title_from_url(&state.url);
            state.url = url.clone();
            if state.title.is_empty() || state.title == prev {
                state.title = title_from_url(&url);
            }
            state.last_accessed = Instant::now();
            self.metadata.insert(self.active_id, (state.title.clone(), state.url.clone()));
        }
    }

    pub fn mark_active(&mut self) {
        if let Some(state) = self.active.get_mut(&self.active_id) {
            state.last_accessed = Instant::now();
        }
    }

    pub fn get_inactive_tabs(&self, timeout_secs: u64) -> Vec<u32> {
        let now = Instant::now();
        self.active
            .iter()
            .filter(|(&id, state)| {
                id != self.active_id && now.duration_since(state.last_accessed).as_secs() > timeout_secs
            })
            .map(|(&id, _)| id)
            .collect()
    }

    pub fn get_active_mut(&mut self, id: u32) -> Option<&mut TabState> {
        self.active.get_mut(&id)
    }

    pub fn set_scroll(&mut self, tab_id: u32, x: f32, y: f32) {
        if let Some(state) = self.active.get_mut(&tab_id) {
            state.scroll_pos = (x, y);
        }
    }

    pub fn snapshot(&self) -> (Vec<TabInfo>, u32) {
        let tabs = self
            .order
            .iter()
            .filter_map(|id| self.tab_info(*id))
            .collect();
        (tabs, self.active_id)
    }

    fn tab_info(&self, id: u32) -> Option<TabInfo> {
        let (title, url) = self.metadata.get(&id).cloned().or_else(|| {
            // Fallback: try to recover from active/suspended if metadata is missing (should not happen)
            if let Some(state) = self.active.get(&id) {
                Some((state.title.clone(), state.url.clone()))
            } else if self.suspended.contains_key(&id) {
                let path = storage_bridge::tab_cache_path(id);
                if let Ok(bytes) = fs::read(path) {
                    if let Ok(state) = bincode::deserialize::<TabState>(&bytes) {
                        return Some((state.title, state.url));
                    }
                }
                None
            } else {
                None
            }
        })?;

        Some(TabInfo {
            id,
            title,
            url,
            suspended: self.suspended.contains_key(&id),
        })
    }
}

fn title_from_url(url: &str) -> String {
    url.trim_start_matches("https://")
        .trim_start_matches("http://")
        .split('/')
        .next()
        .unwrap_or("Memory Brick")
        .to_string()
}
