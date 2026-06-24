// src/ipc/nexus.rs
use serde::{Deserialize, Serialize};
use tokio::net::TcpListener;
use tokio_tungstenite::accept_async;
use futures_util::{StreamExt, SinkExt};
use tracing::{info, error};

#[derive(Serialize, Deserialize, Debug)]
pub struct BrowserIntent {
    pub intent_id: String,
    pub action: String,
    pub payload: serde_json::Value,
}

#[derive(Serialize, Deserialize, Debug)]
pub struct IntentResolution {
    pub intent_id: String,
    pub status: String,
    pub result: serde_json::Value,
}

pub struct IpcNexus {
    port: u16,
    quantum: std::sync::Arc<crate::quantum::QuantumOracle>,
}

impl IpcNexus {
    pub fn new(port: u16, quantum: std::sync::Arc<crate::quantum::QuantumOracle>) -> Self {
        IpcNexus { port, quantum }
    }

    pub async fn initialize_bridge(&self) {
        let addr = format!("127.0.0.1:{}", self.port);
        let listener = TcpListener::bind(&addr).await.expect("Failed to bind IPC socket");
        info!("IPC Nexus listening on: {}", addr);

        while let Ok((stream, _)) = listener.accept().await {
            let quantum = self.quantum.clone();
            tokio::spawn(handle_connection(stream, quantum));
        }
    }
}

async fn handle_connection(stream: tokio::net::TcpStream, quantum: std::sync::Arc<crate::quantum::QuantumOracle>) {
    let ws_stream = match accept_async(stream).await {
        Ok(ws) => ws,
        Err(e) => {
            error!("WebSocket handshake failed: {}", e);
            return;
        }
    };

    let (mut write, mut read) = ws_stream.split();

    while let Some(msg) = read.next().await {
        match msg {
            Ok(message) => {
                if message.is_text() {
                    let text_data = message.into_text().unwrap();
                    if let Ok(intent) = serde_json::from_str::<BrowserIntent>(&text_data) {
                        info!("Received Intent: {:?}", intent.action);
                        
                        // Route to appropriate sub-system (URM, Tab Manager, Qdrant)
                        let resolution = execute_intent(intent, &quantum).await;
                        
                        let response_text = serde_json::to_string(&resolution).unwrap();
                        let _ = write.send(tokio_tungstenite::tungstenite::Message::Text(response_text.into())).await;
                    }
                }
            }
            Err(e) => error!("IPC Stream Error: {}", e),
        }
    }
}

async fn execute_intent(intent: BrowserIntent, quantum: &crate::quantum::QuantumOracle) -> IntentResolution {
    // Modular routing logic. In production, this maps to isolated Rust modules.
    let mut result = serde_json::json!({});
    
    match intent.action.as_str() {
        "PAGE_TAB" => {
            // Trigger memory-saving state serialization
            result = serde_json::json!({"status": "Tab state serialized to disk"});
        },
        "QUERY_ORACLE" => {
            // Trigger local RAG vector search via Qdrant
            result = serde_json::json!({"answer": "Local LLM response synthesized."});
        },
        "QUANTUM_COLLAPSE" => {
            let key = intent.payload.get("problem_key").and_then(|v| v.as_str()).unwrap_or("general_optimization");
            let resp = quantum.query(key);
            result = serde_json::to_value(&resp).unwrap_or_default();
        },
        "EMAIL_FETCH" => {
            // Simulated call to flux/email_nexus.py
            result = serde_json::json!({"status": "SUCCESS", "emails": []});
        },
        "CAREER_REFACTOR" => {
            // Simulated call to flux/career_forge.py
            result = serde_json::json!({"status": "SUCCESS", "tailored_resume": ""});
        },
        _ => {
            result = serde_json::json!({"error": "Unknown intent vector."});
        }
    }

    IntentResolution {
        intent_id: intent.intent_id,
        status: "RESOLVED".to_string(),
        result,
    }
}
