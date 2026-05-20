//! Utah Browser V1.0 — entry point.

use std::sync::Arc;
use tracing::info;
use tracing_subscriber::EnvFilter;
use utah_browser::{audio, engine, evolution, AppConfig, AppState};

fn main() -> anyhow::Result<()> {
    tracing_subscriber::fmt()
        .with_env_filter(EnvFilter::from_default_env().add_directive("utah_browser=info".parse()?))
        .init();

    let config = AppConfig::load()?;
    info!(
        "Utah Browser V1.0-GENESIS — knowledge path: {}",
        config.knowledge.path.display()
    );

    let runtime = Arc::new(
        tokio::runtime::Builder::new_multi_thread()
            .enable_all()
            .build()?,
    );

    let state = Arc::new(AppState::new(config.clone()));

    if config.audio.enabled {
        audio::spawn_audio_listener(config.audio.clone());
    }

    let proxy_for_evolution = {
        // Evolution events are logged; optional IPC bridge can be wired via shared proxy later.
        let _state = state.clone();
        Box::new(move |ev: utah_browser::ipc::IpcEvent| {
            if let utah_browser::ipc::IpcEvent::EvolutionProposal { path, summary } = ev {
                info!("evolution proposal for {path}: {summary}");
            }
        }) as evolution::EventCallback
    };

    evolution::spawn_evolution_daemon(config.clone(), runtime.clone(), proxy_for_evolution);

    engine::run(state, runtime);
    Ok(())
}
