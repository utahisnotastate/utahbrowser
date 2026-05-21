//! Context-Injection API — external apps push signals into the local vault for RAG ingest.

use anyhow::{Context, Result};
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextPacket {
    pub source: String,
    pub label: String,
    pub text: String,
    pub ts_unix: u64,
    #[serde(default)]
    pub metadata: serde_json::Value,
}

fn queue_path() -> PathBuf {
    crate::paths::sovereign_data_root()
        .join("vault")
        .join("inject")
        .join("queue.jsonl")
}

pub fn enqueue_context(source: &str, label: &str, text: &str, metadata: serde_json::Value) -> Result<()> {
    let path = queue_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let packet = ContextPacket {
        source: source.into(),
        label: label.into(),
        text: text.into(),
        ts_unix: unix_now(),
        metadata,
    };
    let line = serde_json::to_string(&packet)?;
    let mut f = OpenOptions::new()
        .create(true)
        .append(true)
        .open(&path)
        .with_context(|| format!("open inject queue {}", path.display()))?;
    writeln!(f, "{line}")?;
    f.flush()?;
    crate::diagnostics::log_step(&format!("vault inject: {source}/{label} ({} bytes)", text.len()));
    Ok(())
}

/// Drain queue into memory for ingestion (caller embeds into Qdrant).
pub fn ingest_pending(max: usize) -> Result<Vec<ContextPacket>> {
    let path = queue_path();
    if !path.is_file() {
        return Ok(Vec::new());
    }
    let raw = std::fs::read_to_string(&path)?;
    let mut out = Vec::new();
    for line in raw.lines().take(max) {
        if line.trim().is_empty() {
            continue;
        }
        if let Ok(p) = serde_json::from_str::<ContextPacket>(line) {
            out.push(p);
        }
    }
    if !out.is_empty() {
        std::fs::write(&path, "")?;
    }
    Ok(out)
}

fn unix_now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}
