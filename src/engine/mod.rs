//! Wry WebView shell, custom protocol asset server, and IPC dispatch.

use crate::ipc::{event_json, IpcEvent, IpcRequest, VerifyResultPayload};
use crate::AppState;
use anyhow::{Context, Result};
use http::header::CONTENT_TYPE;
use std::path::PathBuf;
use std::sync::Arc;
use tao::{
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::WindowBuilder,
};
use wry::http::{Request, Response};
use wry::WebViewBuilder;

const ASSET_SCHEME: &str = "utah";

const TRANSPORT_SCRIPT: &str = r#"
(function() {
  if (window.__utahTransport) return;
  window.__utahTransport = true;
  window.utahSend = function(payload) {
    if (window.ipc && window.ipc.postMessage) {
      window.ipc.postMessage(typeof payload === 'string' ? payload : JSON.stringify(payload));
    }
  };
  window.utahOnEvent = function(ev) {
    var detail = typeof ev === 'string' ? JSON.parse(ev) : ev;
    window.dispatchEvent(new CustomEvent('utah-ipc', { detail: detail }));
  };
})();
"#;

/// User events routed through the Tao event loop (main-thread safe).
#[derive(Debug, Clone)]
pub enum UserEvent {
    Ipc(String),
}

/// Run the native window + webview event loop.
pub fn run(state: Arc<AppState>, runtime: Arc<tokio::runtime::Runtime>) {
    let assets_root = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/ui");
    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title(&state.config.ui.window_title)
        .with_inner_size(LogicalSize::new(1280.0, 800.0))
        .build(&event_loop)
        .expect("create window");

    let assets_for_protocol = assets_root.clone();
    let proxy_nav = proxy.clone();
    let proxy_ipc = proxy.clone();

    let webview = WebViewBuilder::new()
        .with_custom_protocol(ASSET_SCHEME.into(), move |_id, request| {
            if let Some(resp) = try_handle_navigate_route(&request, &proxy_nav) {
                return match resp {
                    Ok(r) => r.map(Into::into),
                    Err(e) => Response::builder()
                        .header(CONTENT_TYPE, "text/plain")
                        .status(500)
                        .body(e.to_string().into_bytes())
                        .unwrap()
                        .map(Into::into),
                };
            }
            match serve_asset(&assets_for_protocol, request) {
                Ok(r) => r.map(Into::into),
                Err(e) => Response::builder()
                    .header(CONTENT_TYPE, "text/plain")
                    .status(500)
                    .body(e.to_string().into_bytes())
                    .unwrap()
                    .map(Into::into),
            }
        })
        .with_ipc_handler(move |req| {
            let _ = proxy_ipc.send_event(UserEvent::Ipc(req.body().clone()));
        })
        .with_initialization_script(TRANSPORT_SCRIPT)
        .with_url(format!("{ASSET_SCHEME}://localhost/index.html"))
        .build(&window)
        .expect("build webview");

    let state_loop = state.clone();
    let rt = runtime;

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match &event {
            Event::UserEvent(UserEvent::Ipc(body)) => {
                if let Err(e) = rt.block_on(handle_ipc(body, &state_loop, &webview)) {
                    tracing::error!("ipc: {e:#}");
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });
}

fn try_handle_navigate_route(
    request: &Request<Vec<u8>>,
    proxy: &EventLoopProxy<UserEvent>,
) -> Option<Result<Response<Vec<u8>>>> {
    let path = request.uri().path();
    if path != "/navigate" {
        return None;
    }
    let uri = request.uri().to_string();
    let url = parse_query(&uri).get("url").cloned().unwrap_or_default();
    if !url.is_empty() {
        let msg = serde_json::json!({ "cmd": "navigate", "url": url }).to_string();
        let _ = proxy.send_event(UserEvent::Ipc(msg));
    }
    Some(Ok(Response::builder()
        .header(CONTENT_TYPE, "text/html")
        .body(b"OK".to_vec())
        .unwrap()))
}

fn parse_query(uri: &str) -> std::collections::HashMap<String, String> {
    let mut map = std::collections::HashMap::new();
    if let Some(q) = uri.split('?').nth(1) {
        for pair in q.split('&') {
            if let Some((k, v)) = pair.split_once('=') {
                map.insert(percent_decode(k), percent_decode(v));
            }
        }
    }
    map
}

fn percent_decode(s: &str) -> String {
    let mut out = String::new();
    let bytes = s.as_bytes();
    let mut i = 0;
    while i < bytes.len() {
        if bytes[i] == b'%' && i + 2 < bytes.len() {
            if let Ok(ch) = u8::from_str_radix(
                std::str::from_utf8(&bytes[i + 1..i + 3]).unwrap_or(""),
                16,
            ) {
                out.push(ch as char);
                i += 3;
                continue;
            }
        }
        out.push(bytes[i] as char);
        i += 1;
    }
    out
}

fn serve_asset(root: &PathBuf, request: Request<Vec<u8>>) -> Result<Response<Vec<u8>>> {
    let path = request.uri().path();
    let rel = if path == "/" || path.is_empty() {
        "index.html"
    } else {
        path.trim_start_matches('/')
    };
    let file = root.join(rel);
    let canonical = std::fs::canonicalize(&file)
        .with_context(|| format!("asset not found: {}", file.display()))?;
    let root_canon = std::fs::canonicalize(root)?;
    if !canonical.starts_with(&root_canon) {
        anyhow::bail!("path traversal blocked");
    }
    let content = std::fs::read(&canonical)?;
    let mime = mime_guess::from_path(&canonical)
        .first_or_octet_stream()
        .to_string();
    Ok(Response::builder()
        .header(CONTENT_TYPE, mime)
        .body(content)
        .unwrap())
}

async fn handle_ipc(body: &str, state: &Arc<AppState>, webview: &wry::WebView) -> Result<()> {
    let req = match IpcRequest::parse(body) {
        Ok(r) => r,
        Err(e) => {
            push_event(webview, IpcEvent::Error {
                message: format!("invalid ipc json: {e}"),
            });
            return Ok(());
        }
    };

    match req {
        IpcRequest::Navigate { url } => {
            webview.load_url(&url)?;
        }
        IpcRequest::GetStatus => {
            let (ollama, qdrant) = state.truth.read().await.health().await;
            push_event(
                webview,
                IpcEvent::Status {
                    ollama,
                    qdrant,
                    knowledge_path: state.config.knowledge.path.display().to_string(),
                    chunks_indexed: state.truth.read().await.chunks_indexed(),
                },
            );
        }
        IpcRequest::EnsureServices => {
            match state.truth.read().await.ensure_services().await {
                Ok(()) => {
                    let (ollama, qdrant) = state.truth.read().await.health().await;
                    push_event(
                        webview,
                        IpcEvent::Status {
                            ollama,
                            qdrant,
                            knowledge_path: state.config.knowledge.path.display().to_string(),
                            chunks_indexed: state.truth.read().await.chunks_indexed(),
                        },
                    );
                }
                Err(e) => {
                    push_event(
                        webview,
                        IpcEvent::Error {
                            message: format!("{e:#}"),
                        },
                    );
                }
            }
        }
        IpcRequest::IngestNotebooks => {
            push_event(
                webview,
                IpcEvent::IngestProgress {
                    message: "Ingesting notebooks…".into(),
                    done: false,
                },
            );
            let count = state.truth.write().await.ingest_notebooks().await?;
            push_event(
                webview,
                IpcEvent::IngestProgress {
                    message: format!("Indexed {count} chunks."),
                    done: true,
                },
            );
        }
        IpcRequest::VerifyText { text } => {
            let result = state.truth.read().await.verify_text(&text).await?;
            push_event(
                webview,
                IpcEvent::VerifyResult(VerifyResultPayload::from(result)),
            );
        }
        IpcRequest::VerifyActiveTab => {
            push_event(
                webview,
                IpcEvent::Error {
                    message: "Active-tab OCR/accessibility capture is planned; paste text or use the iframe verify control.".into(),
                },
            );
        }
    }
    Ok(())
}

fn push_event(webview: &wry::WebView, event: IpcEvent) {
    let js = format!(
        "window.utahOnEvent && window.utahOnEvent({});",
        event_json(&event)
    );
    if let Err(e) = webview.evaluate_script(&js) {
        tracing::warn!("push ipc event: {e}");
    }
}
