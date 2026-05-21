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
    #[serde(default)]
    pub browser: BrowserConfig,
    #[serde(default)]
    pub ghost_link: GhostLinkConfig,
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

#[derive(Debug, Clone, Deserialize)]
pub struct BrowserConfig {
    /// Qdrant collection for intention snapshots (semantic bookmarks).
    pub bookmarks_collection: String,
    /// Serialize inactive tabs to vault cache on switch.
    pub suspend_on_switch: bool,
    /// Queue predictive prefetch hints (Time-Loop / intent-resolution).
    pub prefetch_enabled: bool,
    /// Max RAM for `utah://localhost/buffer/*` compositor cache (MiB).
    #[serde(default = "default_prefetch_buffer_mb")]
    pub prefetch_buffer_max_mb: u32,
}

fn default_prefetch_buffer_mb() -> u32 {
    8
}

impl Default for BrowserConfig {
    fn default() -> Self {
        Self {
            bookmarks_collection: "utah_bookmarks".into(),
            suspend_on_switch: true,
            prefetch_enabled: true,
            prefetch_buffer_max_mb: default_prefetch_buffer_mb(),
        }
    }
}

#[derive(Debug, Clone, Deserialize)]
pub struct GhostLinkConfig {
    pub enabled: bool,
    pub entropy_threshold: f32,
    pub frame_interval_ms: u32,
    pub buffer_seconds: f32,
    pub vision_model: String,
}

impl Default for GhostLinkConfig {
    fn default() -> Self {
        Self {
            enabled: true,
            entropy_threshold: 0.12,
            frame_interval_ms: 500,
            buffer_seconds: 5.0,
            vision_model: "llava".into(),
        }
    }
}

impl AppConfig {
    /// Load configuration from disk, applying environment overrides.
    pub fn load() -> Result<Self> {
        let config_path = crate::paths::config_path();
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

        let install = crate::paths::install_root();
        config.evolution.watch_paths = config
            .evolution
            .watch_paths
            .iter()
            .map(|p| {
                if p.is_absolute() {
                    p.clone()
                } else {
                    install.join(p)
                }
            })
            .collect();

        Ok(config)
    }

    /// Resolved proposals directory under sovereign data (never install root).
    pub fn proposals_dir(&self) -> PathBuf {
        crate::paths::evolution_proposals_dir()
    }
}

// Minimal TOML parsing without extra crate — use serde via manual or add toml dep
// I used toml::from_str but didn't add toml to Cargo.toml - need to add it
