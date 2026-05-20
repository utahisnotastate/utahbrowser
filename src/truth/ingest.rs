//! Notebook ingestion from the knowledge directory (PDF, Markdown, plain text).

use crate::config::KnowledgeConfig;
use anyhow::{Context, Result};
use std::path::{Path, PathBuf};

/// A loaded document ready for chunking and embedding.
pub struct Document {
    pub source: String,
    pub text: String,
}

/// Walk the knowledge path and load supported file types.
pub fn load_documents(knowledge: &KnowledgeConfig) -> Result<Vec<Document>> {
    let root = &knowledge.path;
    if !root.exists() {
        tracing::warn!(
            "knowledge path does not exist: {} — ingest will be empty until configured",
            root.display()
        );
        return Ok(Vec::new());
    }

    let mut docs = Vec::new();
    for entry in walkdir_flat(root)? {
        let ext = entry
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !knowledge.extensions.iter().any(|e| e == &ext) {
            continue;
        }
        let text = match ext.as_str() {
            "pdf" => load_pdf(&entry)?,
            "md" | "markdown" => load_text(&entry)?,
            "txt" => load_text(&entry)?,
            _ => continue,
        };
        if text.trim().is_empty() {
            continue;
        }
        docs.push(Document {
            source: entry.to_string_lossy().into_owned(),
            text,
        });
    }
    Ok(docs)
}

fn walkdir_flat(root: &Path) -> Result<Vec<PathBuf>> {
    let mut stack = vec![root.to_path_buf()];
    let mut files = Vec::new();
    while let Some(dir) = stack.pop() {
        for entry in std::fs::read_dir(&dir).with_context(|| format!("read_dir {}", dir.display()))? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                files.push(path);
            }
        }
    }
    Ok(files)
}

fn load_text(path: &Path) -> Result<String> {
    Ok(std::fs::read_to_string(path).with_context(|| format!("read {}", path.display()))?)
}

fn load_pdf(path: &Path) -> Result<String> {
    let bytes = std::fs::read(path).with_context(|| format!("read pdf {}", path.display()))?;
    pdf_extract::extract_text_from_mem(&bytes)
        .with_context(|| format!("extract pdf {}", path.display()))
}

/// Split text into overlapping chunks for embedding.
pub fn chunk_text(text: &str, chunk_chars: usize, overlap: usize) -> Vec<String> {
    let chars: Vec<char> = text.chars().collect();
    if chars.is_empty() {
        return Vec::new();
    }
    let step = chunk_chars.saturating_sub(overlap).max(1);
    let mut chunks = Vec::new();
    let mut start = 0;
    while start < chars.len() {
        let end = (start + chunk_chars).min(chars.len());
        let chunk: String = chars[start..end].iter().collect();
        if !chunk.trim().is_empty() {
            chunks.push(chunk);
        }
        if end >= chars.len() {
            break;
        }
        start += step;
    }
    chunks
}
