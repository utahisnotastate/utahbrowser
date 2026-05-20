//! Truth Engine — local RAG ingestion and fact verification.

mod embed;
mod ingest;
pub mod ollama;
mod qdrant;
mod verify;

pub use verify::VerificationResult;

use crate::config::AppConfig;
use anyhow::Result;
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

    /// Ingest all notebooks from the knowledge directory into Qdrant.
    pub async fn ingest_notebooks(&self) -> Result<usize> {
        self.qdrant.ensure_collection().await?;
        let documents = ingest::load_documents(&self.config.knowledge)?;
        let mut total = 0usize;

        for doc in documents {
            let chunks = ingest::chunk_text(
                &doc.text,
                self.config.truth.chunk_chars,
                self.config.truth.chunk_overlap,
            );
            for (idx, chunk) in chunks.iter().enumerate() {
                let vector = self.ollama.embed(chunk).await?;
                let point_id = embed::point_id(&doc.source, idx);
                self.qdrant
                    .upsert_point(
                        &point_id,
                        vector,
                        serde_json::json!({
                            "source": doc.source,
                            "chunk_index": idx,
                            "text": chunk,
                        }),
                    )
                    .await?;
                total += 1;
            }
        }

        self.chunks_indexed.store(total, Ordering::Relaxed);
        Ok(total)
    }

    /// Verify a statement against indexed notebook chunks.
    pub async fn verify_text(&self, text: &str) -> Result<VerificationResult> {
        verify::verify_statement(
            text,
            &self.config,
            &self.ollama,
            &self.qdrant,
        )
        .await
    }
}
