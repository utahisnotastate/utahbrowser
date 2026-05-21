//! Typed IPC payloads (JSON over `window.ipc.postMessage`).

use serde::{Deserialize, Serialize};

/// Commands sent from the WebView transport layer to Rust.
#[derive(Debug, Clone, Deserialize)]
#[serde(tag = "cmd", rename_all = "snake_case")]
pub enum IpcRequest {
    Navigate { url: String },
    NewTab { url: Option<String> },
    CloseTab { tab_id: u32 },
    SwitchTab { tab_id: u32 },
    SuspendTab { tab_id: u32 },
    GoBack,
    GoForward,
    Reload,
    GoHome,
    VerifyText { text: String },
    IngestNotebooks,
    GetStatus,
    EnsureServices,
    VerifyActiveTab,
    SyncBrowser,
    ListBookmarks,
    AddBookmark {
        title: Option<String>,
        url: Option<String>,
        intention: Option<String>,
    },
    RemoveBookmark { bookmark_id: u32 },
    SearchBookmarks { query: String },
    VibeExtension {
        name: String,
        intent: String,
        #[serde(default)]
        trigger: Option<String>,
    },
    ListExtensions,
    RunExtension { name: String, action: Option<String> },
    PrefetchHint { url: String },
    GetGhostLinkStatus,
    GetCalibrationConsole,
    BindKnowledgeZone,
    RemoveZone { zone_id: String },
    SetZoneWeight { zone_id: String, weight: f32 },
    SetZoneDirectMap { zone_id: String, direct_map: bool },
    SetDirectMappingGlobal { enabled: bool },
    SanitizeZone { zone_id: String },
    IngestZone { zone_id: String },
    GetUrmStatus,
    RestoreUrmSnapshot,
    DismissUrmOverlay,
    /// Switch layout: web = browsing strip + content below; app = full chrome shell.
    SetShellMode { mode: String },
    /// Clear safe-mode flag after a successful dual-webview session.
    ClearSafeMode,
    /// Third-party context injection into vault queue (RAG ingest).
    InjectContext {
        source: String,
        label: String,
        text: String,
        #[serde(default)]
        metadata: Option<serde_json::Value>,
    },
    IngestContextQueue,
    /// Memory anchor — snapshot tab URL + scroll + intention.
    CreateMemoryAnchor {
        title: Option<String>,
        intention: Option<String>,
        #[serde(default)]
        scroll_x: Option<f32>,
        #[serde(default)]
        scroll_y: Option<f32>,
    },
    ListMemoryAnchors,
}

/// Responses pushed back to the UI via `evaluate_script`.
#[derive(Debug, Clone, Serialize)]
#[serde(tag = "event", rename_all = "snake_case")]
pub enum IpcEvent {
    Status {
        ollama: bool,
        qdrant: bool,
        knowledge_path: String,
        chunks_indexed: usize,
    },
    VerifyResult(VerifyResultPayload),
    IngestProgress { message: String, done: bool },
    Error { message: String },
    EvolutionProposal { path: String, summary: String },
    TabsUpdated {
        tabs: Vec<TabPayload>,
        active_id: u32,
        home_url: String,
    },
    NavigationChanged {
        tab_id: u32,
        url: String,
        title: String,
    },
    BookmarksUpdated { bookmarks: Vec<BookmarkPayload> },
    SpatialBookmarks { hits: Vec<SpatialBookmarkPayload> },
    ExtensionsUpdated { extensions: Vec<ExtensionPayload> },
    PrefetchQueued { urls: Vec<String> },
    /// Memory-buffer injection ready — load via `utah://localhost/buffer/{id}`.
    PrefetchBuffered {
        url: String,
        buffer_id: String,
        buffer_uri: String,
        bytes: usize,
    },
    TabSuspended { tab_id: u32 },
    ExtensionRan { name: String, result: i32 },
    GhostLinkStatus {
        active: bool,
        message: String,
        events: Vec<GhostEventPayload>,
    },
    CalibrationConsole {
        zones: Vec<ZonePayload>,
        telemetry: TelemetryPayload,
        direct_mapping_global: bool,
    },
    ZoneBound { zone: ZonePayload },
    UrmStatus {
        active: bool,
        message: String,
        coherence: f32,
        status: String,
        overlay: Option<UrmOverlayPayload>,
        mutagenesis: Option<UrmMutagenesisPayload>,
        snapshots: Vec<String>,
    },
    SensoryTheme {
        mode: String,
        accent: String,
        contrast: String,
        audio_rms: f32,
    },
    MemoryAnchorsUpdated {
        anchors: Vec<MemoryAnchorPayload>,
    },
    ContextInjected {
        queued: usize,
    },
}

#[derive(Debug, Clone, Serialize)]
pub struct MemoryAnchorPayload {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub intention: String,
    pub scroll_x: f32,
    pub scroll_y: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrmOverlayPayload {
    pub message: String,
    pub severity: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct UrmMutagenesisPayload {
    pub summary: String,
    pub target_file: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct ZonePayload {
    pub id: String,
    pub path: String,
    pub label: String,
    pub weight: f32,
    pub direct_map: bool,
    pub health: String,
    pub total_files: usize,
    pub readable_files: usize,
    pub corrupt_files: usize,
    pub indexed_chunks: usize,
}

#[derive(Debug, Clone, Serialize)]
pub struct TelemetryPayload {
    pub ollama_online: bool,
    pub qdrant_online: bool,
    pub embed_latency_ms: Option<u32>,
    pub vector_points: Option<u64>,
    pub vector_dim: u64,
    pub chunks_indexed: usize,
    pub gpu_note: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct GhostEventPayload {
    pub ts: String,
    pub trigger: String,
    pub entropy: f32,
    pub summary: Option<String>,
}

#[derive(Debug, Clone, Serialize)]
pub struct TabPayload {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub suspended: bool,
}

#[derive(Debug, Clone, Serialize)]
pub struct BookmarkPayload {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub intention: String,
    pub proximity: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct SpatialBookmarkPayload {
    pub id: u32,
    pub title: String,
    pub url: String,
    pub intention: String,
    pub score: f32,
    pub proximity: f32,
}

#[derive(Debug, Clone, Serialize)]
pub struct ExtensionPayload {
    pub name: String,
    pub trigger: String,
    pub intent: String,
}

#[derive(Debug, Clone, Serialize)]
pub struct VerifyResultPayload {
    pub flagged: bool,
    pub similarity: f32,
    pub summary: String,
    pub matched_sources: Vec<String>,
}

impl IpcRequest {
    pub fn parse(raw: &str) -> Result<Self, serde_json::Error> {
        serde_json::from_str(raw)
    }
}

pub fn event_json(event: &IpcEvent) -> String {
    serde_json::to_string(event).unwrap_or_else(|_| {
        r#"{"event":"error","message":"serialization failed"}"#.to_string()
    })
}
