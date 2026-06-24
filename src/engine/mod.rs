//! Unified compositor shell — one WebView2 (Ghost-Chrome frame + content iframe).

mod compositor;
mod truth_guard;

use crate::browser::{
    prefetch_buffer::{self, PrefetchBuffer},
    prefetch_worker,
    ExtensionRuntime, ExtensionTrigger, PrefetchKernel, SemanticBookmarkStore,
};
use crate::browser::tab_manager::TabManager;
use crate::ghost_link::GhostLinkBridge;
use crate::binding::{pick_and_bind, telemetry, ZoneHealth};
use crate::ipc::{
    event_json, BookmarkPayload, CareerPayload, EmailPayload, ExtensionPayload, GhostEventPayload,
    IpcEvent, IpcRequest, MemoryAnchorPayload, SpatialBookmarkPayload, TabPayload,
    TelemetryPayload, UrmMutagenesisPayload, UrmOverlayPayload, ZonePayload,
};
use crate::browser::MemoryAnchorStore;
use crate::urm::UrmBridge;
use crate::diagnostics;
use crate::AppState;
use anyhow::{Context, Result};
use http::header::CONTENT_TYPE;
use std::borrow::Cow;
use std::collections::HashMap;
use std::path::PathBuf;
use std::sync::{Arc, Mutex, RwLock};
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

pub(crate) const ASSET_SCHEME: &str = "utah";

pub(crate) const TRANSPORT_SCRIPT: &str = r#"
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
    /// Background DNS + memory-buffer warm for a hinted URL.
    PrefetchWarm(String),
    /// Prefetch worker finished — push `PrefetchBuffered` to the shell UI.
    PrefetchDone { url: String, buffer_id: String },
    /// Aggressive State Paging check.
    TabInactivityCheck,
    /// Thread-safe event push from background tasks.
    PushEvent(IpcEvent),
}

type ShellViews = compositor::UnifiedShell;

/// Matches chrome strip height in `browser_frame.html` / mockup.css (logical px).
pub(crate) const CHROME_STRIP_H: f64 = 112.0;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum ShellMode {
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

pub(crate) fn window_logical_size(window: &Window) -> (f64, f64) {
    let size = window.inner_size();
    let scale = window.scale_factor();
    (
        size.width as f64 / scale,
        size.height as f64 / scale,
    )
}

type AssetCache = Arc<RwLock<HashMap<String, (String, Vec<u8>)>>>;

struct BrowserUi {
    tabs: TabManager,
    bookmarks: SemanticBookmarkStore,
    anchors: MemoryAnchorStore,
    extensions: ExtensionRuntime,
    prefetch: PrefetchKernel,
    prefetch_buffer: Arc<Mutex<PrefetchBuffer>>,
    prefetch_http: Arc<reqwest::Client>,
    ghost: GhostLinkBridge,
    p2p_search: crate::browser::p2p_search::SearchNode,
    shield: crate::browser::shield::ShieldEngine,
    urm: UrmBridge,
    shell_mode: ShellMode,
    last_theme: Option<crate::ghost_link::SensoryTheme>,
    asset_cache: AssetCache,
    last_shield_update: std::time::Instant,
}

impl BrowserUi {
    fn new(config: &crate::config::AppConfig) -> Result<Self> {
        let home = config.ui.start_url.clone();
        let mut extensions = ExtensionRuntime::new()?;
        let _ = extensions.load_all();
        let buffer_mb = config.browser.prefetch_buffer_max_mb.max(1) as usize;
        Ok(Self {
            tabs: TabManager::new(home, config.browser.suspend_on_switch)?,
            bookmarks: SemanticBookmarkStore::load(config)?,
            anchors: MemoryAnchorStore::load().unwrap_or_else(|e| {
                diagnostics::log_step(&format!("memory anchors unavailable ({e:#})"));
                MemoryAnchorStore::empty()
            }),
            extensions,
            prefetch: PrefetchKernel::new(config.browser.prefetch_enabled),
            prefetch_buffer: Arc::new(Mutex::new(PrefetchBuffer::new(
                buffer_mb * 1024 * 1024,
            ))),
            prefetch_http: Arc::new(
                reqwest::Client::builder()
                    .timeout(std::time::Duration::from_secs(10))
                    .build()
                    .unwrap_or_else(|_| reqwest::Client::new()),
            ),
            ghost: GhostLinkBridge::new(),
            p2p_search: crate::browser::p2p_search::SearchNode::default(),
            shield: crate::browser::shield::ShieldEngine::new()?,
            urm: UrmBridge::default(),
            shell_mode: ShellMode::Web,
            last_theme: None,
            asset_cache: Arc::new(RwLock::new(HashMap::new())),
            last_shield_update: std::time::Instant::now() - std::time::Duration::from_secs(10),
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
    let safe_mode = recovery.should_use_safe_mode();
    if std::env::var("UTAH_DEMO_MODE").ok().as_deref() == Some("1") {
        // safe_mode = true; // Disabled: Unified compositor (iframe) blocks X-Frame-Options: SAMEORIGIN sites like Google
        diagnostics::log_step("demo mode: dual compositor active (UTAH_DEMO_MODE=1)");
    }
    if safe_mode {
        diagnostics::log_step("recovery safe mode flag set (unified compositor, no dual webview)");
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
    let init_script = compositor::frame_init_script(&home_url);

    let proxy_inactivity = proxy.clone();
    runtime.spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_secs(60)).await;
            let _ = proxy_inactivity.send_event(UserEvent::TabInactivityCheck);
        }
    });

    let initial_mode = ShellMode::Web;
    if let Ok(mut b) = browser.lock() {
        b.shell_mode = initial_mode;
    }

    let (prefetch_buffer, asset_cache) = {
        let b = browser.lock().map_err(|_| anyhow::anyhow!("browser lock poisoned"))?;
        (b.prefetch_buffer.clone(), b.asset_cache.clone())
    };

    let shell = if compositor::legacy_dual_enabled() || !safe_mode {
        if compositor::legacy_dual_enabled() {
            diagnostics::log_step("legacy dual webview (UTAH_LEGACY_DUAL=1)");
        } else {
            diagnostics::log_step("dual webview compositor active (default)");
        }
        boot_legacy_dual(
            &window,
            &assets_root,
            &chrome_init_script(&home_url),
            &prefetch_buffer,
            &asset_cache,
            &proxy_nav,
            &proxy_ipc,
            &proxy_page,
            &proxy_title,
            initial_mode,
        )?
    } else {
        compositor::boot(
            &window,
            &assets_root,
            &init_script,
            &prefetch_buffer,
            &asset_cache,
            &proxy_nav,
            &proxy_ipc,
            &proxy_page,
            &proxy_title,
            initial_mode,
        )?
    };

    let mode_label = match shell.layout {
        compositor::CompositorLayout::Unified => "unified",
        compositor::CompositorLayout::LegacyDual => "legacy-dual",
    };
    diagnostics::log_step(&format!("shell ready ({mode_label})"));
    crate::sentinel::signal_shell_ready(mode_label);

    let proxy_sensory = proxy.clone();
    runtime.spawn(async move {
        let mut last_mtime = None;
        let mut interval = std::time::Duration::from_secs(2);
        let theme_path = crate::browser::storage_bridge::ghost_link_theme();

        loop {
            tokio::time::sleep(interval).await;
            
            let current_mtime = std::fs::metadata(&theme_path)
                .and_then(|m| m.modified())
                .ok();

            if current_mtime != last_mtime {
                last_mtime = current_mtime;
                let _ = proxy_sensory.send_event(UserEvent::SensoryPoll);
                // Reset to fast polling if something changed
                interval = std::time::Duration::from_secs(2);
            } else {
                // Adaptive slowdown if no changes detected
                if interval < std::time::Duration::from_secs(10) {
                    interval += std::time::Duration::from_secs(1);
                }
            }
        }
    });

    let start_url = {
        let b = browser.lock().map_err(|_| anyhow::anyhow!("browser lock poisoned"))?;
        b.tabs
            .active_url()
            .unwrap_or_else(|| home_url.clone())
    };

    let ui = shell.ui();
    if shell.layout == compositor::CompositorLayout::Unified {
        if let Err(e) = compositor::set_content_url(ui, &start_url) {
            diagnostics::log_step(&format!("content frame load warn: {e:#}"));
        }
        if let Ok(mut b) = browser.lock() {
            push_tabs_and_bookmarks(ui, &b, &home_url);
            push_navigation(ui, &b);
            fire_extensions(&mut b, ui, ExtensionTrigger::DomLoaded);
        }
        let _ = ui.evaluate_script(
            "if(window.utahOnFrameReady)window.utahOnFrameReady();",
        );
    } else if let Some(chrome) = shell.chrome.as_ref() {
        if let Err(e) = apply_shell_layout(&window, chrome, shell.content(), initial_mode) {
            diagnostics::log_step(&format!("layout warn: {e:#}"));
        }
        if let Ok(b) = browser.lock() {
            push_tabs_and_bookmarks(chrome, &b, &home_url);
            push_navigation(chrome, &b);
        }
        push_shell_mode(chrome, initial_mode);
    }

    if let Ok(b) = browser.lock() {
        b.clear_stale_urm_overlay();
    }

    let _ = proxy.send_event(UserEvent::DeferredLoad(start_url));

    let proxy_drain = proxy.clone();
    let browser_drain = browser.clone();
    let prefetch_enabled = state.config.browser.prefetch_enabled;
    runtime.spawn(async move {
        loop {
            tokio::time::sleep(std::time::Duration::from_millis(1500)).await;
            if !prefetch_enabled {
                continue;
            }
            if let Ok(mut b) = browser_drain.lock() {
                while let Some(url) = b.prefetch.pop() {
                    schedule_prefetch_warm(&proxy_drain, url, true);
                }
            }
        }
    });

    let state_loop = state.clone();
    let rt = runtime;
    let browser_loop = browser.clone();

    event_loop.run(move |event, _, control_flow| {
        *control_flow = ControlFlow::Wait;

        match &event {
            Event::UserEvent(UserEvent::Ipc(body)) => {
                if let Err(e) = rt.block_on(handle_ipc(
                    body,
                    &state_loop,
                    &window,
                    &shell,
                    &browser_loop,
                    &home_url,
                    &proxy,
                    &rt,
                )) {
                    diagnostics::log_step(&format!("ipc error: {e:#}"));
                    push_event(shell.ui(), IpcEvent::Error {
                        message: format!("{e:#}"),
                    });
                }
            }
            Event::UserEvent(UserEvent::DeferredLoad(url)) => {
                diagnostics::log_step(&format!("loading {url}"));
                if let Err(e) = shell_navigate(&shell, url) {
                    diagnostics::log_step(&format!("navigate failed: {e:#}"));
                }
            }
            Event::UserEvent(UserEvent::SensoryPoll) => {
                if let Ok(mut b) = browser_loop.lock() {
                    if let Ok(Some(theme)) = b.ghost.read_theme() {
                        if b.last_theme.as_ref() != Some(&theme) {
                            b.last_theme = Some(theme.clone());
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
            }
            Event::UserEvent(UserEvent::PrefetchWarm(url)) => {
                let url = url.clone();
                let state_w = state_loop.clone();
                let browser_w = browser_loop.clone();
                let proxy_w = proxy.clone();
                rt.spawn(async move {
                    let (client, buffer, enabled) = {
                        let Ok(b) = browser_w.lock() else {
                            return;
                        };
                        (
                            b.prefetch_http.clone(),
                            b.prefetch_buffer.clone(),
                            state_w.config.browser.prefetch_enabled,
                        )
                    };
                    if !enabled {
                        return;
                    }
                    match prefetch_worker::warm_url(&client, buffer, &url).await {
                        Ok(id) => {
                            let _ = proxy_w.send_event(UserEvent::PrefetchDone {
                                url,
                                buffer_id: id,
                            });
                        }
                        Err(e) => tracing::debug!("prefetch warm skipped: {e:#}"),
                    }
                });
            }
            Event::UserEvent(UserEvent::PrefetchDone { url, buffer_id }) => {
                let url = url.clone();
                let buffer_id = buffer_id.clone();
                let buffer_uri = PrefetchBuffer::buffer_uri(&buffer_id);
                let bytes = browser_loop
                    .lock()
                    .ok()
                    .and_then(|b| {
                        b.prefetch_buffer
                            .lock()
                            .ok()
                            .and_then(|buf| buf.get(&buffer_id).map(|e| e.body.len()))
                    })
                    .unwrap_or(0);
                push_event(
                    shell.ui(),
                    IpcEvent::PrefetchBuffered {
                        url,
                        buffer_id,
                        buffer_uri,
                        bytes,
                    },
                );
            }
            Event::UserEvent(UserEvent::TabInactivityCheck) => {
                if let Ok(mut b) = browser_loop.lock() {
                    let inactive = b.tabs.get_inactive_tabs(300); // 5 minutes
                    let mut changed = false;
                    for id in inactive {
                        if let Err(e) = b.tabs.suspend_tab(id) {
                            tracing::error!("Aggressive State Paging: failed to suspend tab {id}: {e:#}");
                        } else {
                            changed = true;
                        }
                    }
                    if changed {
                        push_tabs(shell.ui(), &b, &home_url);
                    }
                }
            }
            Event::UserEvent(UserEvent::PushEvent(ev)) => {
                push_event(shell.ui(), ev.clone());
            }
            Event::UserEvent(UserEvent::PageUrl(url)) => {
                if let Ok(mut b) = browser_loop.lock() {
                    b.tabs.set_active_url(url.clone());
                    push_navigation(shell.ui(), &b);
                    push_active_tab_metadata(shell.ui(), &b);
                    fire_extensions(&mut b, shell.ui(), ExtensionTrigger::Navigation);
                }
            }
            Event::UserEvent(UserEvent::PageTitle(title)) => {
                if let Ok(mut b) = browser_loop.lock() {
                    b.tabs.set_active_title(title.clone());
                    push_active_tab_metadata(shell.ui(), &b);
                }
            }
            Event::WindowEvent {
                event: WindowEvent::Resized(_),
                ..
            } => {
                match shell.layout {
                    compositor::CompositorLayout::Unified => {
                        let (w, h) = window_logical_size(&window);
                        let _ = shell.view.set_bounds(Rect {
                            position: LogicalPosition::new(0.0, 0.0).into(),
                            size: LogicalSize::new(w.max(1.0), h.max(1.0)).into(),
                        });
                    }
                    compositor::CompositorLayout::LegacyDual => {
                        if let Some(chrome) = &shell.chrome {
                            if let Ok(b) = browser_loop.lock() {
                                let _ = apply_shell_layout(
                                    &window,
                                    chrome,
                                    shell.content(),
                                    b.shell_mode,
                                );
                            }
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
    })
}

fn boot_legacy_dual(
    window: &Window,
    assets_root: &PathBuf,
    init_script: &str,
    prefetch_buffer: &Arc<Mutex<PrefetchBuffer>>,
    asset_cache: &AssetCache,
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
    let buffer = prefetch_buffer.clone();
    let cache = asset_cache.clone();
    let chrome = WebViewBuilder::new()
        .with_custom_protocol(ASSET_SCHEME.into(), {
            let proxy = proxy_nav.clone();
            let assets = assets.clone();
            let buffer = buffer.clone();
            let cache = cache.clone();
            move |_id, request| protocol_response(&assets, &buffer, &cache, &request, &proxy).into()
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
        layout: compositor::CompositorLayout::LegacyDual,
        view: content,
        chrome: Some(chrome),
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

fn shell_navigate(shell: &ShellViews, url: &str) -> Result<()> {
    match shell.layout {
        compositor::CompositorLayout::Unified => compositor::set_content_url(shell.ui(), url),
        compositor::CompositorLayout::LegacyDual => shell.content().load_url(url).map_err(Into::into),
    }
}

fn shell_back(shell: &ShellViews) {
    match shell.layout {
        compositor::CompositorLayout::Unified => compositor::content_back(shell.ui()),
        compositor::CompositorLayout::LegacyDual => {
            let _ = shell.content().evaluate_script("window.history.back()");
        }
    }
}

fn shell_forward(shell: &ShellViews) {
    match shell.layout {
        compositor::CompositorLayout::Unified => compositor::content_forward(shell.ui()),
        compositor::CompositorLayout::LegacyDual => {
            let _ = shell
                .content()
                .evaluate_script("window.history.forward()");
        }
    }
}

fn shell_reload(shell: &ShellViews) -> Result<()> {
    match shell.layout {
        compositor::CompositorLayout::Unified => {
            compositor::content_reload(shell.ui());
            Ok(())
        }
        compositor::CompositorLayout::LegacyDual => shell.content().reload().map_err(Into::into),
    }
}

pub(crate) fn protocol_response(
    assets: &PathBuf,
    buffer: &Arc<Mutex<PrefetchBuffer>>,
    cache: &AssetCache,
    request: &Request<Vec<u8>>,
    proxy: &EventLoopProxy<UserEvent>,
) -> Response<Cow<'static, [u8]>> {
    if let Some(resp) = try_handle_navigate_route(request, proxy) {
        return match resp {
            Ok(r) => r.map(|b| Cow::from(b)),
            Err(e) => Response::builder()
                .header(CONTENT_TYPE, "text/plain")
                .status(500)
                .body(Cow::from(e.to_string().into_bytes()))
                .unwrap(),
        };
    }
    let path = request.uri().path();
    if let Ok(buf) = buffer.lock() {
        if let Ok(Some((mime, body))) = prefetch_buffer::try_serve_buffer(&buf, path) {
            return Response::builder()
                .header(CONTENT_TYPE, mime)
                .header("Cache-Control", "private, max-age=3600")
                .header("X-Utah-Buffer", "1")
                .body(Cow::from(body))
                .unwrap();
        }
    }
    match serve_asset(assets, cache, request.clone()) {
        Ok(r) => r.map(|b| Cow::from(b)),
        Err(e) => Response::builder()
            .header(CONTENT_TYPE, "text/plain")
            .status(500)
            .body(Cow::from(e.to_string().into_bytes()))
            .unwrap(),
    }
}

pub(crate) fn try_handle_navigate_route(
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

pub(crate) fn serve_asset(
    root: &PathBuf,
    cache: &AssetCache,
    request: Request<Vec<u8>>,
) -> Result<Response<Vec<u8>>> {
    let path = request.uri().path();
    if let Ok(c) = cache.read() {
        if let Some((mime, body)) = c.get(path) {
            return Ok(Response::builder()
                .header(CONTENT_TYPE, mime)
                .header("X-Utah-Cache", "HIT")
                .body(body.clone())
                .unwrap());
        }
    }

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

    if let Ok(mut c) = cache.write() {
        c.insert(path.to_string(), (mime.clone(), content.clone()));
    }

    Ok(Response::builder()
        .header(CONTENT_TYPE, mime)
        .header("X-Utah-Cache", "MISS")
        .body(content)
        .unwrap())
}

fn schedule_prefetch_warm(
    proxy: &EventLoopProxy<UserEvent>,
    url: String,
    enabled: bool,
) {
    if enabled && !url.is_empty() {
        let _ = proxy.send_event(UserEvent::PrefetchWarm(url));
    }
}

fn fire_extensions(
    browser: &mut BrowserUi,
    chrome: &WebView,
    trigger: ExtensionTrigger,
) {
    let rs = browser.extensions.dispatch(trigger);
    let mut js_scripts = Vec::new();
    for (name, _) in &rs {
        if let Some(code) = browser.extensions.get_js_code(name) {
            js_scripts.push(code);
        }
    }

    for (name, result) in rs {
        match result {
            Ok(code) => push_event(chrome, IpcEvent::ExtensionRan { name, result: code }),
            Err(e) => push_event(chrome, IpcEvent::Error { message: format!("{e:#}") }),
        }
    }
    for js in js_scripts {
        let _ = chrome.evaluate_script(&js);
    }
}

async fn run_python(script: &str, args: Vec<&str>) -> Result<String> {
    let repo = crate::paths::install_root();
    let script_path = repo.join(script);
    let py = std::env::var("UTAH_PYTHON").unwrap_or_else(|_| "python".into());

    let output = tokio::process::Command::new(py)
        .arg(&script_path)
        .args(args)
        .current_dir(repo)
        .output()
        .await
        .context("spawn python daemon")?;

    if !output.status.success() {
        let err = String::from_utf8_lossy(&output.stderr);
        anyhow::bail!("Python daemon failed: {err}");
    }

    Ok(String::from_utf8_lossy(&output.stdout).to_string())
}

async fn handle_ipc(
    body: &str,
    state: &Arc<AppState>,
    window: &Window,
    shell: &ShellViews,
    browser: &Arc<Mutex<BrowserUi>>,
    home_url: &str,
    proxy: &EventLoopProxy<UserEvent>,
    _rt: &tokio::runtime::Runtime,
) -> Result<()> {
    let chrome = shell.ui();
    let layout = shell.layout;
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
            let normalized = normalize_url(&url);

            // Sovereign Secure Shield Check
            {
                let mut b = browser.lock().expect("browser lock");
                let (is_blocked, rule, category) = b.shield.inspect_url(&normalized);
                if is_blocked {
                    b.shield.log_block(normalized.clone(), rule.clone(), category.clone());
                    
                    // Throttle shield updates in navigation too
                    if b.last_shield_update.elapsed().as_millis() > 500 {
                        b.last_shield_update = std::time::Instant::now();
                        push_event(chrome, IpcEvent::ShieldUpdated { metrics: b.shield.get_metrics() });
                    }
                    
                    push_event(chrome, IpcEvent::Error {
                        message: format!("Shield Blocked Threat: {} ({})", rule, category),
                    });
                    if category == "Malware Risk" || category == "Threat Vector" || category == "Scam" {
                        tracing::warn!("[UTAH_SHIELD] Dropped high-risk navigation: {}", normalized);
                        return Ok(());
                    }
                }
            }

            // Phase 4: SOTA Onion Routing
            if normalized.contains(".onion") {
                tracing::info!("[UTAH_TOR] Routing through SOTA Onion Tunnel...");
                push_event(chrome, IpcEvent::Error {
                    message: "Routing through Tor proxy (localhost:9050)...".into()
                });
            }

            if normalized.starts_with("http") || normalized.starts_with("utah") || normalized.starts_with("file") {
                let mut b = browser.lock().expect("browser lock");
                let url = b.tabs.navigate_active(normalized);
                shell_navigate(shell, &url)?;
                push_active_tab_metadata(chrome, &b);
                push_navigation(chrome, &b);
                fire_extensions(&mut b, chrome, ExtensionTrigger::Navigation);
                schedule_prefetch_warm(proxy, url, state.config.browser.prefetch_enabled);
            } else {
                // Phase 3: SOTA In-App Search (Local -> P2P) — Offload to background to prevent UI freeze
                let state = state.clone();
                let proxy = proxy.clone();
                let browser = browser.clone();
                let url = url.to_string();
                
                tokio::spawn(async move {
                    let p2p_search = {
                        let Ok(b) = browser.lock() else { return; };
                        b.p2p_search.clone()
                    };
                    
                    let truth = state.truth.read().await;
                    let (answer, sources) = truth.ask_question(&url).await.unwrap_or_default();
                    let p2p_hits = p2p_search.query(&url).await.unwrap_or_default();
                    
                    if !answer.to_lowercase().contains("don't know") && !answer.is_empty() {
                        let _ = proxy.send_event(UserEvent::PushEvent(IpcEvent::ChatResponse { answer, sources }));
                        let _ = proxy.send_event(UserEvent::Ipc(r#"{"cmd":"set_shell_mode","mode":"app"}"#.into())); // Placeholder to switch view
                    } else if let Some(best) = p2p_hits.first() {
                        let _ = proxy.send_event(UserEvent::DeferredLoad(best.url.clone()));
                    } else {
                        let search_url = format!("https://duckduckgo.com/?q={}", urlencoding_light(&url));
                        let _ = proxy.send_event(UserEvent::DeferredLoad(search_url));
                    }
                });
            }
        }
        IpcRequest::NewTab { url } => {
            let mut b = browser.lock().expect("browser lock");
            let url = url.map(|u| normalize_url(&u));
            let _id = b.tabs.new_tab(url);
            let load = b
                .tabs
                .active_url()
                .unwrap_or_else(|| home_url.to_string());
            shell_navigate(shell, &load)?;
            push_tabs_and_bookmarks(chrome, &b, home_url);
            push_navigation(chrome, &b);
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
                shell_navigate(shell, &load)?;
            }
            push_tabs(chrome, &b, home_url);
            push_navigation(chrome, &b);
        }
        IpcRequest::SwitchTab { tab_id } => {
            // Aggressive State Paging: capture current tab before switching
            {
                let b = browser.lock().expect("browser lock");
                let active_id = b.tabs.active_id();
                if active_id != tab_id {
                    let _ = shell.ui().evaluate_script(&format!(
                        "if(window.utahCaptureTabState) window.utahCaptureTabState({active_id});"
                    ));
                }
            }
            let mut b = browser.lock().expect("browser lock");
            if let Some(url) = b.tabs.switch_tab(tab_id)? {
                shell_navigate(shell, &url)?;
                push_active_tab_changed(chrome, &b);
                push_navigation(chrome, &b);
            }
        }
        IpcRequest::SuspendTab { tab_id } => {
            let mut b = browser.lock().expect("browser lock");
            b.tabs.suspend_tab(tab_id)?;
            push_event(chrome, IpcEvent::TabSuspended { tab_id });
            push_tab_metadata(chrome, &b, tab_id);
        }
        IpcRequest::SaveTabState {
            tab_id,
            scroll_x,
            scroll_y,
            dom_snapshot,
        } => {
            let mut b = browser.lock().expect("browser lock");
            b.tabs.set_scroll(tab_id, scroll_x, scroll_y);
            if let Some(dom) = dom_snapshot {
                if let Some(state) = b.tabs.get_active_mut(tab_id) {
                    state.dom_snapshot = dom.into_bytes();
                }
            }
        }
        IpcRequest::GoBack => shell_back(shell),
        IpcRequest::GoForward => shell_forward(shell),
        IpcRequest::Reload => shell_reload(shell)?,
        IpcRequest::GoHome => {
            let url = home_url.to_string();
            let mut b = browser.lock().expect("browser lock");
            let url = b.tabs.navigate_active(url);
            shell_navigate(shell, &url)?;
            push_active_tab_metadata(chrome, &b);
            push_navigation(chrome, &b);
        }
        IpcRequest::SyncBrowser | IpcRequest::ListBookmarks => {
            let mut b = browser.lock().expect("browser lock");
            b.clear_stale_urm_overlay();
            b.absorb_ghost_prefetch(&state.config);
            push_tabs_and_bookmarks(chrome, &b, home_url);
            push_navigation(chrome, &b);
            if !b.prefetch.pending().is_empty() {
                push_event(
                    chrome,
                    IpcEvent::PrefetchQueued {
                        urls: b.prefetch.pending(),
                    },
                );
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
            let b = browser.lock().expect("browser lock");
            let url = url
                .map(|u| normalize_url(&u))
                .unwrap_or_else(|| {
                    b.tabs
                        .active_url()
                        .unwrap_or_else(|| home_url.to_string())
                });
            let title = title.unwrap_or_else(|| {
                b.tabs
                    .get_title_for_url(&url)
                    .unwrap_or_else(|| url.clone())
            });
            let intention = intention.unwrap_or_else(|| {
                format!("Intention snapshot: {} ({})", title, url)
            });
            let bm = b.bookmarks.add_local(title, url, intention);
            
            // Offload indexing to background
            let state = state.clone();
            let bookmarks = b.bookmarks.clone();
            tokio::spawn(async move {
                let truth = state.truth.read().await;
                if let Err(e) = bookmarks
                    .index_in_qdrant(&bm, truth.ollama(), truth.qdrant())
                    .await
                {
                    tracing::warn!("semantic bookmark index: {e:#}");
                }
            });
            push_bookmarks(chrome, &b);
        }
        IpcRequest::SearchBookmarks { query } => {
            let state = state.clone();
            let proxy = proxy.clone();
            let browser = browser.clone();
            tokio::spawn(async move {
                let b = {
                    let Ok(lock) = browser.lock() else { return; };
                    lock.bookmarks.clone()
                };
                let truth = state.truth.read().await;
                if let Ok(hits) = b.search_semantic(&query, truth.ollama(), truth.qdrant(), 12).await {
                    let _ = proxy.send_event(UserEvent::PushEvent(IpcEvent::SpatialBookmarks {
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
                    }));
                }
            });
        }
        IpcRequest::RemoveBookmark { bookmark_id } => {
            let b = browser.lock().expect("browser lock");
            b.bookmarks.remove(bookmark_id);
            push_bookmarks(chrome, &b);
        }
        IpcRequest::VibeExtension { name, intent, trigger } => {
            let trigger = parse_trigger(trigger.as_deref());
            let truth = state.truth.read().await;
            let mut b = browser.lock().expect("browser lock");
            // Keep it synchronous for now to avoid Clone issues with ExtensionRuntime
            // This is a rare operation and unlikely to cause general browser freezes
            b.extensions.vibe_create(&name, &intent, trigger, truth.ollama()).await?;
            push_extensions(chrome, &b);
        }
        IpcRequest::ListExtensions => {
            let b = browser.lock().expect("browser lock");
            push_extensions(chrome, &b);
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
            let url = normalize_url(&url);
            let enabled = state.config.browser.prefetch_enabled;
            let mut b = browser.lock().expect("browser lock");
            b.prefetch.hint(url.clone());
            push_event(
                chrome,
                IpcEvent::PrefetchQueued {
                    urls: b.prefetch.pending(),
                },
            );
            schedule_prefetch_warm(proxy, url, enabled);
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
        IpcRequest::ListEmails => {
            let proxy = proxy.clone();
            tokio::spawn(async move {
                if let Ok(output) = run_python("flux/email_nexus.py", vec!["--list"]).await {
                    let emails: Vec<EmailPayload> = serde_json::from_str(&output).unwrap_or_default();
                    let _ = proxy.send_event(UserEvent::PushEvent(IpcEvent::EmailsUpdated { emails }));
                }
            });
        }
        IpcRequest::FetchEmailDetail { id } => {
            let proxy = proxy.clone();
            tokio::spawn(async move {
                if let Ok(output) = run_python("flux/email_nexus.py", vec!["--fetch", &id]).await {
                    let detail: serde_json::Value = serde_json::from_str(&output).unwrap_or_default();
                    let body = detail.get("body").and_then(|v| v.as_str()).unwrap_or("No content").to_string();
                    let _ = proxy.send_event(UserEvent::PushEvent(IpcEvent::EmailDetail { body }));
                }
            });
        }
        IpcRequest::RefactorResume { jd } => {
            let proxy = proxy.clone();
            tokio::spawn(async move {
                if let Ok(output) = run_python("flux/career_forge.py", vec!["--refactor", &jd]).await {
                    let _ = proxy.send_event(UserEvent::PushEvent(IpcEvent::ResumeRefactored { tailored_resume: output }));
                }
            });
        }
        IpcRequest::GetCareerHistory => {
            let proxy = proxy.clone();
            tokio::spawn(async move {
                if let Ok(output) = run_python("flux/career_forge.py", vec!["--history"]).await {
                    let history: Vec<CareerPayload> = serde_json::from_str(&output).unwrap_or_default();
                    let _ = proxy.send_event(UserEvent::PushEvent(IpcEvent::CareerHistory { history }));
                }
            });
        }
        IpcRequest::SubmitApplication { company, title } => {
            let proxy = proxy.clone();
            tokio::spawn(async move {
                let _ = run_python("flux/career_forge.py", vec!["--submit", "--company", &company, "--title", &title]).await;
                if let Ok(output) = run_python("flux/career_forge.py", vec!["--history"]).await {
                    let history: Vec<CareerPayload> = serde_json::from_str(&output).unwrap_or_default();
                    let _ = proxy.send_event(UserEvent::PushEvent(IpcEvent::CareerHistory { history }));
                }
            });
        }
        IpcRequest::ExecutePersonaSwap { target, source } => {
            let proxy = proxy.clone();
            tokio::spawn(async move {
                if let Ok(output) = run_python("flux/persona_engine/guardian.py", vec!["--swap", "--target", &target, "--source", &source]).await {
                    let res: serde_json::Value = serde_json::from_str(&output).unwrap_or_default();
                    let status = res.get("status").and_then(|v| v.as_str()).unwrap_or("ERROR").to_string();
                    let output_path = res.get("output_path").and_then(|v| v.as_str()).unwrap_or("").to_string();
                    let _ = proxy.send_event(UserEvent::PushEvent(IpcEvent::PersonaSwapResult { status, output_path }));
                }
            });
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
            let state = state.clone();
            let proxy = proxy.clone();
            tokio::spawn(async move {
                let _ = proxy.send_event(UserEvent::PushEvent(IpcEvent::IngestProgress {
                    message: "Ingesting notebooks…".into(),
                    done: false,
                }));
                let bindings = state.bindings.read().await;
                if let Ok(count) = state.truth.write().await.ingest_notebooks(&bindings).await {
                    let _ = proxy.send_event(UserEvent::PushEvent(IpcEvent::IngestProgress {
                        message: format!("Indexed {count} chunks."),
                        done: true,
                    }));
                }
            });
        }
        IpcRequest::VerifyText { text } => {
            let state = state.clone();
            let proxy = proxy.clone();
            tokio::spawn(async move {
                let result = if text.contains('<') && text.contains('>') {
                    truth_guard::verify_content_integrity(&state, &text).await
                } else {
                    state.truth.read().await.verify_text(&text).await
                };
                if let Ok(res) = result {
                    let _ = proxy.send_event(UserEvent::PushEvent(IpcEvent::VerifyResult(truth_guard::payload_from_result(res))));
                }
            });
        }
        IpcRequest::ChatQuery { query } => {
            let state = state.clone();
            let proxy = proxy.clone();
            tokio::spawn(async move {
                let truth = state.truth.read().await;
                if let Ok((answer, sources)) = truth.ask_question(&query).await {
                    let _ = proxy.send_event(UserEvent::PushEvent(IpcEvent::ChatResponse { answer, sources }));
                }
            });
        }
        IpcRequest::QuantumQuery { problem_key, sync } => {
            let resp = state.quantum.query(&problem_key);
            if sync {
                tracing::info!("[UTAH_QUANTUM] Reifying logic path: {}", problem_key);
            }
            push_event(
                chrome,
                IpcEvent::QuantumOracleResponse {
                    problem_key,
                    logic_payload: resp.logic_payload,
                    status: resp.status,
                    verification_checksum: resp.verification_checksum,
                },
            );
        }
        IpcRequest::GetQuantumState => {
            let s = state.quantum.get_state();
            push_event(
                chrome,
                IpcEvent::QuantumStateUpdated {
                    state: crate::ipc::QuantumStatePayload {
                        entropy: s.entropy,
                        timeline: s.timeline,
                        anchor_stable: s.anchor_stable,
                    },
                },
            );
        }
        IpcRequest::GetShieldMetrics => {
            let b = browser.lock().expect("browser lock");
            push_event(chrome, IpcEvent::ShieldUpdated { metrics: b.shield.get_metrics() });
        }
        IpcRequest::ReportShieldBlock { url, category } => {
            let mut b = browser.lock().expect("browser lock");
            b.shield.log_block(url, "Geometric Heuristic".into(), category);
            // Throttle UI updates to 2Hz to prevent IPC congestion
            if b.last_shield_update.elapsed().as_millis() > 500 {
                b.last_shield_update = std::time::Instant::now();
                push_event(chrome, IpcEvent::ShieldUpdated { metrics: b.shield.get_metrics() });
            }
        }
        IpcRequest::GhostLinkIncognito { enabled } => {
            if enabled {
                tracing::info!("[UTAH_VOID] Entering Absolute Void-State...");
                let void_dir = crate::paths::sovereign_data_root().join("void_vault");
                let _ = std::fs::create_dir_all(&void_dir);
            } else {
                tracing::info!("[UTAH_VOID] Exiting Void-State. Wiping volatile data...");
                let void_dir = crate::paths::sovereign_data_root().join("void_vault");
                let _ = crate::paths::zero_fill_dir(&void_dir);
            }
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
            let shell_mode = if mode.eq_ignore_ascii_case("app") {
                ShellMode::App
            } else {
                ShellMode::Web
            };
            if let Ok(mut b) = browser.lock() {
                b.shell_mode = shell_mode;
            }
            match layout {
                compositor::CompositorLayout::Unified => {
                    let active = browser
                        .lock()
                        .ok()
                        .and_then(|b| b.tabs.active_url());
                    compositor::apply_shell_mode(
                        chrome,
                        shell_mode,
                        active.as_deref(),
                    )?;
                }
                compositor::CompositorLayout::LegacyDual => {
                    let content = shell.content();
                    apply_shell_layout(window, chrome, content, shell_mode)?;
                    push_shell_mode(chrome, shell_mode);
                }
            }
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
            "https://duckduckgo.com/?q={}",
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

fn push_tabs(chrome: &WebView, browser: &BrowserUi, home_url: &str) {
    let (tabs, active_id) = browser.tabs.snapshot();
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

fn push_active_tab_metadata(chrome: &WebView, browser: &BrowserUi) {
    push_tab_metadata(chrome, browser, browser.tabs.active_id());
}

fn push_tab_metadata(chrome: &WebView, browser: &BrowserUi, tab_id: u32) {
    let (tabs, _) = browser.tabs.snapshot();
    if let Some(tab) = tabs.into_iter().find(|t| t.id == tab_id) {
        push_event(
            chrome,
            IpcEvent::TabMetadataUpdated {
                tab: TabPayload {
                    id: tab.id,
                    title: tab.title,
                    url: tab.url,
                    suspended: tab.suspended,
                },
            },
        );
    }
}

fn push_active_tab_changed(chrome: &WebView, browser: &BrowserUi) {
    push_event(
        chrome,
        IpcEvent::ActiveTabChanged {
            active_id: browser.tabs.active_id(),
        },
    );
}

fn push_navigation(chrome: &WebView, browser: &BrowserUi) {
    let (tabs, active_id) = browser.tabs.snapshot();
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

fn push_bookmarks(chrome: &WebView, browser: &BrowserUi) {
    let bookmarks: Vec<BookmarkPayload> = browser
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

fn push_extensions(chrome: &WebView, browser: &BrowserUi) {
    if let Ok(list) = browser.extensions.list_manifests() {
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

fn parse_trigger(raw: Option<&str>) -> ExtensionTrigger {
    match raw.unwrap_or("DOM_LOADED") {
        "CLICK" => ExtensionTrigger::Click,
        "NAVIGATION" => ExtensionTrigger::Navigation,
        _ => ExtensionTrigger::DomLoaded,
    }
}

fn push_tabs_and_bookmarks(chrome: &WebView, browser: &BrowserUi, home_url: &str) {
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
