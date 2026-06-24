//! Sovereign Secure Shield — Network interceptor and ad-blocking engine.
//! Ported from Utah-Omega-23 SOTA protocols.

use serde::{Deserialize, Serialize};
use std::collections::HashSet;
use std::sync::{Arc, Mutex};
use std::path::PathBuf;
use crate::browser::storage_bridge;
use anyhow::{Context, Result};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ShieldEvent {
    pub timestamp: f64,
    pub url: String,
    pub rule: String,
    pub category: String,
}

#[derive(Debug, Clone)]
struct ShieldData {
    events: Vec<ShieldEvent>,
    category_counts: std::collections::HashMap<String, usize>,
    total_blocked: usize,
}

#[derive(Debug, Clone)]
pub struct ShieldEngine {
    blocked_domains: HashSet<String>,
    data: Arc<Mutex<ShieldData>>,
    log_path: PathBuf,
    dirty_count: Arc<std::sync::atomic::AtomicUsize>,
}

impl ShieldEngine {
    pub fn new() -> Result<Self> {
        let mut blocked_domains = HashSet::new();
        // SOTA Baseline rules for Advertising, Tracking, and High-Risk sites
        let baseline = [
            ("doubleclick.net", "Advertising"),
            ("adservice.google.com", "Advertising"),
            ("popads.net", "Aggressive Popup"),
            ("trackingscript.xyz", "Tracker"),
            ("maliciousredirect.su", "Malware Risk"),
            ("fakesecurityalert.cc", "Exploit Attempt"),
            ("adult-tracker-matrix.xyz", "Privacy Tracking"),
            ("popup-clicker.net", "Malicious Popup"),
            ("ad-server.com", "Advertising"),
            ("redirect-trap.su", "Malicious Redirect"),
            ("exploit-delivery.biz", "High-Risk Exploit"),
            ("tracking-pixel.io", "Tracker"),
            ("unsafe-adult-network.com", "Adult/High-Risk"),
            ("scam-alert.su", "Scam"),
            ("future-malware-vector.io", "Akashic Threat"),
            ("identity-theft-matrix.cc", "High-Risk Exploit"),
            ("quantum-drain-script.xyz", "Resource Hijacker"),
        ];

        for (domain, _) in baseline {
            blocked_domains.insert(domain.to_string());
        }

        let log_path = storage_bridge::vault_dir("vault").join("shield_logs.json");
        let events: Vec<ShieldEvent> = if log_path.is_file() {
            let raw = std::fs::read_to_string(&log_path).unwrap_or_default();
            serde_json::from_str(&raw).unwrap_or_default()
        } else {
            Vec::new()
        };

        let mut category_counts = std::collections::HashMap::new();
        let total_blocked = events.len();
        for ev in &events {
            *category_counts.entry(ev.category.clone()).or_insert(0) += 1;
        }

        Ok(Self {
            blocked_domains,
            data: Arc::new(Mutex::new(ShieldData {
                events,
                category_counts,
                total_blocked,
            })),
            log_path,
            dirty_count: Arc::new(std::sync::atomic::AtomicUsize::new(0)),
        })
    }

    /// Evaluates a URL against the threat matrix.
    /// Returns (is_blocked, matched_rule, category).
    pub fn inspect_url(&self, url: &str) -> (bool, String, String) {
        let Some(host) = self.extract_host(url) else {
            return (false, String::new(), String::new());
        };

        // Optimized subdomain traversal
        let mut current = host;
        while let Some(dot_idx) = current.find('.') {
            if self.blocked_domains.contains(current) {
                let category = self.get_category(current);
                return (true, current.to_string(), category);
            }
            // Move to next subdomain level
            current = &current[dot_idx + 1..];
            if !current.contains('.') { break; } // Skip TLD-only checks
        }

        (false, String::new(), String::new())
    }

    fn extract_host<'a>(&self, url: &'a str) -> Option<&'a str> {
        url.strip_prefix("http://")
            .or_else(|| url.strip_prefix("https://"))
            .and_then(|r| r.split('/').next())
            .and_then(|r| r.split(':').next())
    }

    fn get_category(&self, domain: &str) -> String {
        match domain {
            d if d.contains("ad") || d.contains("doubleclick") => "Advertising".to_string(),
            d if d.contains("tracker") || d.contains("pixel") => "Tracker".to_string(),
            d if d.contains("popup") => "Aggressive Popup".to_string(),
            d if d.contains("redirect") || d.contains("trap") => "Malicious Redirect".to_string(),
            d if d.contains("exploit") || d.contains("su") || d.contains("biz") => "Threat Vector".to_string(),
            _ => "Privacy Risk".to_string(),
        }
    }

    pub fn log_block(&self, url: String, rule: String, category: String) {
        let event = ShieldEvent {
            timestamp: std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap_or_default()
                .as_secs_f64(),
            url,
            rule,
            category: category.clone(),
        };
        
        if let Ok(mut data) = self.data.lock() {
            *data.category_counts.entry(category).or_insert(0) += 1;
            data.total_blocked += 1;
            data.events.push(event);
            
            if data.events.len() > 1000 {
                data.events.remove(0);
            }
            
            // Buffered save logic — only write to disk every 20 events to prevent I/O stalls
            let count = self.dirty_count.fetch_add(1, std::sync::atomic::Ordering::SeqCst);
            if count >= 20 {
                self.dirty_count.store(0, std::sync::atomic::Ordering::SeqCst);
                let _ = self.save(&data.events);
            }
        }
    }

    fn save(&self, events: &[ShieldEvent]) -> Result<()> {
        let raw = serde_json::to_string(events)?;
        std::fs::write(&self.log_path, raw).context("save shield logs")?;
        Ok(())
    }

    pub fn get_metrics(&self) -> serde_json::Value {
        let Ok(data) = self.data.lock() else {
            return serde_json::json!({"total_threats_prevented": 0, "breakdown": {}, "status": "Error"});
        };
        
        serde_json::json!({
            "total_threats_prevented": data.total_blocked,
            "breakdown": data.category_counts,
            "system_status": "Protected",
            "last_events": data.events.iter().rev().take(10).collect::<Vec<_>>()
        })
    }
}
