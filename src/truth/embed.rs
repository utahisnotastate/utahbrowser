//! Embedding helpers and stable point identifiers.

use sha2::{Digest, Sha256};

/// Deterministic Qdrant point id from source path and chunk index.
pub fn point_id(source: &str, chunk_index: usize) -> String {
    let mut hasher = Sha256::new();
    hasher.update(source.as_bytes());
    hasher.update(chunk_index.to_le_bytes());
    format!("{:x}", hasher.finalize())
}
