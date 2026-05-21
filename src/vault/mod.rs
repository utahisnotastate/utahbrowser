//! Sovereign vault — context injection queue for third-party → RAG pipeline.

mod context_injection;

pub use context_injection::{enqueue_context, ingest_pending, ContextPacket};
