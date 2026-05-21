//! Cognitive zone persistence — semantic folder bindings with priority weights.

use crate::browser::storage_bridge;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum ZoneHealth {
    Healthy,
    Degraded,
    Critical,
    Unknown,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct KnowledgeZone {
    pub id: String,
    pub path: PathBuf,
    pub label: String,
    pub weight: f32,
    pub direct_map: bool,
    pub health: ZoneHealth,
    pub total_files: usize,
    pub readable_files: usize,
    pub corrupt_files: usize,
    pub indexed_chunks: usize,
    pub last_ingest_unix: Option<u64>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ZonesManifest {
    pub direct_mapping_global: bool,
    pub zones: Vec<KnowledgeZone>,
}

impl Default for ZonesManifest {
    fn default() -> Self {
        Self {
            direct_mapping_global: false,
            zones: Vec::new(),
        }
    }
}

/// Semantic Binding Engine — maps folders to the knowledge manifold.
#[derive(Debug)]
pub struct SemanticBindingStore {
    path: PathBuf,
    manifest: ZonesManifest,
}

impl SemanticBindingStore {
    pub fn load() -> Result<Self> {
        storage_bridge::ensure_vault()?;
        let path = storage_bridge::zones_manifest_path();
        let manifest = if path.is_file() {
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("read zones {}", path.display()))?;
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            ZonesManifest::default()
        };
        Ok(Self { path, manifest })
    }

    pub fn empty() -> Self {
        let _ = storage_bridge::ensure_vault();
        Self {
            path: storage_bridge::zones_manifest_path(),
            manifest: ZonesManifest::default(),
        }
    }

    pub fn save(&self) -> Result<()> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent)?;
        }
        let raw = serde_json::to_string_pretty(&self.manifest)?;
        fs::write(&self.path, raw)?;
        Ok(())
    }

    pub fn manifest(&self) -> &ZonesManifest {
        &self.manifest
    }

    pub fn zones(&self) -> &[KnowledgeZone] {
        &self.manifest.zones
    }

    pub fn direct_mapping_global(&self) -> bool {
        self.manifest.direct_mapping_global
    }

    pub fn set_direct_mapping_global(&mut self, enabled: bool) {
        self.manifest.direct_mapping_global = enabled;
        let _ = self.save();
    }

    pub fn bind_zone(&mut self, path: PathBuf, label: Option<String>) -> Result<KnowledgeZone> {
        let label = label.unwrap_or_else(|| {
            path.file_name()
                .and_then(|n| n.to_str())
                .unwrap_or("Cognitive Zone")
                .to_string()
        });
        let id = format!(
            "zone-{}",
            SystemTime::now()
                .duration_since(UNIX_EPOCH)
                .unwrap_or_default()
                .as_millis()
        );
        let mut zone = KnowledgeZone {
            id,
            path: path.clone(),
            label,
            weight: 1.0,
            direct_map: self.manifest.direct_mapping_global,
            health: ZoneHealth::Unknown,
            total_files: 0,
            readable_files: 0,
            corrupt_files: 0,
            indexed_chunks: 0,
            last_ingest_unix: None,
        };
        zone = crate::binding::health::scan_zone_health(zone)?;
        self.manifest.zones.retain(|z| z.path != path);
        self.manifest.zones.push(zone.clone());
        self.save()?;
        Ok(zone)
    }

    pub fn remove_zone(&mut self, zone_id: &str) -> bool {
        let before = self.manifest.zones.len();
        self.manifest.zones.retain(|z| z.id != zone_id);
        if self.manifest.zones.len() != before {
            let _ = self.save();
            true
        } else {
            false
        }
    }

    pub fn update_weight(&mut self, zone_id: &str, weight: f32) -> Result<()> {
        let weight = weight.clamp(0.1, 5.0);
        if let Some(z) = self.manifest.zones.iter_mut().find(|z| z.id == zone_id) {
            z.weight = weight;
            self.save()?;
        }
        Ok(())
    }

    pub fn update_direct_map(&mut self, zone_id: &str, direct_map: bool) -> Result<()> {
        if let Some(z) = self.manifest.zones.iter_mut().find(|z| z.id == zone_id) {
            z.direct_map = direct_map;
            self.save()?;
        }
        Ok(())
    }

    pub fn refresh_health(&mut self, zone_id: &str) -> Result<KnowledgeZone> {
        let path = self
            .manifest
            .zones
            .iter()
            .find(|z| z.id == zone_id)
            .map(|z| z.path.clone())
            .ok_or_else(|| anyhow::anyhow!("zone not found"))?;
        let label = self
            .manifest
            .zones
            .iter()
            .find(|z| z.id == zone_id)
            .map(|z| z.label.clone())
            .unwrap_or_default();
        let weight = self
            .manifest
            .zones
            .iter()
            .find(|z| z.id == zone_id)
            .map(|z| z.weight)
            .unwrap_or(1.0);
        let direct_map = self
            .manifest
            .zones
            .iter()
            .find(|z| z.id == zone_id)
            .map(|z| z.direct_map)
            .unwrap_or(false);
        let indexed = self
            .manifest
            .zones
            .iter()
            .find(|z| z.id == zone_id)
            .map(|z| z.indexed_chunks)
            .unwrap_or(0);
        let mut zone = KnowledgeZone {
            id: zone_id.to_string(),
            path,
            label,
            weight,
            direct_map,
            health: ZoneHealth::Unknown,
            total_files: 0,
            readable_files: 0,
            corrupt_files: 0,
            indexed_chunks: indexed,
            last_ingest_unix: None,
        };
        zone = crate::binding::health::scan_zone_health(zone)?;
        if let Some(z) = self.manifest.zones.iter_mut().find(|z| z.id == zone_id) {
            *z = zone.clone();
            self.save()?;
        }
        Ok(zone)
    }

    pub fn sanitize_zone(&mut self, zone_id: &str, extensions: &[String]) -> Result<(usize, KnowledgeZone)> {
        let path = self
            .manifest
            .zones
            .iter()
            .find(|z| z.id == zone_id)
            .map(|z| z.path.clone())
            .ok_or_else(|| anyhow::anyhow!("zone not found"))?;
        let removed = crate::binding::health::prune_unreadable(&path, extensions)?;
        let zone = self.refresh_health(zone_id)?;
        Ok((removed, zone))
    }

    pub fn record_ingest(&mut self, zone_id: &str, chunks: usize) -> Result<()> {
        let now = SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap_or_default()
            .as_secs();
        if let Some(z) = self.manifest.zones.iter_mut().find(|z| z.id == zone_id) {
            z.indexed_chunks = chunks;
            z.last_ingest_unix = Some(now);
            self.save()?;
        }
        Ok(())
    }

    pub fn weight_for_source(&self, source: &str) -> f32 {
        for zone in &self.manifest.zones {
            if source.starts_with(zone.path.to_string_lossy().as_ref()) {
                return zone.weight;
            }
        }
        1.0
    }

    pub fn all_zone_paths(&self) -> Vec<PathBuf> {
        self.manifest.zones.iter().map(|z| z.path.clone()).collect()
    }
}
