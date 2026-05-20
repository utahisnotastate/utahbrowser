//! Application configuration loaded from `config/default.toml` with env overrides.

use anyhow::{Context, Result};
use serde::Deserialize;
use std::path::PathBuf;

/// Root configuration for Utah Browser (local-only services).
#[derive(Debug, Clone, Deserialize)]
pub struct AppConfig {
    pub knowledge: KnowledgeConfig,
    pub ollama: OllamaConfig,
    pub qdrant: QdrantConfig,
    pub truth: TruthConfig,
    pub audio: AudioConfig,
    pub evolution: EvolutionConfig,
    pub ui: UiConfig,
}

#[derive(Debug, Clone, Deserialize)]
pub struct KnowledgeConfig {
    pub path: PathBuf,
    pub extensions: Vec<String>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct OllamaConfig {
    pub host: String,
    pub embed_model: String,
    pub chat_model: String,
    pub timeout_secs: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct QdrantConfig {
    pub url: String,
    pub collection: String,
    pub vector_size: u64,
}

#[derive(Debug, Clone, Deserialize)]
pub struct TruthConfig {
    pub similarity_threshold: f32,
    pub chunk_chars: usize,
    pub chunk_overlap: usize,
    pub max_context_chunks: usize,
}

#[derive(Debug, Clone, Deserialize)]
pub struct AudioConfig {
    pub enabled: bool,
    pub sample_rate_hint: u32,
}

#[derive(Debug, Clone, Deserialize)]
pub struct EvolutionConfig {
    pub enabled: bool,
    pub watch_paths: Vec<PathBuf>,
    pub debounce_ms: u64,
    pub proposals_dir: PathBuf,
}

#[derive(Debug, Clone, Deserialize)]
pub struct UiConfig {
    pub start_url: String,
    pub window_title: String,
}

impl AppConfig {
    /// Load configuration from disk, applying environment overrides.
    pub fn load() -> Result<Self> {
        let manifest_dir = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        let config_path = manifest_dir.join("config/default.toml");
        let raw = std::fs::read_to_string(&config_path)
            .with_context(|| format!("read config at {}", config_path.display()))?;
        let mut config: AppConfig = toml::from_str(&raw)
            .map_err(|e| anyhow::anyhow!("parse config: {e}"))?;

        if let Ok(path) = std::env::var("UTAH_KNOWLEDGE_PATH") {
            config.knowledge.path = PathBuf::from(path);
        }
        if let Ok(host) = std::env::var("OLLAMA_HOST") {
            config.ollama.host = host;
        }
        if let Ok(url) = std::env::var("QDRANT_URL") {
            config.qdrant.url = url;
        }

        Ok(config)
    }

    /// Resolved proposals directory (created on demand).
    pub fn proposals_dir(&self) -> PathBuf {
        let base = PathBuf::from(env!("CARGO_MANIFEST_DIR"));
        base.join(&self.evolution.proposals_dir)
    }
}

// Minimal TOML parsing without extra crate — use serde via manual or add toml dep
// I used toml::from_str but didn't add toml to Cargo.toml - need to add it
