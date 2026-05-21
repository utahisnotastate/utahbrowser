//! Intent-resolution worker: DNS touch + bounded GET into the memory buffer.

use super::prefetch_buffer::PrefetchBuffer;
use anyhow::{Context, Result};
use reqwest::Client;
use std::sync::{Arc, Mutex};
use std::time::Duration;

const MAX_FETCH_BYTES: usize = 256 * 1024;

fn host_port(url: &str) -> Option<(&str, u16)> {
    let (scheme, rest) = if let Some(r) = url.strip_prefix("https://") {
        ("https", r)
    } else if let Some(r) = url.strip_prefix("http://") {
        ("http", r)
    } else {
        return None;
    };
    let authority = rest.split('/').next()?.split('#').next()?.split('?').next()?;
    let (host, port) = if let Some((h, p)) = authority.split_once(':') {
        (h, p.parse().ok()?)
    } else {
        let default = if scheme == "https" { 443 } else { 80 };
        (authority, default)
    };
    if host.is_empty() {
        return None;
    }
    Some((host, port))
}

pub async fn warm_url(
    client: &Client,
    buffer: Arc<Mutex<PrefetchBuffer>>,
    url: &str,
) -> Result<String> {
    if let Some((host, port)) = host_port(url) {
        let _ = tokio::time::timeout(
            Duration::from_secs(2),
            tokio::net::lookup_host((host, port)),
        )
        .await;
    }

    let resp = client
        .get(url)
        .header(reqwest::header::RANGE, format!("bytes=0-{}", MAX_FETCH_BYTES - 1))
        .timeout(Duration::from_secs(8))
        .send()
        .await
        .context("prefetch GET")?;

    let status = resp.status();
    let content_type = resp
        .headers()
        .get(reqwest::header::CONTENT_TYPE)
        .and_then(|v| v.to_str().ok())
        .unwrap_or("application/octet-stream")
        .to_string();

    let bytes = resp.bytes().await.context("prefetch body")?;
    let body = bytes.as_ref();
    let body = if body.len() > MAX_FETCH_BYTES {
        &body[..MAX_FETCH_BYTES]
    } else {
        body
    };

    if !status.is_success() && status.as_u16() != 206 {
        anyhow::bail!("prefetch HTTP {}", status);
    }

    let id = {
        let mut buf = buffer.lock().expect("prefetch buffer lock");
        buf.insert(url.to_string(), content_type, body.to_vec())
    };
    tracing::debug!("[UTAH_TIME_LOOP] buffered {url} -> {id} ({} bytes)", body.len());
    Ok(id)
}
