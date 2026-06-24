//! Utah Browser V1.0 — entry point (Sentinel-Core boot).

use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;
use utah_browser::{diagnostics, engine, sentinel, AppConfig, AppState};

fn main() {
    diagnostics::prepare_environment();
    let _instance_guard = match diagnostics::acquire_instance_lock() {
        Ok(g) => g,
        Err(e) => {
            diagnostics::show_fatal("Utah Browser", &e.to_string());
            std::process::exit(1);
        }
    };
    diagnostics::log_step("Utah Browser process start (Sentinel-Core)");

    std::panic::set_hook(Box::new(|info| {
        let msg = format!("PANIC: {info}");
        diagnostics::record_boot_failure(&msg);
        diagnostics::show_fatal("Utah Browser crashed", &diagnostics::fatal_message(&msg));
    }));

    if let Err(e) = run_app() {
        let msg = format!("{e:#}");
        diagnostics::record_boot_failure(&msg);
        diagnostics::show_fatal("Utah Browser", &diagnostics::fatal_message(&msg));
        std::process::exit(1);
    }
}

fn run_app() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("utah_browser=info".parse()?))
        .init();

    let config = AppConfig::load()?;
    diagnostics::log_step(&format!(
        "config loaded - home: {} evolution: {} sovereign: {}",
        config.ui.start_url,
        if config.evolution.enabled { "on" } else { "off" },
        utah_browser::paths::sovereign_data_root().display()
    ));

    let py = std::env::var("UTAH_PYTHON").unwrap_or_else(|_| "python".into());
    let py_found = std::process::Command::new(&py).arg("--version").status().is_ok();
    info!(
        "Utah Browser V1.0-GENESIS — knowledge path: {} (Python: {})",
        config.knowledge.path.display(),
        if py_found { &py } else { "NOT FOUND" }
    );

    let recovery = diagnostics::load_recovery();
    if recovery.should_use_safe_mode() {
        diagnostics::log_step("recovery: safe mode will be used for this session");
    }

    let runtime = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?,
    );

    let state = Arc::new(AppState::new(config.clone())?);

    // Initialize high-speed IPC Nexus (SOTA Phase 7)
    let nexus = utah_browser::ipc::nexus::IpcNexus::new(9002, state.quantum.clone());
    runtime.spawn(async move {
        nexus.initialize_bridge().await;
    });

    // Sentinel-Core: daemons start only after shell.ready + grace (never during WebView boot).
    sentinel::spawn_delayed_background_services(config, runtime.clone(), recovery);

    engine::run(state, runtime);
    Ok(())
}
