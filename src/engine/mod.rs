//! Wry WebView shell (chrome + content), custom protocol, and IPC dispatch.

mod truth_guard;

use crate::browser::{
    ExtensionRuntime, ExtensionTrigger, PrefetchKernel, SemanticBookmarkStore,
};
use crate::browser::tab_manager::TabManager;
use crate::ghost_link::GhostLinkBridge;
use crate::binding::{pick_and_bind, telemetry, ZoneHealth};
use crate::ipc::{
    event_json, BookmarkPayload, ExtensionPayload, GhostEventPayload, IpcEvent, IpcRequest,
    MemoryAnchorPayload, SpatialBookmarkPayload, TabPayload, TelemetryPayload,
    UrmMutagenesisPayload, UrmOverlayPayload, VerifyResultPayload, ZonePayload,
};
use crate::browser::MemoryAnchorStore;
use crate::urm::UrmBridge;
use crate::diagnostics;
use crate::AppState;
use anyhow::{Context, Result};
use http::header::CONTENT_TYPE;
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tao::{
    dpi::LogicalPosition,
    dpi::LogicalSize,
    event::{Event, WindowEvent},
    event_loop::{ControlFlow, EventLoopBuilder, EventLoopProxy},
    window::Window,
    window::WindowBuilder,
};
use wry::http::{Request, Response};
use wry::{PageLoadEvent, Rect, WebView, WebViewBuilder};

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
    PageUrl(String),
    PageTitle(String),
    /// Deferred navigation so WebView2 can finish HWND setup first.
    DeferredLoad(String),
    /// Poll Ghost-Link theme.json for haptic UI palette.
    SensoryPoll,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellLayout {
    /// Chrome strip + content pane (default).
    Dual,
    /// Full-window single webview — auto-fallback when dual crashes.
    Single,
}

struct ShellViews {
    layout: ShellLayout,
    chrome: Option<WebView>,
    content: WebView,
}

impl ShellViews {
    fn ui(&self) -> &WebView {
        self.chrome.as_ref().unwrap_or(&self.content)
    }
}

/// Matches `--chrome-strip-h` in mockup.css (logical px).
const CHROME_STRIP_H: f64 = 112.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShellMode {
    /// Compact top chrome + native content fills the rest (google.com, etc.).
    Web,
    /// Full-window Utah shell (dashboard, truth guard, …); content hidden.
    App,
}

impl Default for ShellMode {
    fn default() -> Self {
        Self::Web
    }
}

fn window_logical_size(window: &Window) -> (f64, f64) {
    let size = window.inner_size();
    let scale = window.scale_factor();
    (
        size.width as f64 / scale,
        size.height as f64 / scale,
    )
}

struct BrowserUi {
    tabs: TabManager,
    bookmarks: SemanticBookmarkStore,
    anchors: MemoryAnchorStore,
    extensions: ExtensionRuntime,
    prefetch: PrefetchKernel,
    ghost: GhostLinkBridge,
    urm: UrmBridge,
    shell_mode: ShellMode,
}

impl BrowserUi {
    fn new(config: &crate::config::AppConfig) -> Result<Self> {
        let home = config.ui.start_url.clone();
        let mut extensions = ExtensionRuntime::new()?;
        let _ = extensions.load_all();
        Ok(Self {
            tabs: TabManager::new(home, config.browser.suspend_on_switch)?,
            bookmarks: SemanticBookmarkStore::load(config)?,
            anchors: MemoryAnchorStore::load().unwrap_or_else(|e| {
                diagnostics::log_step(&format!("memory anchors unavailable ({e:#})"));
                MemoryAnchorStore::empty()
            }),
            extensions,
            prefetch: PrefetchKernel::new(config.browser.prefetch_enabled),
            ghost: GhostLinkBridge::new(),
            urm: UrmBridge::default(),
            shell_mode: ShellMode::Web,
        })
    }

    /// Clear stale Nexus overlay files so startup never shows a grey warning banner.
    fn clear_stale_urm_overlay(&self) {
        let path = crate::browser::storage_bridge::urm_browser_overlay();
        if path.is_file() {
            let _ = std::fs::remove_file(path);
        }
    }

    fn absorb_ghost_prefetch(&mut self, config: &crate::config::AppConfig) {
        if !config.ghost_link.enabled {
            return;
        }
        if let Ok(Some(url)) = self.ghost.consume_prefetch_url() {
            self.prefetch.hint(url);
        }
    }
}

/// Run the native window + webview event loop (with crash recovery).
pub fn run(state: Arc<AppState>, runtime: Arc<tokio::runtime::Runtime>) {
    if let Err(e) = run_inner(state, runtime) {
        let msg = format!("{e:#}");
        diagnostics::record_boot_failure(&msg);
        diagnostics::show_fatal(
            "Utah Browser",
            &diagnostics::fatal_message(&msg),
        );
    }
}

fn run_inner(state: Arc<AppState>, runtime: Arc<tokio::runtime::Runtime>) -> Result<()> {
    let recovery = diagnostics::load_recovery();
    let mut safe_mode = recovery.should_use_safe_mode();
    if std::env::var("UTAH_DEMO_MODE").ok().as_deref() == Some("1") {
        safe_mode = true;
        diagnostics::log_step("demo mode: forcing single webview (UTAH_DEMO_MODE=1)");
    }
    if safe_mode {
        diagnostics::log_step("starting in safe mode (single webview fallback)");
    }

    let home_url = state.config.ui.start_url.clone();
    let browser = Arc::new(Mutex::new(
        BrowserUi::new(&state.config).context("initialize browser UI state")?,
    ));

    let assets_root = crate::paths::assets_ui_dir();
    diagnostics::log_step(&format!("assets: {}", assets_root.display()));

    let event_loop = EventLoopBuilder::<UserEvent>::with_user_event().build();
    let proxy = event_loop.create_proxy();

    let window = WindowBuilder::new()
        .with_title(&state.config.ui.window_title)
        .with_inner_size(LogicalSize::new(1280.0, 800.0))
        .build(&event_loop)
        .context("create window")?;
    diagnostics::log_step("window created");

    let proxy_nav = proxy.clone();
    let proxy_ipc = proxy.clone();
    let proxy_page = proxy.clone();
    let proxy_title = proxy.clone();
    let init_script = chrome_init_script(&home_url);

    let initial_mode = ShellMode::Web;
    if let Ok(mut b) = browser.lock() {
        b.shell_mode = initial_mode;
    }

    let shell = boot_shell(
        &window,
        &assets_root,
        &init_script,
        &proxy_nav,
        &proxy_ipc,
        &proxy_page,
        &proxy_title,
        initial_mode,
        safe_mode,
    )?;

    let mode_label = match shell.layout {
        ShellLayout::Dual => "dual",
        ShellLayout::Single => "single-safe",
    };
    diagnostics::log_step(&format!("shell ready ({mode_label})"));
    crate::sentinel::signal_shell_ready(mode_label);

    let proxy_sensory = proxy.clone();
    std::thread::spawn(move || {
        loop {
            std::thread::sleep(std::time::Duration::from_secs(2));
            let _ = proxy_sensory.send_event(UserEvent::SensoryPoll);
        }
    });

    let start_url = {
        let b = browser.lock().map_err(|_| anyhow::anyhow!("browser lock poisoned"))?;
        b.tabs
            .active_url()
            .unwrap_or_else(|| home_url.clone())
    };

    if shell.layout == ShellLayout::Dual {
        if let (Some(chrome), _) = (&shell.chrome, &shell.content) {
            if let Err(e) = apply_shell_layout(&window, chrome, &shell.content, initial_mode) {
                diagnostics::log_step(&format!("layout warn: {e:#}"));
            }
            push_tabs_and_bookmarks(chrome, &browser, &home_url);
            push_navigation(chrome, &browser);
            push_shell_mode(chrome, initial_mode);
        }
    } else {
        let title = format!("Utah Browser — Safe Mode — {start_url}");
        window.set_title(&title);
    }

    if let Ok(b) = browser.lock() {
        b.clear_stale_urm_overlay();
    }

    let _ = proxy.send_event(UserEvent::DeferredLoad(start_url));

    let state_loop = state.clone();
    let rt = runtime;
    let browser_loop = browser.clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;
        // shell moved here - can't clone ui above, remove that line

        match &event {
            Event::UserEvent(UserEvent::Ipc(body)) => {
                if let Err(e) = rt.block_on(handle_ipc(
                    body,
                    &state_loop,
                    &window,
                    shell.ui(),
                    &shell.content,
                    &browser_loop,
                    &home_url,
                    shell.layout,
                )) {
                    diagnostics::log_step(&format!("ipc error: {e:#}"));
                    push_event(shell.ui(), IpcEvent::Error {
                        message: format!("{e:#}"),
                    });
                }
            }
            Event::UserEvent(UserEvent::DeferredLoad(url)) => {
                diagnostics::log_step(&format!("loading {url}"));
                if let Err(e) = shell.content.load_url(url) {
                    diagnostics::log_step(&format!("load_url failed: {e:#}"));
                }
            }
            Event::UserEvent(UserEvent::SensoryPoll) => {
                if let Ok(b) = browser_loop.lock() {
                    if let Ok(Some(theme)) = b.ghost.read_theme() {
                        push_event(
                            shell.ui(),
                            IpcEvent::SensoryTheme {
                                mode: theme.mode,
                                accent: theme.accent,
                                contrast: theme.contrast,
                                audio_rms: theme.audio_rms,
                            },
                        );
                    }
                }
            }
            Event::UserEvent(UserEvent::PageUrl(url)) => {
                if let Ok(mut b) = browser_loop.lock() {
                    b.tabs.set_active_url(url.clone());
                    push_navigation(shell.ui(), &browser_loop);
                    push_tabs(shell.ui(), &browser_loop, &home_url);
                }
            }
            Event::UserEvent(UserEvent::PageTitle(title)) => {
                if let Ok(mut b) = browser_loop.lock() {
                    b.tabs.set_active_title(title.clone());
                    push_tabs(shell.ui(), &browser_loop, &home_url);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                if shell.layout == ShellLayout::Dual {
                    if let Some(chrome) = &shell.chrome {
                        if let Ok(b) = browser_loop.lock() {
                            let _ = apply_shell_layout(
                                &window,
                                chrome,
                                &shell.content,
                                b.shell_mode,
                            );
                        }
                    }
                }
            }
            Event::WindowEvent {
                event: WindowEvent::CloseRequested,
                ..
            } => {
                diagnostics::record_boot_success(mode_label);
                *control_flow = ControlFlow::Exit;
            }
            _ => {}
        }
    });

    Ok(())
}

fn boot_shell(
    window: &Window,
    assets_root: &PathBuf,
    init_script: &str,
    proxy_nav: &EventLoopProxy<UserEvent>,
    proxy_ipc: &EventLoopProxy<UserEvent>,
    proxy_page: &EventLoopProxy<UserEvent>,
    proxy_title: &EventLoopProxy<UserEvent>,
    initial_mode: ShellMode,
    force_single: bool,
) -> Result<ShellViews> {
    if force_single {
        return boot_single_webview(window, proxy_page, proxy_title, proxy_ipc);
    }
    match boot_dual_webview(
        window,
        assets_root,
        init_script,
        proxy_nav,
        proxy_ipc,
        proxy_page,
        proxy_title,
        initial_mode,
    ) {
        Ok(views) => Ok(views),
        Err(e) => {
            diagnostics::log_error("dual webview boot", &format!("{e:#}"));
            diagnostics::log_step("auto-fix: retrying with single webview (safe mode)");
            boot_single_webview(window, proxy_page, proxy_title, proxy_ipc)
        }
    }
}

fn boot_dual_webview(
    window: &Window,
    assets_root: &PathBuf,
    init_script: &str,
    proxy_nav: &EventLoopProxy<UserEvent>,
    proxy_ipc: &EventLoopProxy<UserEvent>,
    proxy_page: &EventLoopProxy<UserEvent>,
    proxy_title: &EventLoopProxy<UserEvent>,
    initial_mode: ShellMode,
) -> Result<ShellViews> {
    diagnostics::log_step("boot: dual webview");

    let content = WebViewBuilder::new()
        .with_url("about:blank")
        .with_bounds(content_bounds_for_mode(window, initial_mode))
        .with_back_forward_navigation_gestures(true)
        .with_on_page_load_handler({
            let proxy = proxy_page.clone();
            move |event, url| {
                if matches!(event, PageLoadEvent::Finished) {
                    let _ = proxy.send_event(UserEvent::PageUrl(url));
                }
            }
        })
        .with_document_title_changed_handler({
            let proxy = proxy_title.clone();
            move |title| {
                let _ = proxy.send_event(UserEvent::PageTitle(title));
            }
        })
        .build_as_child(window)
        .context("build content webview")?;

    let assets = assets_root.clone();
    let chrome = WebViewBuilder::new()
        .with_custom_protocol(ASSET_SCHEME.into(), {
            let proxy = proxy_nav.clone();
            move |_id, request| {
                if let Some(resp) = try_handle_navigate_route(&request, &proxy) {
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
                match serve_asset(&assets, request) {
                    Ok(r) => r.map(Into::into),
                    Err(e) => Response::builder()
                        .header(CONTENT_TYPE, "text/plain")
                        .status(500)
                        .body(e.to_string().into_bytes())
                        .unwrap()
                        .map(Into::into),
                }
            }
        })
        .with_ipc_handler({
            let proxy = proxy_ipc.clone();
            move |req| {
                let _ = proxy.send_event(UserEvent::Ipc(req.body().clone()));
            }
        })
        .with_initialization_script(init_script)
        .with_url(format!("{ASSET_SCHEME}://localhost/index.html"))
        .with_bounds(chrome_bounds_for_mode(window, initial_mode))
        .build_as_child(window)
        .context("build chrome webview")?;

    Ok(ShellViews {
        layout: ShellLayout::Dual,
        chrome: Some(chrome),
        content,
    })
}

fn boot_single_webview(
    window: &Window,
    proxy_page: &EventLoopProxy<UserEvent>,
    proxy_title: &EventLoopProxy<UserEvent>,
    proxy_ipc: &EventLoopProxy<UserEvent>,
) -> Result<ShellViews> {
    diagnostics::log_step("boot: single webview (safe mode)");

    let (w, h) = window_logical_size(window);
    let content = WebViewBuilder::new()
        .with_url("about:blank")
        .with_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(w.max(1.0), h.max(1.0)).into(),
        })
        .with_back_forward_navigation_gestures(true)
        .with_on_page_load_handler({
            let proxy = proxy_page.clone();
            move |event, url| {
                if matches!(event, PageLoadEvent::Finished) {
                    let _ = proxy.send_event(UserEvent::PageUrl(url));
                }
            }
        })
        .with_document_title_changed_handler({
            let proxy = proxy_title.clone();
            move |title| {
                let _ = proxy.send_event(UserEvent::PageTitle(title));
            }
        })
        .with_ipc_handler({
            let proxy = proxy_ipc.clone();
            move |req| {
                let _ = proxy.send_event(UserEvent::Ipc(req.body().clone()));
            }
        })
        .build_as_child(window)
        .context("build single webview")?;

    Ok(ShellViews {
        layout: ShellLayout::Single,
        chrome: None,
        content,
    })
}

const BOOT_SCRIPT: &str = r#"
document.addEventListener('DOMContentLoaded', function() {
  document.body.classList.add('utah-mode-web');
  var web = document.getElementById('view-web');
  if (web) web.checked = true;
  if (window.utahSetShellMode) window.utahSetShellMode('web');
});
"#;

fn chrome_init_script(home_url: &str) -> String {
    let home = serde_json::to_string(home_url).unwrap_or_else(|_| "\"\"".into());
    format!(
        "{TRANSPORT_SCRIPT}\nwindow.__utahHomeUrl = {home};\n{BOOT_SCRIPT}\n"
    )
}

fn chrome_bounds_for_mode(window: &Window, mode: ShellMode) -> Rect {
    let (w, h) = window_logical_size(window);
    let chrome_h = match mode {
        ShellMode::Web => CHROME_STRIP_H.min(h),
        ShellMode::App => h,
    };
    Rect {
        position: LogicalPosition::new(0.0, 0.0).into(),
        size: LogicalSize::new(w.max(1.0), chrome_h.max(1.0)).into(),
    }
}

fn content_bounds_for_mode(window: &Window, mode: ShellMode) -> Rect {
    let (w, h) = window_logical_size(window);
    match mode {
        ShellMode::Web => {
            let top = CHROME_STRIP_H.min(h);
            Rect {
                position: LogicalPosition::new(0.0, top).into(),
                size: LogicalSize::new(w.max(1.0), (h - top).max(1.0)).into(),
            }
        }
        ShellMode::App => hidden_content_bounds(),
    }
}

fn hidden_content_bounds() -> Rect {
    Rect {
        position: LogicalPosition::new(-10_000.0, -10_000.0).into(),
        size: LogicalSize::new(1.0, 1.0).into(),
    }
}

fn apply_shell_layout(
    window: &Window,
    chrome: &WebView,
    content: &WebView,
    mode: ShellMode,
) -> Result<()> {
    chrome.set_bounds(chrome_bounds_for_mode(window, mode))?;
    content.set_bounds(content_bounds_for_mode(window, mode))?;
    Ok(())
}

fn push_shell_mode(chrome: &WebView, mode: ShellMode) {
    let name = if mode == ShellMode::Web { "web" } else { "app" };
    let cls = if mode == ShellMode::Web {
        "document.body.classList.add('utah-mode-web');document.body.classList.remove('utah-mode-app');"
    } else {
        "document.body.classList.add('utah-mode-app');document.body.classList.remove('utah-mode-web');"
    };
    let _ = chrome.evaluate_script(&format!(
        "{cls}if(window.utahOnShellMode)window.utahOnShellMode('{name}');"
    ));
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

async fn handle_ipc(
    body: &str,
    state: &Arc<AppState>,
    _window: &Window,
    chrome: &WebView,
    content: &WebView,
    browser: &Arc<Mutex<BrowserUi>>,
    home_url: &str,
    layout: ShellLayout,
) -> Result<()> {
    let req = match IpcRequest::parse(body) {
        Ok(r) => r,
        Err(e) => {
            push_event(chrome, IpcEvent::Error {
                message: format!("invalid ipc json: {e}"),
            });
            return Ok(());
        }
    };

    match req {
        IpcRequest::Navigate { url } => {
            let url = normalize_url(&url);
            let mut b = browser.lock().expect("browser lock");
            let url = b.tabs.navigate_active(url);
            content.load_url(&url)?;
            push_tabs(chrome, browser, home_url);
            push_navigation(chrome, browser);
        }
        IpcRequest::NewTab { url } => {
            let mut b = browser.lock().expect("browser lock");
            let url = url.map(|u| normalize_url(&u));
            let _id = b.tabs.new_tab(url);
            let load = b
                .tabs
                .active_url()
                .unwrap_or_else(|| home_url.to_string());
            content.load_url(&load)?;
            push_tabs_and_bookmarks(chrome, browser, home_url);
            push_navigation(chrome, browser);
        }
        IpcRequest::CloseTab { tab_id } => {
            let mut b = browser.lock().expect("browser lock");
            if !b.tabs.close_tab(tab_id)? {
                push_event(
                    chrome,
                    IpcEvent::Error {
                        message: "Cannot close the last tab.".into(),
                    },
                );
                return Ok(());
            }
            if let Some(load) = b.tabs.active_url() {
                content.load_url(&load)?;
            }
            push_tabs(chrome, browser, home_url);
            push_navigation(chrome, browser);
        }
        IpcRequest::SwitchTab { tab_id } => {
            let mut b = browser.lock().expect("browser lock");
            if let Some(url) = b.tabs.switch_tab(tab_id)? {
                content.load_url(&url)?;
                push_tabs(chrome, browser, home_url);
                push_navigation(chrome, browser);
            }
        }
        IpcRequest::SuspendTab { tab_id } => {
            let mut b = browser.lock().expect("browser lock");
            b.tabs.suspend_tab(tab_id)?;
            push_event(chrome, IpcEvent::TabSuspended { tab_id });
            push_tabs(chrome, browser, home_url);
        }
        IpcRequest::GoBack => {
            let _ = content.evaluate_script("window.history.back()");
        }
        IpcRequest::GoForward => {
            let _ = content.evaluate_script("window.history.forward()");
        }
        IpcRequest::Reload => {
            content.reload()?;
        }
        IpcRequest::GoHome => {
            let url = home_url.to_string();
            let mut b = browser.lock().expect("browser lock");
            let url = b.tabs.navigate_active(url);
            content.load_url(&url)?;
            push_tabs(chrome, browser, home_url);
            push_navigation(chrome, browser);
        }
        IpcRequest::SyncBrowser | IpcRequest::ListBookmarks => {
            if let Ok(b) = browser.lock() {
                b.clear_stale_urm_overlay();
            }
            if let Ok(mut b) = browser.lock() {
                b.absorb_ghost_prefetch(&state.config);
            }
            push_tabs_and_bookmarks(chrome, browser, home_url);
            push_navigation(chrome, browser);
            if let Ok(b) = browser.lock() {
                if !b.prefetch.pending().is_empty() {
                    push_event(
                        chrome,
                        IpcEvent::PrefetchQueued {
                            urls: b.prefetch.pending(),
                        },
                    );
                }
            }
        }
        IpcRequest::GetGhostLinkStatus => {
            let b = browser.lock().expect("browser lock");
            let events: Vec<GhostEventPayload> = b
                .ghost
                .recent_events(5)
                .unwrap_or_default()
                .into_iter()
                .map(|e| GhostEventPayload {
                    ts: e.ts,
                    trigger: e.trigger,
                    entropy: e.entropy,
                    summary: e.summary,
                })
                .collect();
            push_event(
                chrome,
                IpcEvent::GhostLinkStatus {
                    active: b.ghost.is_active(),
                    message: b.ghost.status_summary(),
                    events,
                },
            );
        }
        IpcRequest::AddBookmark {
            title,
            url,
            intention,
        } => {
            let mut b = browser.lock().expect("browser lock");
            let url = url
                .map(|u| normalize_url(&u))
                .unwrap_or_else(|| {
                    b.tabs
                        .active_url()
                        .unwrap_or_else(|| home_url.to_string())
                });
            let title = title.unwrap_or_else(|| {
                b.tabs
                    .snapshot()
                    .0
                    .into_iter()
                    .find(|t| t.url == url)
                    .map(|t| t.title)
                    .unwrap_or_else(|| url.clone())
            });
            let intention = intention.unwrap_or_else(|| {
                format!("Intention snapshot: {} ({})", title, url)
            });
            let bm = b.bookmarks.add_local(title, url, intention);
            let truth = state.truth.read().await;
            if let Err(e) = b
                .bookmarks
                .index_in_qdrant(&bm, truth.ollama(), truth.qdrant())
                .await
            {
                tracing::warn!("semantic bookmark index: {e:#}");
            }
            push_bookmarks(chrome, browser);
        }
        IpcRequest::SearchBookmarks { query } => {
            let b = browser.lock().expect("browser lock");
            let truth = state.truth.read().await;
            let hits = b
                .bookmarks
                .search_semantic(&query, truth.ollama(), truth.qdrant(), 12)
                .await
                .unwrap_or_default();
            push_event(
                chrome,
                IpcEvent::SpatialBookmarks {
                    hits: hits
                        .into_iter()
                        .map(|h| SpatialBookmarkPayload {
                            id: h.id,
                            title: h.title,
                            url: h.url,
                            intention: h.intention,
                            score: h.score,
                            proximity: h.proximity,
                        })
                        .collect(),
                },
            );
        }
        IpcRequest::RemoveBookmark { bookmark_id } => {
            let mut b = browser.lock().expect("browser lock");
            b.bookmarks.remove(bookmark_id);
            push_bookmarks(chrome, browser);
        }
        IpcRequest::VibeExtension { name, intent, trigger } => {
            let mut b = browser.lock().expect("browser lock");
            let trigger = parse_trigger(trigger.as_deref());
            match b.extensions.vibe_create(&name, &intent, trigger) {
                Ok(_) => push_extensions(chrome, browser),
                Err(e) => {
                    push_event(chrome, IpcEvent::Error { message: format!("{e:#}") });
                }
            }
        }
        IpcRequest::ListExtensions => {
            push_extensions(chrome, browser);
        }
        IpcRequest::RunExtension { name, action } => {
            let mut b = browser.lock().expect("browser lock");
            let code = action
                .as_deref()
                .map(|a| a.len() as i32)
                .unwrap_or(0);
            match b.extensions.execute(&name, code) {
                Ok(result) => push_event(
                    chrome,
                    IpcEvent::ExtensionRan { name, result },
                ),
                Err(e) => {
                    push_event(chrome, IpcEvent::Error { message: format!("{e:#}") });
                }
            }
        }
        IpcRequest::PrefetchHint { url } => {
            let mut b = browser.lock().expect("browser lock");
            b.prefetch.hint(normalize_url(&url));
            push_event(
                chrome,
                IpcEvent::PrefetchQueued {
                    urls: b.prefetch.pending(),
                },
            );
        }
        IpcRequest::GetStatus => {
            let (ollama, qdrant) = state.truth.read().await.health().await;
            push_event(
                chrome,
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
                        chrome,
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
                        chrome,
                        IpcEvent::Error {
                            message: format!("{e:#}"),
                        },
                    );
                }
            }
        }
        IpcRequest::IngestNotebooks => {
            push_event(
                chrome,
                IpcEvent::IngestProgress {
                    message: "Ingesting notebooks…".into(),
                    done: false,
                },
            );
            let bindings = state.bindings.read().await;
            let count = state
                .truth
                .write()
                .await
                .ingest_notebooks(&bindings)
                .await?;
            push_event(
                chrome,
                IpcEvent::IngestProgress {
                    message: format!("Indexed {count} chunks."),
                    done: true,
                },
            );
        }
        IpcRequest::VerifyText { text } => {
            let result = if text.contains('<') && text.contains('>') {
                truth_guard::verify_content_integrity(state, &text).await?
            } else {
                state.truth.read().await.verify_text(&text).await?
            };
            push_event(
                chrome,
                IpcEvent::VerifyResult(truth_guard::payload_from_result(result)),
            );
        }
        IpcRequest::VerifyActiveTab => {
            push_event(
                chrome,
                IpcEvent::Error {
                    message: "Active-tab capture is planned; paste text in Truth Engine or use Verify on selection.".into(),
                },
            );
        }
        IpcRequest::GetCalibrationConsole => {
            push_calibration_console(chrome, state).await;
        }
        IpcRequest::BindKnowledgeZone => {
            let mut bindings = state.bindings.write().await;
            match pick_and_bind(&mut bindings) {
                Ok(Some(zone)) => {
                    let payload = zone_to_payload(&zone);
                    let zone_id = zone.id.clone();
                    let path = zone.path.clone();
                    let weight = zone.weight;
                    let direct_map = zone.direct_map;
                    drop(bindings);
                    push_event(chrome, IpcEvent::ZoneBound { zone: payload });
                    let count = state
                        .truth
                        .write()
                        .await
                        .ingest_zone(&path, &zone_id, weight, direct_map)
                        .await?;
                    state.bindings.write().await.record_ingest(&zone_id, count)?;
                    push_calibration_console(chrome, state).await;
                    push_event(
                        chrome,
                        IpcEvent::IngestProgress {
                            message: format!("Zone bound — indexed {count} chunks."),
                            done: true,
                        },
                    );
                }
                Ok(None) => {
                    push_event(
                        chrome,
                        IpcEvent::Error {
                            message: "Folder selection cancelled.".into(),
                        },
                    );
                }
                Err(e) => {
                    push_event(chrome, IpcEvent::Error { message: format!("{e:#}") });
                }
            }
        }
        IpcRequest::RemoveZone { zone_id } => {
            let mut bindings = state.bindings.write().await;
            bindings.remove_zone(&zone_id);
            drop(bindings);
            push_calibration_console(chrome, state).await;
        }
        IpcRequest::SetZoneWeight { zone_id, weight } => {
            let mut bindings = state.bindings.write().await;
            bindings.update_weight(&zone_id, weight)?;
            drop(bindings);
            push_calibration_console(chrome, state).await;
        }
        IpcRequest::SetZoneDirectMap { zone_id, direct_map } => {
            let mut bindings = state.bindings.write().await;
            bindings.update_direct_map(&zone_id, direct_map)?;
            drop(bindings);
            push_calibration_console(chrome, state).await;
        }
        IpcRequest::SetDirectMappingGlobal { enabled } => {
            let mut bindings = state.bindings.write().await;
            bindings.set_direct_mapping_global(enabled);
            drop(bindings);
            push_calibration_console(chrome, state).await;
        }
        IpcRequest::SanitizeZone { zone_id } => {
            let extensions = state.config.knowledge.extensions.clone();
            let mut bindings = state.bindings.write().await;
            let (removed, _) = bindings.sanitize_zone(&zone_id, &extensions)?;
            drop(bindings);
            push_calibration_console(chrome, state).await;
            push_event(
                chrome,
                IpcEvent::IngestProgress {
                    message: format!("Sanitized zone — removed {removed} unreadable files."),
                    done: true,
                },
            );
        }
        IpcRequest::GetUrmStatus => {
            push_urm_status(chrome, browser);
        }
        IpcRequest::RestoreUrmSnapshot => {
            let restored = spawn_urm_restore(&state.config);
            push_urm_status(chrome, browser);
            push_event(
                chrome,
                IpcEvent::IngestProgress {
                    message: if restored {
                        "URM snapshot restore signaled.".into()
                    } else {
                        "No URM snapshot to restore.".into()
                    },
                    done: true,
                },
            );
        }
        IpcRequest::DismissUrmOverlay => {
            let path = crate::browser::storage_bridge::urm_browser_overlay();
            if path.is_file() {
                let _ = std::fs::remove_file(path);
            }
        }
        IpcRequest::InjectContext {
            source,
            label,
            text,
            metadata,
        } => {
            let meta = metadata.unwrap_or(serde_json::json!({}));
            crate::vault::enqueue_context(&source, &label, &text, meta)?;
            push_event(
                chrome,
                IpcEvent::ContextInjected { queued: 1 },
            );
        }
        IpcRequest::IngestContextQueue => {
            let packets = crate::vault::ingest_pending(64)?;
            let mut ingested = 0usize;
            for p in &packets {
                if state
                    .truth
                    .read()
                    .await
                    .ingest_context_snippet(&p.source, &p.label, &p.text)
                    .await
                    .is_ok()
                {
                    ingested += 1;
                }
            }
            push_event(
                chrome,
                IpcEvent::IngestProgress {
                    message: format!("Ingested {ingested} context packets from vault queue."),
                    done: true,
                },
            );
        }
        IpcRequest::CreateMemoryAnchor {
            title,
            intention,
            scroll_x,
            scroll_y,
        } => {
            let mut b = browser.lock().expect("browser lock");
            let url = b
                .tabs
                .active_url()
                .unwrap_or_else(|| home_url.to_string());
            let tab_title = b
                .tabs
                .active_title()
                .unwrap_or_else(|| "Tab".into());
            let anchor = b.anchors.create(
                title.unwrap_or_default(),
                url,
                intention.unwrap_or_else(|| "Memory anchor".into()),
                scroll_x.unwrap_or(0.0),
                scroll_y.unwrap_or(0.0),
                tab_title,
            );
            push_memory_anchors(chrome, &b);
            push_event(
                chrome,
                IpcEvent::IngestProgress {
                    message: format!("Memory anchor #{} saved.", anchor.id),
                    done: true,
                },
            );
        }
        IpcRequest::ListMemoryAnchors => {
            let b = browser.lock().expect("browser lock");
            push_memory_anchors(chrome, &b);
        }
        IpcRequest::ClearSafeMode => {
            diagnostics::clear_safe_mode();
            push_event(chrome, IpcEvent::IngestProgress {
                message: "Safe mode cleared — restart Utah Browser for full UI.".into(),
                done: true,
            });
        }
        IpcRequest::SetShellMode { mode } => {
            if layout != ShellLayout::Dual {
                push_event(chrome, IpcEvent::Error {
                    message: "Shell panels require dual webview mode. Restart after safe mode clears.".into(),
                });
                return Ok(());
            }
            let shell_mode = if mode.eq_ignore_ascii_case("app") {
                ShellMode::App
            } else {
                ShellMode::Web
            };
            if let Ok(mut b) = browser.lock() {
                b.shell_mode = shell_mode;
            }
            apply_shell_layout(_window, chrome, content, shell_mode)?;
            push_shell_mode(chrome, shell_mode);
        }
        IpcRequest::IngestZone { zone_id } => {
            let (path, weight, direct_map) = {
                let bindings = state.bindings.read().await;
                let z = bindings
                    .zones()
                    .iter()
                    .find(|z| z.id == zone_id)
                    .ok_or_else(|| anyhow::anyhow!("zone not found"))?;
                (z.path.clone(), z.weight, z.direct_map)
            };
            push_event(
                chrome,
                IpcEvent::IngestProgress {
                    message: format!("Indexing zone {}…", zone_id),
                    done: false,
                },
            );
            let count = state
                .truth
                .write()
                .await
                .ingest_zone(&path, &zone_id, weight, direct_map)
                .await?;
            state.bindings.write().await.record_ingest(&zone_id, count)?;
            push_calibration_console(chrome, state).await;
            push_event(
                chrome,
                IpcEvent::IngestProgress {
                    message: format!("Zone indexed — {count} chunks."),
                    done: true,
                },
            );
        }
    }
    Ok(())
}

fn health_label(h: ZoneHealth) -> &'static str {
    match h {
        ZoneHealth::Healthy => "healthy",
        ZoneHealth::Degraded => "degraded",
        ZoneHealth::Critical => "critical",
        ZoneHealth::Unknown => "unknown",
    }
}

fn zone_to_payload(zone: &crate::binding::KnowledgeZone) -> ZonePayload {
    ZonePayload {
        id: zone.id.clone(),
        path: zone.path.display().to_string(),
        label: zone.label.clone(),
        weight: zone.weight,
        direct_map: zone.direct_map,
        health: health_label(zone.health).to_string(),
        total_files: zone.total_files,
        readable_files: zone.readable_files,
        corrupt_files: zone.corrupt_files,
        indexed_chunks: zone.indexed_chunks,
    }
}

fn push_urm_status(chrome: &WebView, browser: &Arc<Mutex<BrowserUi>>) {
    if let Ok(b) = browser.lock() {
        let active = b.urm.is_active();
        let message = b.urm.status_line();
        let (coherence, status, snapshots, overlay, mutagenesis) =
            match b.urm.read_state() {
                Ok(Some(s)) => {
                    let overlay = b.urm.read_overlay().ok().flatten().map(|o| UrmOverlayPayload {
                        message: o.message,
                        severity: o.severity,
                    });
                    let mutagenesis = b
                        .urm
                        .read_mutagenesis_latest()
                        .ok()
                        .flatten()
                        .map(|m| UrmMutagenesisPayload {
                            summary: m.summary,
                            target_file: m.target_file,
                        });
                    (
                        s.coherence,
                        s.status,
                        s.snapshots,
                        overlay,
                        mutagenesis,
                    )
                }
                _ => (0.0, "OFFLINE".into(), vec![], None, None),
            };
        push_event(
            chrome,
            IpcEvent::UrmStatus {
                active,
                message,
                coherence,
                status,
                overlay,
                mutagenesis,
                snapshots,
            },
        );
        b.clear_stale_urm_overlay();
    }
}

fn spawn_urm_restore(_config: &crate::config::AppConfig) -> bool {
    let repo = crate::paths::install_root();
    let script = repo.join("urm").join("nexus_orchestrator.py");
    if !script.is_file() {
        return crate::browser::storage_bridge::urm_snapshots_dir().join("latest.json").is_file();
    }
    let py = std::env::var("UTAH_PYTHON").unwrap_or_else(|_| "python".into());
    std::process::Command::new(py)
        .arg(&script)
        .arg("--restore")
        .current_dir(repo.join("urm"))
        .env("UTAH_VAULT", crate::browser::storage_bridge::vault_root())
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

async fn push_calibration_console(chrome: &WebView, state: &Arc<AppState>) {
    let bindings = state.bindings.read().await;
    let zones: Vec<ZonePayload> = bindings.zones().iter().map(zone_to_payload).collect();
    let direct_mapping_global = bindings.direct_mapping_global();
    drop(bindings);

    let truth = state.truth.read().await;
    let tele = telemetry::collect(
        &state.config,
        truth.ollama(),
        truth.qdrant(),
        truth.chunks_indexed(),
    )
    .await
    .unwrap_or(telemetry::InferenceTelemetry {
        ollama_online: false,
        qdrant_online: false,
        embed_latency_ms: None,
        vector_points: None,
        vector_dim: state.config.qdrant.vector_size,
        chunks_indexed: truth.chunks_indexed(),
        gpu_note: "Telemetry unavailable".into(),
    });
    drop(truth);

    push_event(
        chrome,
        IpcEvent::CalibrationConsole {
            zones,
            telemetry: TelemetryPayload {
                ollama_online: tele.ollama_online,
                qdrant_online: tele.qdrant_online,
                embed_latency_ms: tele.embed_latency_ms,
                vector_points: tele.vector_points,
                vector_dim: tele.vector_dim,
                chunks_indexed: tele.chunks_indexed,
                gpu_note: tele.gpu_note,
            },
            direct_mapping_global,
        },
    );
}

fn normalize_url(input: &str) -> String {
    let trimmed = input.trim();
    if trimmed.is_empty() {
        return String::new();
    }
    if trimmed.contains("://") {
        trimmed.to_string()
    } else if trimmed.contains('.') && !trimmed.contains(' ') {
        format!("https://{trimmed}")
    } else {
        format!(
            "https://www.google.com/search?q={}",
            urlencoding_light(trimmed)
        )
    }
}

fn urlencoding_light(s: &str) -> String {
    s.chars()
        .map(|c| match c {
            ' ' => "+".to_string(),
            'A'..='Z' | 'a'..='z' | '0'..='9' | '-' | '_' | '.' | '~' => c.to_string(),
            _ => format!("%{:02X}", c as u32),
        })
        .collect()
}

fn push_memory_anchors(chrome: &WebView, browser: &BrowserUi) {
    let anchors: Vec<MemoryAnchorPayload> = browser
        .anchors
        .list()
        .iter()
        .map(|a| MemoryAnchorPayload {
            id: a.id,
            title: a.title.clone(),
            url: a.url.clone(),
            intention: a.intention.clone(),
            scroll_x: a.scroll_x,
            scroll_y: a.scroll_y,
        })
        .collect();
    push_event(chrome, IpcEvent::MemoryAnchorsUpdated { anchors });
}

fn push_tabs(chrome: &WebView, browser: &Arc<Mutex<BrowserUi>>, home_url: &str) {
    if let Ok(b) = browser.lock() {
        let (tabs, active_id) = b.tabs.snapshot();
        push_event(
            chrome,
            IpcEvent::TabsUpdated {
                tabs: tabs
                    .into_iter()
                    .map(|t| TabPayload {
                        id: t.id,
                        title: t.title,
                        url: t.url,
                        suspended: t.suspended,
                    })
                    .collect(),
                active_id,
                home_url: home_url.to_string(),
            },
        );
    }
}

fn push_navigation(chrome: &WebView, browser: &Arc<Mutex<BrowserUi>>) {
    if let Ok(b) = browser.lock() {
        let (tabs, active_id) = b.tabs.snapshot();
        if let Some(tab) = tabs.into_iter().find(|t| t.id == active_id) {
            push_event(
                chrome,
                IpcEvent::NavigationChanged {
                    tab_id: tab.id,
                    url: tab.url,
                    title: tab.title,
                },
            );
        }
    }
}

fn push_bookmarks(chrome: &WebView, browser: &Arc<Mutex<BrowserUi>>) {
    if let Ok(b) = browser.lock() {
        let bookmarks: Vec<BookmarkPayload> = b
            .bookmarks
            .list()
            .iter()
            .map(|bm| BookmarkPayload {
                id: bm.id,
                title: bm.title.clone(),
                url: bm.url.clone(),
                intention: bm.intention.clone(),
                proximity: bm.proximity,
            })
            .collect();
        push_event(chrome, IpcEvent::BookmarksUpdated { bookmarks });
    }
}

fn push_extensions(chrome: &WebView, browser: &Arc<Mutex<BrowserUi>>) {
    if let Ok(b) = browser.lock() {
        if let Ok(list) = b.extensions.list_manifests() {
            let extensions: Vec<ExtensionPayload> = list
                .into_iter()
                .map(|e| ExtensionPayload {
                    name: e.name,
                    trigger: e.trigger,
                    intent: e.intent,
                })
                .collect();
            push_event(chrome, IpcEvent::ExtensionsUpdated { extensions });
        }
    }
}

fn parse_trigger(raw: Option<&str>) -> ExtensionTrigger {
    match raw.unwrap_or("DOM_LOADED") {
        "CLICK" => ExtensionTrigger::Click,
        "NAVIGATION" => ExtensionTrigger::Navigation,
        _ => ExtensionTrigger::DomLoaded,
    }
}

fn push_tabs_and_bookmarks(chrome: &WebView, browser: &Arc<Mutex<BrowserUi>>, home_url: &str) {
    push_tabs(chrome, browser, home_url);
    push_bookmarks(chrome, browser);
}

fn push_event(webview: &WebView, event: IpcEvent) {
    let js = format!(
        "window.utahOnEvent && window.utahOnEvent({});",
        event_json(&event)
    );
    if let Err(e) = webview.evaluate_script(&js) {
        tracing::warn!("push ipc event: {e}");
    }
}
