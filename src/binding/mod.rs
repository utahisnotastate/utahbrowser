//! Semantic Binding Engine — dynamic cognitive zones for the Truth manifold.

pub mod data_binding;
pub mod health;
pub mod telemetry;
pub mod zones;

pub use data_binding::{pick_and_bind, spawn_ingestion_daemon};
pub use telemetry::InferenceTelemetry;
pub use zones::{KnowledgeZone, SemanticBindingStore, ZoneHealth, ZonesManifest};
