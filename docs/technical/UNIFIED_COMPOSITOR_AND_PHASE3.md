# Unified Compositor & Phase 3 (SOTA stack)

Utah Browser’s default shell is a **single WebView2** instance. Chrome (tabs, URL bar, bookmarks) and web content (external sites) share one engine: chrome is HTML; content is an `<iframe>`.

**Repository:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

---

## Phase 1 — Sentinel / de-bloat (shipped)

| Item | Implementation |
|------|----------------|
| Remove dual WebView by default | `compositor::boot()` in `src/engine/compositor.rs` |
| Legacy dual (opt-in) | `UTAH_LEGACY_DUAL=1` → `boot_legacy_dual()` |
| Ghost-Chrome frame | `assets/ui/browser_frame.html` |
| Launcher false crash | `src/bin/utah_launch.rs` detached spawn |
| Sovereign paths | `%APPDATA%\UtahBrowser\` via `src/paths.rs`, `src/sentinel.rs` |

---

## Phase 2 — Ghost-Chrome UI (shipped)

| Item | Implementation |
|------|----------------|
| Fluidic tiles / CSS | `assets/ui/fluidic.css`, `fluidic-paint.js` on frame + dashboard |
| Sensory theme | `sensory-theme.js` + Ghost-Link `theme.json` poll |
| Bookmark tether drag | `assets/ui/chrome-drag.js` |
| Semantic resize | Unified mode: full-window WebView bounds on `Resized` (Rust), chrome flex in CSS |

---

## Phase 3 — SOTA features (shipped)

### 1. Direct memory-buffer injection

Hot prefetches are stored in RAM and served without hitting the network again:

- **Rust:** `src/browser/prefetch_buffer.rs`, `prefetch_worker.rs`
- **Route:** `utah://localhost/buffer/{id}` via `protocol_response()` in `src/engine/mod.rs`
- **Config:** `browser.prefetch_buffer_max_mb` (default 8) in `config/default.toml`
- **UI:** `window.utahPreloadBuffer(buffer_uri)` in `compositor.js` + `prefetch_buffered` IPC event

This bypasses repeated OS/network round-trips for hinted URLs (first fetch still uses HTTP Range).

### 2. Intent-resolution buffer

| Layer | File |
|-------|------|
| Cursor trajectory → hint | `assets/ui/intent-resolver.js` |
| Bookmark hover hint | `assets/ui/browser.js` |
| Queue + DNS/GET warm | `PrefetchKernel` + `PrefetchWarm` user event |
| Ghost-Link hints | `ghost-link` → `prefetch.json` → `absorb_ghost_prefetch` |

**Note:** Intent tracking applies to **chrome DOM** only (cross-origin iframe content cannot be read).

### 3. WASM-native extensions

| Layer | File |
|-------|------|
| Sandbox host | `src/browser/extensions.rs` (wasmi) |
| Triggers | `NAVIGATION` (page load), `DOM_LOADED` (frame ready), manifest `CLICK` |
| Create via UI | IPC `vibe_extension` |
| Vault path | `%APPDATA%/UtahBrowser/extensions/{name}/module.wasm` |

Extensions export `on_event(i32) -> i32`. Vibe-create still ships a minimal stub module until the AI compile pipeline is wired.

---

## Boot modes

| Mode | Log label | How |
|------|-----------|-----|
| **Unified** (default) | `shell ready (unified)` | Normal launch |
| Legacy dual | `shell ready (legacy-dual)` | `UTAH_LEGACY_DUAL=1` |
| Demo | `UTAH_DEMO_MODE=1` | `Build-Demo.ps1` — unified + `demo.toml` |

---

## Key files

```
src/engine/compositor.rs      # Unified WebView boot
src/engine/mod.rs             # protocol_response, IPC, shell_navigate
src/browser/prefetch_buffer.rs
src/browser/prefetch_worker.rs
assets/ui/browser_frame.html
assets/ui/compositor.js
assets/ui/intent-resolver.js
```

---

## Related docs

- [Crash on launch report](CRASH_ON_LAUNCH_REPORT.md) — historical dual-WebView issues
- [System architecture report](SYSTEM_ARCHITECTURE_REPORT.md) — full stack
- [Demo guide](../DEMO.md) — evaluator quick start
