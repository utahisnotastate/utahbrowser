//! Background verification: embed claim → search Qdrant → score → optional LLM summary.

use crate::config::AppConfig;
use crate::ipc::VerifyResultPayload;
use crate::truth::ollama::OllamaClient;
use crate::truth::qdrant::QdrantClient;
use anyhow::Result;

/// Outcome of a truth-check against indexed notebooks.
#[derive(Debug, Clone)]
pub struct VerificationResult {
    pub flagged: bool,
    pub similarity: f32,
    pub summary: String,
    pub matched_sources: Vec<String>,
}

pub async fn verify_statement(
    text: &str,
    config: &AppConfig,
    ollama: &OllamaClient,
    qdrant: &QdrantClient,
) -> Result<VerificationResult> {
    let trimmed = text.trim();
    if trimmed.is_empty() {
        return Ok(VerificationResult {
            flagged: false,
            similarity: 1.0,
            summary: "No text to verify.".into(),
            matched_sources: vec![],
        });
    }

    let vector = ollama.embed(trimmed).await?;
    let hits = qdrant
        .search(vector, config.truth.max_context_chunks)
        .await?;

    let mut weighted_hits: Vec<(f32, serde_json::Value)> = hits
        .into_iter()
        .map(|(score, payload)| {
            let w = payload
                .get("zone_weight")
                .and_then(|v| v.as_f64())
                .unwrap_or(1.0) as f32;
            (score * w.clamp(0.1, 5.0), payload)
        })
        .collect();
    weighted_hits.sort_by(|a, b| b.0.partial_cmp(&a.0).unwrap_or(std::cmp::Ordering::Equal));

    let best_score = weighted_hits.first().map(|(s, _)| *s).unwrap_or(0.0);
    let threshold = config.truth.similarity_threshold;
    let flagged = best_score < threshold;

    let mut matched_sources = Vec::new();
    let mut context_snippets = Vec::new();
    for (score, payload) in &weighted_hits {
        if let Some(src) = payload.get("source").and_then(|v| v.as_str()) {
            if !matched_sources.contains(&src.to_string()) {
                matched_sources.push(src.to_string());
            }
        }
        if let Some(t) = payload.get("text").and_then(|v| v.as_str()) {
            context_snippets.push(format!("[score={score:.3}] {t}"));
        }
    }

    let summary = if flagged {
        build_flag_summary(ollama, trimmed, best_score, &context_snippets).await?
    } else {
        format!(
            "Statement aligns with notebook corpus (similarity {:.2} ≥ threshold {:.2}).",
            best_score, threshold
        )
    };

    Ok(VerificationResult {
        flagged,
        similarity: best_score,
        summary,
        matched_sources,
    })
}

async fn build_flag_summary(
    ollama: &OllamaClient,
    claim: &str,
    score: f32,
    context: &[String],
) -> Result<String> {
    let ctx = context.join("\n---\n");
    let system = "You are Truth Guard for Utah Browser. Compare the CLAIM to NOTEBOOK excerpts. \
        Be concise. State whether the claim misrepresents the notebooks and why. No telemetry.";
    let user = format!(
        "CLAIM:\n{claim}\n\nBest vector similarity: {score:.3}\n\nNOTEBOOK EXCERPTS:\n{ctx}"
    );
    match ollama.complete(system, &user).await {
        Ok(s) => Ok(s.trim().to_string()),
        Err(e) => Ok(format!(
            "Potential discrepancy (similarity {score:.3}). LLM summary unavailable: {e}"
        )),
    }
}

impl From<VerificationResult> for VerifyResultPayload {
    fn from(v: VerificationResult) -> Self {
        Self {
            flagged: v.flagged,
            similarity: v.similarity,
            summary: v.summary,
            matched_sources: v.matched_sources,
        }
    }
}
