//! Qdrant REST client for local vector storage.

use crate::config::QdrantConfig;
use anyhow::{Context, Result};
use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::time::Duration;

#[derive(Clone)]
pub struct QdrantClient {
    http: Client,
    config: QdrantConfig,
}

#[derive(Serialize)]
struct UpsertBody {
    points: Vec<PointStruct>,
}

#[derive(Serialize)]
struct PointStruct {
    id: String,
    vector: Vec<f32>,
    payload: serde_json::Value,
}

#[derive(Deserialize)]
struct SearchResponse {
    result: Vec<ScoredPoint>,
}

#[derive(Deserialize)]
struct ScoredPoint {
    score: f32,
    payload: Option<serde_json::Value>,
}

impl QdrantClient {
    pub fn new(config: QdrantConfig) -> Self {
        let http = Client::builder()
            .timeout(Duration::from_secs(30))
            .build()
            .expect("reqwest client");
        Self { http, config }
    }

    fn base(&self) -> String {
        self.config.url.trim_end_matches('/').to_string()
    }

    pub async fn ping(&self) -> Result<bool> {
        let url = format!("{}/collections", self.base());
        Ok(self.http.get(url).send().await?.status().is_success())
    }

    pub async fn ensure_collection(&self) -> Result<()> {
        let url = format!("{}/collections/{}", self.base(), self.config.collection);
        if self.http.get(&url).send().await?.status().is_success() {
            return Ok(());
        }
        #[derive(Serialize)]
        struct CreateBody {
            vectors: VectorsConfig,
        }
        #[derive(Serialize)]
        struct VectorsConfig {
            size: u64,
            distance: &'static str,
        }
        let create_url = format!("{}/collections/{}", self.base(), self.config.collection);
        let body = CreateBody {
            vectors: VectorsConfig {
                size: self.config.vector_size,
                distance: "Cosine",
            },
        };
        self.http
            .put(&create_url)
            .json(&body)
            .send()
            .await?
            .error_for_status()
            .context("create qdrant collection")?;
        Ok(())
    }

    pub async fn upsert_point(
        &self,
        id: &str,
        vector: Vec<f32>,
        payload: serde_json::Value,
    ) -> Result<()> {
        let url = format!(
            "{}/collections/{}/points",
            self.base(),
            self.config.collection
        );
        let body = UpsertBody {
            points: vec![PointStruct {
                id: id.to_string(),
                vector,
                payload,
            }],
        };
        self.http
            .put(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()
            .context("qdrant upsert")?;
        Ok(())
    }

    pub async fn search(&self, vector: Vec<f32>, limit: usize) -> Result<Vec<(f32, serde_json::Value)>> {
        let url = format!(
            "{}/collections/{}/points/search",
            self.base(),
            self.config.collection
        );
        #[derive(Serialize)]
        struct SearchBody {
            vector: Vec<f32>,
            limit: usize,
            with_payload: bool,
        }
        let body = SearchBody {
            vector,
            limit,
            with_payload: true,
        };
        let resp: SearchResponse = self
            .http
            .post(&url)
            .json(&body)
            .send()
            .await?
            .error_for_status()?
            .json()
            .await?;
        Ok(resp
            .result
            .into_iter()
            .filter_map(|p| p.payload.map(|pl| (p.score, pl)))
            .collect())
    }
}
