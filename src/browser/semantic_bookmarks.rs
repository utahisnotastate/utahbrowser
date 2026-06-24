//! Semantic bookmark map — local JSON index + Qdrant intention vectors.

use std::sync::{Arc, Mutex};
use crate::browser::storage_bridge;
use crate::config::AppConfig;
use crate::truth::ollama::OllamaClient;
use crate::truth::qdrant::QdrantClient;
use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bookmark {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub intention: String,
    /// Spatial graph proximity (0 = center, higher = farther) for UI layout.
    pub proximity: f32,
}

#[derive(Debug, Default, Serialize, Deserialize)]
struct BookmarkFile {
    next_id: u32,
    items: Vec<Bookmark>,
}

#[derive(Debug, Clone, Serialize)]
pub struct SemanticHit {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub intention: String,
    pub score: f32,
    pub proximity: f32,
}

/// Bookmarks as intention snapshots in the spatial knowledge graph.
#[derive(Debug, Clone)]
pub struct SemanticBookmarkStore {
    path: std::path::PathBuf,
    data: Arc<Mutex<BookmarkFile>>,
    collection: String,
}

impl SemanticBookmarkStore {
    pub fn load(config: &AppConfig) -> Result<Self> {
        storage_bridge::ensure_vault()?;
        let path = storage_bridge::bookmarks_index_path();
        storage_bridge::migrate_legacy_bookmarks_if_needed(&path)?;
        let data = if path.is_file() {
            let raw = fs::read_to_string(&path)
                .with_context(|| format!("read bookmarks {}", path.display()))?;
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            BookmarkFile::default()
        };
        Ok(Self {
            path,
            data: Arc::new(Mutex::new(data)),
            collection: config.browser.bookmarks_collection.clone(),
        })
    }

    pub fn list(&self) -> Vec<Bookmark> {
        self.data.lock().map(|d| d.items.clone()).unwrap_or_default()
    }

    pub fn add_local(&self, title: String, url: String, intention: String) -> Bookmark {
        let mut data = self.data.lock().expect("bookmarks lock");
        let id = data.next_id;
        data.next_id += 1;
        let title = if title.trim().is_empty() {
            url.clone()
        } else {
            title
        };
        let intention = if intention.trim().is_empty() {
            format!("Visit {} — {}", title, url)
        } else {
            intention
        };
        let proximity = proximity_for_count(data.items.len());
        let bm = Bookmark {
            id,
            title,
            url,
            intention,
            proximity,
        };
        data.items.push(bm.clone());
        let _ = self.save_locked(&data);
        bm
    }

    fn save_locked(&self, data: &BookmarkFile) -> Result<()> {
        let raw = serde_json::to_string_pretty(data)?;
        fs::write(&self.path, raw)?;
        Ok(())
    }

    pub fn remove(&self, id: u32) {
        if let Ok(mut data) = self.data.lock() {
            data.items.retain(|b| b.id != id);
            let _ = self.save_locked(&data);
        }
    }

    pub async fn index_in_qdrant(
        &self,
        bm: &Bookmark,
        ollama: &OllamaClient,
        qdrant: &QdrantClient,
    ) -> Result<()> {
        let q = qdrant.with_collection(&self.collection);
        q.ensure_collection().await?;
        let text = format!("{} | {} | {}", bm.intention, bm.title, bm.url);
        let vector = ollama.embed(&text).await?;
        q.upsert_point(
            &format!("bm-{}", bm.id),
            vector,
            serde_json::json!({
                "id": bm.id,
                "title": bm.title,
                "url": bm.url,
                "intention": bm.intention,
                "proximity": bm.proximity,
            }),
        )
        .await?;
        Ok(())
    }

    pub async fn search_semantic(
        &self,
        query: &str,
        ollama: &OllamaClient,
        qdrant: &QdrantClient,
        limit: usize,
    ) -> Result<Vec<SemanticHit>> {
        let q = qdrant.with_collection(&self.collection);
        if !q.ping().await.unwrap_or(false) {
            return Ok(self.fallback_search(query));
        }
        let _ = q.ensure_collection().await;
        let vector = ollama.embed(query).await?;
        let hits = q.search(vector, limit).await?;
        let mut out = Vec::new();
        for (score, payload) in hits {
            if let Some(hit) = payload_to_hit(payload, score) {
                out.push(hit);
            }
        }
        if out.is_empty() {
            return Ok(self.fallback_search(query));
        }
        Ok(out)
    }

    fn fallback_search(&self, query: &str) -> Vec<SemanticHit> {
        let q = query.to_lowercase();
        let Ok(data) = self.data.lock() else { return Vec::new(); };
        data.items
            .iter()
            .filter(|b| {
                b.title.to_lowercase().contains(&q)
                    || b.url.to_lowercase().contains(&q)
                    || b.intention.to_lowercase().contains(&q)
            })
            .map(|b| SemanticHit {
                id: b.id,
                title: b.title.clone(),
                url: b.url.clone(),
                intention: b.intention.clone(),
                score: 1.0,
                proximity: b.proximity,
            })
            .collect()
    }
}

fn proximity_for_count(n: usize) -> f32 {
    // Spiral outward in the spatial graph as more sites are saved.
    ((n as f32) * 0.12 + 0.05).min(1.0)
}

fn payload_to_hit(payload: serde_json::Value, score: f32) -> Option<SemanticHit> {
    Some(SemanticHit {
        id: payload.get("id")?.as_u64()? as u32,
        title: payload.get("title")?.as_str()?.to_string(),
        url: payload.get("url")?.as_str()?.to_string(),
        intention: payload
            .get("intention")
            .and_then(|v| v.as_str())
            .unwrap_or("")
            .to_string(),
        score,
        proximity: payload
            .get("proximity")
            .and_then(|v| v.as_f64())
            .unwrap_or(0.2) as f32,
    })
}
