//! Browser core — spatial graph tabs, semantic bookmarks, Wasm extensions, prefetch kernel.

pub mod extensions;
pub mod memory_anchor;
pub mod p2p_search;
pub mod prefetch;
pub mod prefetch_buffer;
pub mod prefetch_worker;
pub mod semantic_bookmarks;
pub mod shield;
pub mod storage_bridge;
pub mod tab_manager;

pub use extensions::{ExtensionRuntime, ExtensionTrigger};
pub use prefetch::PrefetchKernel;
pub use prefetch_buffer::PrefetchBuffer;
pub use semantic_bookmarks::{SemanticBookmarkStore, SemanticHit};
pub use shield::ShieldEngine;
pub use memory_anchor::{MemoryAnchor, MemoryAnchorStore};
pub use tab_manager::{TabInfo, TabManager};
