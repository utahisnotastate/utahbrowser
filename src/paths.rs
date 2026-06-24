//! Install root (read-only packaged assets) vs sovereign data dir (runtime writes).

use std::path::{Path, PathBuf};

/// Directory containing `utah-browser.exe` (or repo root in dev) — config/assets only at runtime.
pub fn install_root() -> PathBuf {
    if let Ok(home) = std::env::var("UTAH_BROWSER_HOME") {
        let p = PathBuf::from(home);
        if p.is_dir() {
            return p;
        }
    }
    if let Ok(exe) = std::env::current_exe() {
        if let Some(parent) = exe.parent() {
            if parent.join("config").is_dir() || parent.join("utah-browser.exe").is_file() {
                return parent.to_path_buf();
            }
        }
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR"))
}

/// Writable runtime root — `%APPDATA%/UtahBrowser` on Windows.
pub fn sovereign_data_root() -> PathBuf {
    if let Ok(dir) = std::env::var("UTAH_SOVEREIGN_DATA") {
        let p = PathBuf::from(dir);
        if !p.as_os_str().is_empty() {
            return p;
        }
    }
    #[cfg(windows)]
    {
        if let Ok(appdata) = std::env::var("APPDATA") {
            return PathBuf::from(appdata).join("UtahBrowser");
        }
    }
    dirs::home_dir()
        .unwrap_or_else(|| PathBuf::from("."))
        .join(".utah_browser")
        .join("UtahBrowser")
}

pub fn ensure_sovereign_dirs() -> std::io::Result<()> {
    for dir in [
        sovereign_data_root(),
        sovereign_data_root().join("logs"),
        evolution_proposals_dir(),
        sovereign_webview2_dir(),
    ] {
        std::fs::create_dir_all(&dir)?;
    }
    Ok(())
}

pub fn sovereign_browser_log() -> PathBuf {
    sovereign_data_root().join("logs").join("browser.log")
}

pub fn sovereign_recovery_path() -> PathBuf {
    sovereign_data_root().join("recovery.json")
}

pub fn sovereign_ready_signal_path() -> PathBuf {
    sovereign_data_root().join("shell.ready")
}

pub fn instance_lock_path() -> PathBuf {
    sovereign_data_root().join("utah-browser.lock")
}

/// Optional mirror so `dist/logs/browser.log` stays current during portable runs.
pub fn install_log_mirror() -> PathBuf {
    install_root().join("logs").join("browser.log")
}

pub fn evolution_log_path() -> PathBuf {
    sovereign_data_root().join("logs").join("evolution.log")
}

pub fn evolution_proposals_dir() -> PathBuf {
    sovereign_data_root().join("evolution").join("proposals")
}

pub fn sovereign_webview2_dir() -> PathBuf {
    sovereign_data_root().join("webview2")
}

pub fn config_path() -> PathBuf {
    let portable = install_root().join("config/default.toml");
    if portable.is_file() {
        return portable;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("config/default.toml")
}

pub fn assets_ui_dir() -> PathBuf {
    let portable = install_root().join("assets/ui");
    if portable.is_dir() {
        return portable;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("assets/ui")
}

pub fn scripts_dir() -> PathBuf {
    let portable = install_root().join("scripts");
    if portable.is_dir() {
        return portable;
    }
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("scripts")
}

/// Legacy helper — prefer [`evolution_proposals_dir`] for evolution output.
pub fn resolve_under_install(rel: impl AsRef<Path>) -> PathBuf {
    install_root().join(rel.as_ref())
}

/// SOTA Absolute Void-State: Cryptographic zero-fill of a directory before deletion.
pub fn zero_fill_dir(path: &Path) -> std::io::Result<()> {
    use std::io::Write;
    if !path.is_dir() {
        return Ok(());
    }
    for entry in std::fs::read_dir(path)? {
        let entry = entry?;
        let p = entry.path();
        if p.is_file() {
            if let Ok(mut f) = std::fs::OpenOptions::new().write(true).open(&p) {
                if let Ok(meta) = std::fs::metadata(&p) {
                    let len = meta.len();
                    let zeroes = vec![0u8; 8192];
                    let mut remaining = len;
                    while remaining > 0 {
                        let to_write = remaining.min(8192) as usize;
                        let _ = f.write_all(&zeroes[..to_write]);
                        remaining -= to_write as u64;
                    }
                    let _ = f.flush();
                }
            }
            let _ = std::fs::remove_file(p);
        } else if p.is_dir() {
            let _ = zero_fill_dir(&p);
        }
    }
    let _ = std::fs::remove_dir(path);
    Ok(())
}
