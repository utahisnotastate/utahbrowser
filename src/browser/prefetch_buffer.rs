//! In-memory prefetch cache served via `utah://localhost/buffer/{id}` (compositor fast path).

use anyhow::{bail, Result};
use sha2::{Digest, Sha256};
use std::collections::HashMap;

const DEFAULT_MAX_BYTES: usize = 8 * 1024 * 1024;

#[derive(Debug, Clone)]
pub struct CachedResource {
    pub url: String,
    pub content_type: String,
    pub body: Vec<u8>,
}

/// GPU/compositor bypass: hot resources are served from RAM through the custom protocol.
#[derive(Debug)]
pub struct PrefetchBuffer {
    by_id: HashMap<String, CachedResource>,
    url_to_id: HashMap<String, String>,
    total_bytes: usize,
    max_bytes: usize,
}

impl PrefetchBuffer {
    pub fn new(max_bytes: usize) -> Self {
        Self {
            by_id: HashMap::new(),
            url_to_id: HashMap::new(),
            total_bytes: 0,
            max_bytes: if max_bytes == 0 {
                DEFAULT_MAX_BYTES
            } else {
                max_bytes
            },
        }
    }

    pub fn id_for_url(url: &str) -> String {
        let hash = Sha256::digest(url.as_bytes());
        hash[..12]
            .iter()
            .map(|b| format!("{b:02x}"))
            .collect::<String>()
    }

    pub fn buffer_uri(id: &str) -> String {
        format!("utah://localhost/buffer/{id}")
    }

    pub fn insert(&mut self, url: String, content_type: String, body: Vec<u8>) -> String {
        let id = Self::id_for_url(&url);
        let size = body.len();
        if let Some(old_id) = self.url_to_id.get(&url) {
            if let Some(old) = self.by_id.remove(old_id) {
                self.total_bytes = self.total_bytes.saturating_sub(old.body.len());
            }
        }
        while self.total_bytes + size > self.max_bytes && !self.by_id.is_empty() {
            if let Some(key) = self.by_id.keys().next().cloned() {
                if let Some(evicted) = self.by_id.remove(&key) {
                    self.url_to_id.remove(&evicted.url);
                    self.total_bytes = self.total_bytes.saturating_sub(evicted.body.len());
                }
            }
        }
        self.url_to_id.insert(url.clone(), id.clone());
        self.total_bytes += size;
        self.by_id.insert(
            id.clone(),
            CachedResource {
                url,
                content_type,
                body,
            },
        );
        id
    }

    pub fn get(&self, id: &str) -> Option<&CachedResource> {
        self.by_id.get(id)
    }

    pub fn get_by_url(&self, url: &str) -> Option<&CachedResource> {
        self.url_to_id
            .get(url)
            .and_then(|id| self.by_id.get(id))
    }
}

pub fn try_serve_buffer(
    buffer: &PrefetchBuffer,
    path: &str,
) -> Result<Option<(String, Vec<u8>)>> {
    let prefix = "/buffer/";
    if !path.starts_with(prefix) {
        return Ok(None);
    }
    let id = path.trim_start_matches(prefix).trim_matches('/');
    if id.is_empty() || id.contains("..") {
        bail!("invalid buffer id");
    }
    let Some(entry) = buffer.get(id) else {
        return Ok(None);
    };
    Ok(Some((entry.content_type.clone(), entry.body.clone())))
}
