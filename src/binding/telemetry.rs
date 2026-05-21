//! Inference telemetry for the Calibration Console (local services only).

use crate::config::AppConfig;
use crate::truth::ollama::OllamaClient;
use crate::truth::qdrant::QdrantClient;
use anyhow::Result;
use serde::Serialize;
use std::time::Instant;

#[derive(Debug, Clone, Serialize)]
pub struct InferenceTelemetry {
    pub ollama_online: bool,
    pub qdrant_online: bool,
    pub embed_latency_ms: Option<u32>,
    pub vector_points: Option<u64>,
    pub vector_dim: u64,
    pub chunks_indexed: usize,
    pub gpu_note: String,
}

pub async fn collect(
    config: &AppConfig,
    ollama: &OllamaClient,
    qdrant: &QdrantClient,
    chunks_indexed: usize,
) -> Result<InferenceTelemetry> {
    let ollama_online = ollama.ping().await.unwrap_or(false);
    let qdrant_online = qdrant.ping().await.unwrap_or(false);

    let embed_latency_ms = if ollama_online {
        let start = Instant::now();
        let ok = ollama.embed("utah telemetry probe").await.is_ok();
        if ok {
            Some(start.elapsed().as_millis().min(u128::from(u32::MAX)) as u32)
        } else {
            None
        }
    } else {
        None
    };

    let vector_points = if qdrant_online {
        qdrant.collection_points().await.ok()
    } else {
        None
    };

    Ok(InferenceTelemetry {
        ollama_online,
        qdrant_online,
        embed_latency_ms,
        vector_points,
        vector_dim: config.qdrant.vector_size,
        chunks_indexed,
        gpu_note: if embed_latency_ms.unwrap_or(999) < 80 {
            "Inference fast — likely GPU/NPU acceleration".into()
        } else if ollama_online {
            "Inference active — CPU or loaded model".into()
        } else {
            "Ollama offline — start ollama serve".into()
        },
    })
}
