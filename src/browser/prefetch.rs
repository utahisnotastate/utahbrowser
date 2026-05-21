//! Predictive prefetch kernel (Time-Loop / intent-resolution queue).

use std::collections::VecDeque;
use std::sync::atomic::{AtomicU64, Ordering};

const MAX_QUEUE: usize = 16;

static HINT_COUNTER: AtomicU64 = AtomicU64::new(0);

/// Background URL prefetch queue (content WebView pre-warm is engine-driven).
#[derive(Debug, Default)]
pub struct PrefetchKernel {
    queue: VecDeque<String>,
    enabled: bool,
}

impl PrefetchKernel {
    pub fn new(enabled: bool) -> Self {
        Self {
            queue: VecDeque::new(),
            enabled,
        }
    }

    pub fn hint(&mut self, url: String) {
        if !self.enabled || url.trim().is_empty() {
            return;
        }
        if self.queue.iter().any(|u| u == &url) {
            return;
        }
        tracing::debug!("[UTAH_TIME_LOOP] prefetch hint queued: {url}");
        self.queue.push_back(url);
        while self.queue.len() > MAX_QUEUE {
            self.queue.pop_front();
        }
    }

    pub fn pop(&mut self) -> Option<String> {
        self.queue.pop_front()
    }

    pub fn pending(&self) -> Vec<String> {
        self.queue.iter().cloned().collect()
    }

    pub fn next_hint_id(&self) -> u64 {
        HINT_COUNTER.fetch_add(1, Ordering::Relaxed)
    }
}
