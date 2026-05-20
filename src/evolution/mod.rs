//! Autonomous Evolution — filesystem watcher + local LLM optimization proposals.

mod watcher;

pub use watcher::{spawn_evolution_daemon, EventCallback};
