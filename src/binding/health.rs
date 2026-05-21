//! Zone data health scanning and one-click sanitization.

use super::zones::{KnowledgeZone, ZoneHealth};
use anyhow::Result;
use std::path::Path;

const READ_PROBE_BYTES: usize = 4096;

pub fn scan_zone_health(mut zone: KnowledgeZone) -> Result<KnowledgeZone> {
    let extensions = ["md", "txt", "markdown", "pdf"];
    let root = &zone.path;
    if !root.exists() {
        zone.health = ZoneHealth::Critical;
        return Ok(zone);
    }

    let mut total = 0usize;
    let mut readable = 0usize;
    let mut corrupt = 0usize;

    for path in walk_files(root)? {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !extensions.contains(&ext.as_str()) {
            continue;
        }
        total += 1;
        if probe_readable(&path, &ext) {
            readable += 1;
        } else {
            corrupt += 1;
        }
    }

    zone.total_files = total;
    zone.readable_files = readable;
    zone.corrupt_files = corrupt;
    zone.health = if total == 0 {
        ZoneHealth::Degraded
    } else if corrupt == 0 {
        ZoneHealth::Healthy
    } else if readable > 0 {
        ZoneHealth::Degraded
    } else {
        ZoneHealth::Critical
    };
    Ok(zone)
}

pub fn prune_unreadable(root: &Path, extensions: &[String]) -> Result<usize> {
    let mut removed = 0usize;
    for path in walk_files(root)? {
        let ext = path
            .extension()
            .and_then(|e| e.to_str())
            .unwrap_or("")
            .to_lowercase();
        if !extensions.iter().any(|e| e == &ext) {
            continue;
        }
        if !probe_readable(&path, &ext) {
            if std::fs::remove_file(&path).is_ok() {
                removed += 1;
            }
        }
    }
    Ok(removed)
}

fn probe_readable(path: &Path, ext: &str) -> bool {
    match ext {
        "md" | "markdown" | "txt" => std::fs::read(path)
            .map(|b| !b.is_empty())
            .unwrap_or(false),
        "pdf" => std::fs::read(path)
            .ok()
            .and_then(|b| pdf_extract::extract_text_from_mem(&b).ok())
            .map(|t| !t.trim().is_empty())
            .unwrap_or(false),
        _ => std::fs::read(path)
            .map(|b| b.len() >= READ_PROBE_BYTES.min(1))
            .unwrap_or(false),
    }
}

fn walk_files(root: &Path) -> Result<Vec<std::path::PathBuf>> {
    let mut stack = vec![root.to_path_buf()];
    let mut out = Vec::new();
    while let Some(dir) = stack.pop() {
        let entries = match std::fs::read_dir(&dir) {
            Ok(e) => e,
            Err(_) => continue,
        };
        for entry in entries.flatten() {
            let path = entry.path();
            if path.is_dir() {
                stack.push(path);
            } else if path.is_file() {
                out.push(path);
            }
        }
    }
    Ok(out)
}
