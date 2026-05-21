//! Async semantic integrity checks — decoupled from the Wry render thread.

use crate::ipc::VerifyResultPayload;
use crate::truth::VerificationResult;
use crate::AppState;
use anyhow::Result;

/// Strip HTML to plain text for RAG verification (best-effort, local-only).
pub fn html_to_plain_text(html: &str) -> String {
    let mut out = String::with_capacity(html.len().min(16_000));
    let mut in_tag = false;
    for ch in html.chars().take(32_000) {
        match ch {
            '<' => in_tag = true,
            '>' => in_tag = false,
            _ if !in_tag => {
                if ch.is_whitespace() {
                    if !out.ends_with(' ') {
                        out.push(' ');
                    }
                } else {
                    out.push(ch);
                }
            }
            _ => {}
        }
    }
    let trimmed = out.trim();
    if trimmed.len() > 8_000 {
        trimmed.chars().take(8_000).collect()
    } else {
        trimmed.to_string()
    }
}

/// Non-blocking semantic verification via the local Truth Engine (Ollama + Qdrant).
pub async fn verify_content_integrity(
    state: &AppState,
    html_content: &str,
) -> Result<VerificationResult> {
    let plain = html_to_plain_text(html_content);
    if plain.len() < 12 {
        return Ok(VerificationResult {
            flagged: false,
            similarity: 1.0,
            summary: "Not enough visible text to verify.".into(),
            matched_sources: vec![],
        });
    }
    state.truth.read().await.verify_text(&plain).await
}

pub fn payload_from_result(result: VerificationResult) -> VerifyResultPayload {
    VerifyResultPayload::from(result)
}
