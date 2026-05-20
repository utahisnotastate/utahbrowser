# Utah Browser — technical manual

**Version:** 1.0-GENESIS  
**Repository:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

## Overview

Utah Browser is a native desktop shell (Rust + [Wry](https://github.com/tauri-apps/wry)) with:

- Custom protocol asset server (`utah://localhost`)
- JSON IPC to a local Truth Engine (Ollama embeddings + Qdrant vector search)
- CSS-only UI state (Utah.css); `transport.js` is IPC-only
- Background services: audio capture scaffold (`cpal`), evolution watcher (`notify`)

```
┌─────────────────────────────────────────────────────────┐
│  Tao EventLoop + Wry WebView (shell UI, utah:// assets) │
│         │ IPC (postMessage)          │ custom protocol   │
│         ▼                            ▼                  │
│  Tokio runtime ◄── TruthEngine ──► Ollama / Qdrant       │
│         ▲                                               │
│  Evolution daemon (filesystem → Ollama proposals)         │
│  Audio daemon (cpal input scaffold)                     │
└─────────────────────────────────────────────────────────┘
```

## Requirements

| Component | Version / notes |
|-----------|-----------------|
| Rust | ≥ 1.75 (`rust-version` in `Cargo.toml`) |
| Windows | WebView2 runtime (usually preinstalled on Win11) |
| Ollama | Local HTTP API `127.0.0.1:11434` |
| Qdrant | REST `127.0.0.1:6333` (Docker recommended) |

## Build

```powershell
cargo build --release
cargo run --release
```

Release profile: LTO, `codegen-units = 1`, strip symbols.

### Zero-Click Kernel

```powershell
.\scripts\install.ps1 [-KnowledgePath <path>] [-SkipBuild] [-SkipHealth] [-SkipPull] [-ForcePull]
```

Modules under `scripts/kernel/`:

- `Read-UtahConfig.ps1` — parses `config/default.toml`
- `Health.ps1` — Ollama `/api/tags`, Qdrant `/readyz` chain
- `Models.ps1` — conditional `ollama pull` for `embed_model` and `chat_model`

Artifacts: `dist/utah-browser.exe`, `health-report.json`, `Launch-UtahBrowser.ps1`, `utah.env.ps1`.

## Configuration

File: `config/default.toml`

| Section | Keys | Purpose |
|---------|------|---------|
| `knowledge` | `path`, `extensions` | Notebook corpus root |
| `ollama` | `host`, `embed_model`, `chat_model` | Embedding + optional LLM summary |
| `qdrant` | `url`, `collection`, `vector_size` | Vector store (cosine, 768-dim default) |
| `truth` | `similarity_threshold`, chunk sizes | Verification sensitivity |
| `evolution` | `watch_paths`, `debounce_ms` | Code watcher scope |
| `ui` | `start_url`, `window_title` | Shell defaults |

Environment overrides:

| Variable | Overrides |
|----------|-----------|
| `UTAH_KNOWLEDGE_PATH` | `knowledge.path` |
| `OLLAMA_HOST` | `ollama.host` |
| `QDRANT_URL` | `qdrant.url` |

## Source layout

```
src/
  main.rs              # tracing, Tokio, daemon spawn, engine::run
  lib.rs               # AppState
  config.rs            # TOML load
  engine/mod.rs        # Wry + Tao user events + IPC dispatch
  ipc/messages.rs      # IpcRequest / IpcEvent
  truth/               # ingest, ollama, qdrant, verify
  audio/capture.rs     # cpal loop
  evolution/watcher.rs # notify + proposal files
assets/ui/             # index.html, utah.css, layout.css, transport.js
```

## IPC contract

Transport: `window.utahSend({ cmd: "...", ... })` → Rust `with_ipc_handler` → `UserEvent::Ipc` on main thread.

### Requests (`IpcRequest`)

| `cmd` | Fields | Behavior |
|-------|--------|----------|
| `navigate` | `url` | `webview.load_url` |
| `verify_text` | `text` | Embed → Qdrant search → threshold + optional Ollama summary |
| `ingest_notebooks` | — | Walk knowledge path, chunk, upsert points |
| `get_status` | — | Ollama/Qdrant ping, chunk count |
| `verify_active_tab` | — | Stub (accessibility pipeline planned) |

### Events (`IpcEvent`)

Pushed via `evaluate_script("window.utahOnEvent(...)")` — `status`, `verify_result`, `ingest_progress`, `error`, `evolution_proposal`.

Custom protocol route: `utah://localhost/navigate?url=...` (form-friendly, no JS).

## Truth Engine pipeline

1. **Ingest** — `truth/ingest.rs` walks `knowledge.path`, extracts PDF/text/markdown
2. **Chunk** — overlapping character windows (`chunk_chars`, `chunk_overlap`)
3. **Embed** — `POST {ollama}/api/embeddings`
4. **Store** — `PUT {qdrant}/collections/{name}/points` with SHA256 point ids
5. **Verify** — embed claim, `POST .../points/search`, flag if best score &lt; `similarity_threshold`

## Evolution daemon

- Watches `evolution.watch_paths` for `*.rs`, `*.toml`, `*.css`, `*.html`, `*.md`
- Debounced Ollama completion → markdown file in `evolution/proposals/`
- **Does not** auto-apply patches

## UI architecture (Utah.css)

- Fluid tokens: `clamp()`, `dvh`, `env(safe-area-inset-*)`
- Chrome: CSS container queries (`container-name: chrome`)
- &lt; 48rem: tools overlay + scrim; ≥ 48rem: docked panel grid
- Truth HUD: corner card / narrow bottom sheet

## Security notes

- Custom protocol resolves paths under `assets/ui` only (canonical path check)
- No outbound telemetry in application code
- Secrets excluded via `.gitignore` (`.env`, `utah.env.ps1`)
- iframe `sandbox` attribute on content frame (relaxed for general browsing — review for hardened deployments)

## Planned work

- Windows UI Automation / accessibility for `verify_active_tab`
- Loopback audio → STT → streaming verify
- Embedding dimension auto-detection vs fixed `vector_size`
- CI: `cargo test`, `cargo clippy`, release artifacts

## API references

- [Ollama API](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [Qdrant REST](https://qdrant.tech/documentation/interfaces/#rest-api)
- [Wry](https://docs.rs/wry/latest/wry/)
