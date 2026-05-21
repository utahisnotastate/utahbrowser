//! Typed IPC payloads (JSON over `window.ipc.postMessage`).

use serde::{Deserialize, Serialize};

/// Commands sent from the WebView transport layer to Rust.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum IpcRequest {
    /// Navigate the active webview to a URL.
    Navigate { url: String },
    /// Verify arbitrary text against the notebook corpus.
    VerifyText { text: String },
    /// Ingest notebooks from the configured knowledge path into Qdrant.
    IngestNotebooks,
    /// Return service health (Ollama, Qdrant, corpus path).
    GetStatus,
    /// Ensure Qdrant is up (auto-start on Windows) before Truth Engine work.
    EnsureServices,
    /// Extract visible page text and verify (active tab).
    VerifyActiveTab,
}

/// Responses pushed back to the UI via `evaluate_script`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum IpcEvent {
    Status { ollama: bool, qdrant: bool, knowledge_path: String, chunks_indexed: usize },
    VerifyResult(VerifyResultPayload),
    IngestProgress { message: String, done: bool },
    Error { message: String },
    EvolutionProposal { path: String, summary: String },
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyResultPayload {
    pub flagged: bool,
    pub similarity: f32,
    pub summary: String,
    pub matched_sources: Vec<String>,
}

impl IpcRequest {
    pub fn parse(raw: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(raw)
    }
}

pub fn event_json(event: &IpcEvent) -> String {
    serde_json::to_string(event).unwrap_or_else(|_| {
        r#"{"event":"error","message":"serialization failed"}"#.to_string()
    })
}
