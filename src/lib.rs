//! Utah Browser — privacy-first offline web shell with local truth verification.

pub mod audio;
pub mod binding;
pub mod browser;
pub mod config;
pub mod diagnostics;
pub mod engine;
pub mod ghost_link;
pub mod evolution;
pub mod ipc;
pub mod paths;
pub mod sentinel;
pub mod truth;
pub mod urm;
pub mod vault;

pub use config::AppConfig;

use std::sync::Arc;
use tokio::sync::RwLock;

use binding::SemanticBindingStore;
use truth::TruthEngine;

/// Shared runtime state for IPC, navigation, and background services.
pub struct AppState {
    pub config: AppConfig,
    pub truth: Arc<RwLock<TruthEngine>>,
    pub bindings: Arc<RwLock<SemanticBindingStore>>,
}

impl AppState {
    pub fn new(config: AppConfig) -> anyhow::Result<Self> {
        let bindings = match SemanticBindingStore::load() {
            Ok(b) => b,
            Err(e) => {
                crate::diagnostics::log_step(&format!(
                    "semantic bindings unavailable ({e:#}); using empty store"
                ));
                SemanticBindingStore::empty()
            }
        };
        Ok(Self {
            truth: Arc::new(RwLock::new(TruthEngine::new(config.clone()))),
            bindings: Arc::new(RwLock::new(bindings)),
            config,
        })
    }
}
