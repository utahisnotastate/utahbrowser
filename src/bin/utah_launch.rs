//! Sentinel launcher — spawn browser core detached, exit immediately (orphaned parent).

#![cfg_attr(all(windows, not(debug_assertions)), windows_subsystem = "windows")]

use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::{env, process};

#[cfg(windows)]
use std::os::windows::process::CommandExt;

#[cfg(windows)]
const DETACHED_PROCESS: u32 = 0x00000008;

#[cfg(windows)]
const CREATE_NEW_PROCESS_GROUP: u32 = 0x00000200;

#[cfg(windows)]
const CREATE_NO_WINDOW: u32 = 0x08000000;

fn main() {
    if let Err(e) = run() {
        show_error(&format!("Utah Browser could not start:\n{e}"));
        process::exit(1);
    }
    // Orphaned launcher — do not wait on child; avoids 0xcfffffff false failures.
    process::exit(0);
}

fn run() -> Result<(), String> {
    let home = install_home()?;
    env::set_current_dir(&home).map_err(|e| e.to_string())?;
    env::set_var("UTAH_BROWSER_HOME", &home);

    bootstrap_services(&home);

    let browser = home.join("utah-browser.exe");
    if !browser.is_file() {
        return Err(format!(
            "utah-browser.exe not found in {}\nRun install.ps1 once to build the dist folder.",
            home.display()
        ));
    }

    let mut cmd = Command::new(&browser);
    cmd.current_dir(&home)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());

    #[cfg(windows)]
    cmd.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);

    cmd.spawn()
        .map_err(|e| format!("failed to start browser: {e}"))?;

    Ok(())
}

fn install_home() -> Result<PathBuf, String> {
    let exe = env::current_exe().map_err(|e| e.to_string())?;
    let dir = exe
        .parent()
        .ok_or_else(|| "invalid executable path".to_string())?
        .to_path_buf();
    Ok(dir)
}

fn bootstrap_services(home: &PathBuf) {
    let ensure = home.join("scripts").join("Ensure-Services.ps1");
    if !ensure.is_file() {
        return;
    }
    let mut ps = Command::new("powershell");
    ps.args([
        "-NoProfile",
        "-WindowStyle",
        "Hidden",
        "-ExecutionPolicy",
        "Bypass",
        "-File",
    ])
    .arg(&ensure)
    .arg("-ProjectRoot")
    .arg(home)
    .stdin(Stdio::null())
    .stdout(Stdio::null())
    .stderr(Stdio::null());

    #[cfg(windows)]
    ps.creation_flags(DETACHED_PROCESS | CREATE_NEW_PROCESS_GROUP | CREATE_NO_WINDOW);

    let _ = ps.spawn();
}

#[cfg(windows)]
fn show_error(msg: &str) {
    use std::ffi::OsStr;
    use std::os::windows::ffi::OsStrExt;
    use std::ptr::null_mut;

    #[link(name = "user32")]
    extern "system" {
        fn MessageBoxW(hwnd: *mut (), text: *const u16, caption: *const u16, utype: u32) -> i32;
    }

    let text: Vec<u16> = OsStr::new(msg).encode_wide().chain(Some(0)).collect();
    let caption: Vec<u16> = OsStr::new("Utah Browser")
        .encode_wide()
        .chain(Some(0))
        .collect();
    unsafe {
        MessageBoxW(null_mut(), text.as_ptr(), caption.as_ptr(), 0x10);
    }
}

#[cfg(not(windows))]
fn show_error(msg: &str) {
    eprintln!("{msg}");
}
