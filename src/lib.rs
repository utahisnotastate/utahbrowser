//! Utah Browser — privacy-first offline web shell with local truth verification.

pub mod audio;
pub mod config;
pub mod engine;
pub mod evolution;
pub mod ipc;
pub mod truth;

pub use config::AppConfig;

use std::sync::Arc;
use tokio::sync::RwLock;

use truth::TruthEngine;

/// Shared runtime state for IPC, navigation, and background services.
pub struct AppState {
    pub config: AppConfig,
    pub truth: Arc<RwLock<TruthEngine>>,
}

impl AppState {
    pub fn new(config: AppConfig) -> Self {
        Self {
            truth: Arc::new(RwLock::new(TruthEngine::new(config.clone()))),
            config,
        }
    }
}
