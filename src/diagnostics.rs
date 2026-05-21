//! Startup diagnostics, sovereign logging, recovery state, and user-visible errors.

use anyhow::Result;
use serde::{Deserialize, Serialize};
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::sync::Mutex;
use std::time::{SystemTime, UNIX_EPOCH};

static LOG_MUTEX: Mutex<()> = Mutex::new(());

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RecoveryState {
    pub consecutive_failures: u32,
    pub last_error: Option<String>,
    pub last_boot_mode: Option<String>,
    pub force_safe_mode: bool,
    pub last_success_unix: Option<u64>,
}

impl RecoveryState {
    pub fn should_use_safe_mode(&self) -> bool {
        self.force_safe_mode || self.consecutive_failures >= 2
    }
}

pub fn recovery_path() -> PathBuf {
    crate::paths::sovereign_recovery_path()
}

/// Primary log is sovereign; mirrors to dist/logs when present + %TEMP%.
pub fn log_paths() -> Vec<PathBuf> {
    let mut paths = vec![crate::paths::sovereign_browser_log()];
    let mirror = crate::paths::install_log_mirror();
    if let Some(parent) = mirror.parent() {
        let _ = std::fs::create_dir_all(parent);
        paths.push(mirror);
    }
    if let Some(temp) = std::env::temp_dir().to_str() {
        paths.push(PathBuf::from(temp).join("utah-browser.log"));
    }
    paths
}

/// Prevent two WebView2 instances fighting over the same profile (common crash on relaunch).
pub struct InstanceLock {
    path: PathBuf,
}

impl Drop for InstanceLock {
    fn drop(&mut self) {
        let _ = std::fs::remove_file(&self.path);
    }
}

pub fn acquire_instance_lock() -> Result<InstanceLock> {
    let _ = crate::paths::ensure_sovereign_dirs();
    let path = crate::paths::instance_lock_path();
    if path.is_file() {
        if let Ok(raw) = std::fs::read_to_string(&path) {
            if let Ok(pid) = raw.trim().parse::<u32>() {
                if process_is_alive(pid) {
                    anyhow::bail!(
                        "Utah Browser is already running (process {pid}). Close it before starting again."
                    );
                }
            }
        }
        let _ = std::fs::remove_file(&path);
    }
    std::fs::write(&path, std::process::id().to_string())?;
    Ok(InstanceLock { path })
}

#[cfg(windows)]
fn process_is_alive(pid: u32) -> bool {
    use std::ptr::null_mut;
    const PROCESS_QUERY_LIMITED_INFORMATION: u32 = 0x1000;
    #[link(name = "kernel32")]
    extern "system" {
        fn OpenProcess(dwDesiredAccess: u32, bInheritHandle: i32, dwProcessId: u32) -> *mut ();
        fn CloseHandle(hObject: *mut ()) -> i32;
    }
    unsafe {
        let h = OpenProcess(PROCESS_QUERY_LIMITED_INFORMATION, 0, pid);
        if h.is_null() {
            return false;
        }
        CloseHandle(h);
        true
    }
}

#[cfg(not(windows))]
fn process_is_alive(pid: u32) -> bool {
    std::fs::read(format!("/proc/{pid}/stat")).is_ok()
}

/// Call before WebView2 initializes.
pub fn prepare_environment() {
    let _ = crate::paths::ensure_sovereign_dirs();
    crate::sentinel::clear_ready_signal();
    let data = crate::paths::sovereign_webview2_dir();
    let _ = std::fs::create_dir_all(&data);
    std::env::set_var(
        "WEBVIEW2_USER_DATA_FOLDER",
        data.to_string_lossy().as_ref(),
    );
    log_step(&format!(
        "environment prepared (sovereign: {})",
        crate::paths::sovereign_data_root().display()
    ));
}

pub fn log_step(msg: &str) {
    let ts = timestamp();
    let line = format!("[{ts}] {msg}\n");
    let _guard = LOG_MUTEX.lock().unwrap_or_else(|e| e.into_inner());
    for path in log_paths() {
        append_line(path, &line);
    }
    tracing::info!("{msg}");
}

pub fn log_error(context: &str, err: &str) {
    log_step(&format!("ERROR [{context}]: {err}"));
}

pub fn load_recovery() -> RecoveryState {
    let path = recovery_path();
    if !path.is_file() {
        return RecoveryState::default();
    }
    match std::fs::read_to_string(&path) {
        Ok(raw) => serde_json::from_str(&raw).unwrap_or_default(),
        Err(e) => {
            log_step(&format!("recovery.json unreadable: {e}"));
            RecoveryState::default()
        }
    }
}

pub fn save_recovery(state: &RecoveryState) -> Result<()> {
    let path = recovery_path();
    if let Some(parent) = path.parent() {
        std::fs::create_dir_all(parent)?;
    }
    let raw = serde_json::to_string_pretty(state)?;
    std::fs::write(&path, raw)?;
    Ok(())
}

pub fn record_boot_success(mode: &str) {
    let mut r = load_recovery();
    r.consecutive_failures = 0;
    r.last_error = None;
    r.last_boot_mode = Some(mode.into());
    r.last_success_unix = Some(unix_now());
    let _ = save_recovery(&r);
    log_step(&format!("boot OK ({mode})"));
}

pub fn record_boot_failure(err: &str) {
    let mut r = load_recovery();
    r.consecutive_failures = r.consecutive_failures.saturating_add(1);
    r.last_error = Some(err.chars().take(2000).collect());
    if r.consecutive_failures >= 2 {
        r.force_safe_mode = true;
        log_step("auto-enabling safe mode for next launch");
    }
    let _ = save_recovery(&r);
    log_step(&format!("boot FAILED (#{}): {err}", r.consecutive_failures));
}

pub fn clear_safe_mode() {
    let mut r = load_recovery();
    r.force_safe_mode = false;
    r.consecutive_failures = 0;
    let _ = save_recovery(&r);
    log_step("safe mode cleared");
}

pub fn show_fatal(title: &str, body: &str) {
    log_step(&format!("FATAL DIALOG: {title} — {body}"));
    #[cfg(windows)]
    show_message_box(title, body);
    #[cfg(not(windows))]
    eprintln!("{title}\n{body}");
}

pub fn fatal_message(err: &str) -> String {
    let logs = log_paths()
        .iter()
        .map(|p| p.display().to_string())
        .collect::<Vec<_>>()
        .join("\n  ");
    format!(
        "{err}\n\nDiagnostics written to:\n  {logs}\n\nRecovery: {}\n\nInstall folder stays read-only at runtime. If this keeps happening, safe mode will start automatically.",
        recovery_path().display()
    )
}

fn append_line(path: PathBuf, line: &str) {
    if let Some(parent) = path.parent() {
        let _ = std::fs::create_dir_all(parent);
    }
    if let Ok(mut f) = OpenOptions::new().create(true).append(true).open(&path) {
        let _ = f.write_all(line.as_bytes());
        let _ = f.flush();
    }
}

fn timestamp() -> String {
    let secs = unix_now();
    format!("{secs}")
}

fn unix_now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

#[cfg(windows)]
fn show_message_box(title: &str, body: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(hwnd: *mut (), text: *const u16, caption: *const u16, utype: u32) -> i32;
    }

    let text: Vec<u16> = OsStr::new(body).encode_wide().chain(Some(0)).collect();
    let caption: Vec<u16> = OsStr::new(title).encode_wide().chain(Some(0)).collect();
    unsafe {
        MessageBoxW(null_mut(), text.as_ptr(), caption.as_ptr(), 0x10);
    }
}
