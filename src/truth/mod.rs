//! Truth Engine — local RAG ingestion and fact verification.

mod embed;
mod ingest;
pub mod ollama;
pub mod qdrant;
mod services;
mod verify;

pub use verify::VerificationResult;

use crate::binding::SemanticBindingStore;
use crate::config::AppConfig;
use anyhow::Result;
use std::path::Path;
use std::sync::atomic::{AtomicUsize, Ordering};

/// Orchestrates notebook ingestion, embedding, and verification.
pub struct TruthEngine {
    config: AppConfig,
    ollama: ollama::OllamaClient,
    qdrant: qdrant::QdrantClient,
    chunks_indexed: AtomicUsize,
}

impl TruthEngine {
    pub fn new(config: AppConfig) -> Self {
        Self {
            ollama: ollama::OllamaClient::new(config.ollama.clone()),
            qdrant: qdrant::QdrantClient::new(config.qdrant.clone()),
            chunks_indexed: AtomicUsize::new(0),
            config,
        }
    }

    pub fn chunks_indexed(&self) -> usize {
        self.chunks_indexed.load(Ordering::Relaxed)
    }

    pub async fn health(&self) -> (bool, bool) {
        let ollama = self.ollama.ping().await.unwrap_or(false);
        let qdrant = self.qdrant.ping().await.unwrap_or(false);
        (ollama, qdrant)
    }

    /// Ensure Qdrant is reachable (auto-starts Docker on Windows when possible).
    pub async fn ensure_services(&self) -> Result<()> {
        services::ensure_qdrant_ready(&self.config, &self.qdrant).await
    }

    /// Ingest default knowledge path plus all bound cognitive zones.
    pub async fn ingest_notebooks(&self, bindings: &SemanticBindingStore) -> Result<usize> {
        let mut total = 0usize;
        total += self
            .ingest_path(
                &self.config.knowledge.path,
                "default",
                1.0,
                false,
            )
            .await?;
        for zone in bindings.zones() {
            if zone.direct_map {
                tracing::info!(
                    "zone {} direct-map — skipping vector copy",
                    zone.label
                );
                continue;
            }
            total += self
                .ingest_path(&zone.path, &zone.id, zone.weight, zone.direct_map)
                .await?;
        }
        self.chunks_indexed.store(total, Ordering::Relaxed);
        Ok(total)
    }

    /// Ingest a single cognitive zone (live binding).
    pub async fn ingest_zone(
        &self,
        zone_path: &Path,
        zone_id: &str,
        weight: f32,
        direct_map: bool,
    ) -> Result<usize> {
        if direct_map {
            return Ok(0);
        }
        self.ingest_path(zone_path, zone_id, weight, direct_map).await
    }

    async fn ingest_path(
        &self,
        root: &Path,
        zone_id: &str,
        zone_weight: f32,
        direct_map: bool,
    ) -> Result<usize> {
        services::ensure_qdrant_ready(&self.config, &self.qdrant).await?;
        if direct_map {
            return Ok(0);
        }
        let documents = if zone_id == "default" {
            ingest::load_documents(&self.config.knowledge)?
        } else {
            ingest::load_documents_from_path(root, &self.config.knowledge.extensions)?
        };
        let mut total = 0usize;
        for doc in documents {
            let chunks = ingest::chunk_text(
                &doc.text,
                self.config.truth.chunk_chars,
                self.config.truth.chunk_overlap,
            );
            for (idx, chunk) in chunks.iter().enumerate() {
                let vector = self.ollama.embed(chunk).await?;
                let point_id = embed::point_id(&format!("{}#{}", zone_id, doc.source), idx);
                self.qdrant
                    .upsert_point(
                        &point_id,
                        vector,
                        serde_json::json!({
                            "source": doc.source,
                            "chunk_index": idx,
                            "text": chunk,
                            "zone_id": zone_id,
                            "zone_weight": zone_weight,
                        }),
                    )
                    .await?;
                total += 1;
            }
        }
        Ok(total)
    }

    /// Embed text via local Ollama (semantic bookmarks, search, etc.).
    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        self.ollama.embed(text).await
    }

    pub fn qdrant(&self) -> &qdrant::QdrantClient {
        &self.qdrant
    }

    pub fn ollama(&self) -> &ollama::OllamaClient {
        &self.ollama
    }

    /// Ingest a single injected context packet into Qdrant.
    pub async fn ingest_context_snippet(
        &self,
        source: &str,
        label: &str,
        text: &str,
    ) -> Result<()> {
        services::ensure_qdrant_ready(&self.config, &self.qdrant).await?;
        let vector = self.ollama.embed(text).await?;
        let point_id = embed::point_id(&format!("inject:{source}:{label}"), 0);
        self.qdrant
            .upsert_point(
                &point_id,
                vector,
                serde_json::json!({
                    "source": format!("inject/{source}"),
                    "chunk_index": 0,
                    "text": text,
                    "zone_id": "inject",
                    "zone_weight": 1.0,
                }),
            )
            .await?;
        Ok(())
    }

    /// Verify a statement against indexed notebook chunks.
    pub async fn verify_text(&self, text: &str) -> Result<VerificationResult> {
        services::ensure_qdrant_ready(&self.config, &self.qdrant).await?;
        verify::verify_statement(
            text,
            &self.config,
            &self.ollama,
            &self.qdrant,
        )
        .await
    }
}
