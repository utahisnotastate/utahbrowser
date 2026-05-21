//! Unified compositor — single WebView2 instance (chrome HTML + content iframe).

use super::{
    protocol_response, ShellMode, UserEvent, ASSET_SCHEME, TRANSPORT_SCRIPT,
};
use crate::browser::PrefetchBuffer;
use anyhow::{Context, Result};
use std::path::PathBuf;
use std::sync::{Arc, Mutex};
use tao::{
    dpi::{LogicalPosition, LogicalSize},
    event_loop::EventLoopProxy,
    window::Window,
};
use wry::{PageLoadEvent, Rect, WebView, WebViewBuilder};

pub const FRAME_PATH: &str = "browser_frame.html";
pub const DASHBOARD_PATH: &str = "index.html";

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CompositorLayout {
    /// Ghost-Chrome frame + iframe for the open web.
    Unified,
    /// Legacy dual HWND webviews (opt-in).
    LegacyDual,
}

pub struct UnifiedShell {
    pub layout: CompositorLayout,
    /// Unified: sole webview. Legacy dual: content pane.
    pub view: WebView,
    /// Legacy dual only — chrome strip webview.
    pub chrome: Option<WebView>,
}

impl UnifiedShell {
    pub fn ui(&self) -> &WebView {
        self.chrome.as_ref().unwrap_or(&self.view)
    }

    pub fn content(&self) -> &WebView {
        &self.view
    }
}

pub fn legacy_dual_enabled() -> bool {
    std::env::var("UTAH_LEGACY_DUAL")
        .ok()
        .map(|v| v == "1" || v.eq_ignore_ascii_case("true"))
        .unwrap_or(false)
}

pub fn boot(
    window: &Window,
    assets_root: &PathBuf,
    init_script: &str,
    prefetch_buffer: &Arc<Mutex<PrefetchBuffer>>,
    proxy_nav: &EventLoopProxy<UserEvent>,
    proxy_ipc: &EventLoopProxy<UserEvent>,
    proxy_page: &EventLoopProxy<UserEvent>,
    proxy_title: &EventLoopProxy<UserEvent>,
    initial_mode: ShellMode,
) -> Result<UnifiedShell> {
    crate::diagnostics::log_step("boot: unified compositor (single webview)");

    let (w, h) = super::window_logical_size(window);
    let assets = assets_root.clone();

    let buffer = prefetch_buffer.clone();
    let view = WebViewBuilder::new()
        .with_custom_protocol(ASSET_SCHEME.into(), {
            let proxy = proxy_nav.clone();
            let assets = assets.clone();
            let buffer = buffer.clone();
            move |_id, request| protocol_response(&assets, &buffer, &request, &proxy).into()
        })
        .with_ipc_handler({
            let proxy = proxy_ipc.clone();
            move |req| {
                let _ = proxy.send_event(UserEvent::Ipc(req.body().clone()));
            }
        })
        .with_initialization_script(init_script)
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
        .with_url(shell_entry_url(initial_mode))
        .with_bounds(Rect {
            position: LogicalPosition::new(0.0, 0.0).into(),
            size: LogicalSize::new(w.max(1.0), h.max(1.0)).into(),
        })
        .build_as_child(window)
        .context("build unified webview")?;

    Ok(UnifiedShell {
        layout: CompositorLayout::Unified,
        view,
        chrome: None,
    })
}

pub fn shell_entry_url(mode: ShellMode) -> String {
    match mode {
        ShellMode::App => format!("{ASSET_SCHEME}://localhost/{DASHBOARD_PATH}"),
        ShellMode::Web => format!("{ASSET_SCHEME}://localhost/{FRAME_PATH}"),
    }
}

pub fn set_content_url(view: &WebView, url: &str) -> Result<()> {
    let u = serde_json::to_string(url)?;
    let _ = view.evaluate_script(&format!("window.utahNavigateContent({u});"));
    Ok(())
}

pub fn content_back(view: &WebView) {
    let _ = view.evaluate_script("window.utahContentBack && window.utahContentBack();");
}

pub fn content_forward(view: &WebView) {
    let _ = view.evaluate_script("window.utahContentForward && window.utahContentForward();");
}

pub fn content_reload(view: &WebView) {
    let _ = view.evaluate_script("window.utahContentReload && window.utahContentReload();");
}

pub fn apply_shell_mode(view: &WebView, mode: ShellMode, active_content_url: Option<&str>) -> Result<()> {
    let entry = shell_entry_url(mode);
    view.load_url(&entry)?;
    if mode == ShellMode::Web {
        if let Some(url) = active_content_url {
            set_content_url(view, url)?;
        }
    }
    let name = if mode == ShellMode::Web { "web" } else { "app" };
    let _ = view.evaluate_script(&format!(
        "if(window.utahOnShellMode)window.utahOnShellMode('{name}');"
    ));
    Ok(())
}

pub fn frame_init_script(home_url: &str) -> String {
    let home = serde_json::to_string(home_url).unwrap_or_else(|_| "\"\"".into());
    format!(
        "{TRANSPORT_SCRIPT}\nwindow.__utahHomeUrl = {home};\n\
         document.addEventListener('DOMContentLoaded',function(){{\n\
           if(window.utahNavigateContent)window.utahNavigateContent({home});\n\
         }});\n"
    )
}
