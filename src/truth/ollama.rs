//! Ollama HTTP client for local embeddings and optional reasoning.

use crate::config::OllamaConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct OllamaClient {
    http: Client,
    config: OllamaConfig,
}

#[derive(Serialize)]
struct EmbedRequest<'a> {
    model: &'a str,
    prompt: &'a str,
}

#[derive(Deserialize)]
struct EmbedResponse {
    embedding: Vec<f32>,
}

impl OllamaClient {
    pub fn new(config: OllamaConfig) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(config.timeout_secs))
            .build()
            .expect("reqwest client");
        Self { http, config }
    }

    pub async fn ping(&self) -> Result<bool> {
        let url = format!("{}/api/tags", self.config.host.trim_end_matches('/'));
        Ok(self.http.get(url).send().await?.status().is_success())
    }

    pub async fn embed(&self, text: &str) -> Result<Vec<f32>> {
        let url = format!("{}/api/embeddings", self.config.host.trim_end_matches('/'));
        let body = EmbedRequest {
            model: &self.config.embed_model,
            prompt: text,
        };
        let resp: EmbedResponse = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await
            .context("ollama embed request")?
            .error_for_status()
            .context("ollama embed status")?
            .json()
            .await
            .context("ollama embed json")?;
        Ok(resp.embedding)
    }

    pub async fn complete(&self, system: &str, user: &str) -> Result<String> {
        let url = format!("{}/api/generate", self.config.host.trim_end_matches('/'));
        #[derive(Serialize)]
        struct GenRequest<'a> {
            model: &'a str,
            system: &'a str,
            prompt: &'a str,
            stream: bool,
        }
        #[derive(Deserialize)]
        struct GenResponse {
            response: String,
        }
        let body = GenRequest {
            model: &self.config.chat_model,
            system,
            prompt: user,
            stream: false,
        };
        let resp: GenResponse = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(resp.response)
    }
}
