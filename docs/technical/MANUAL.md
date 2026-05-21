# Utah Browser — technical manual

**Version:** 1.0-GENESIS  
**Repository:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

---

## Overview

Native desktop shell (Rust + [Wry](https://github.com/tauri-apps/wry)) with:

- Custom protocol `utah://localhost` for shell assets
- JSON IPC between WebView transport layer and Tokio backend
- **Truth Engine:** Ollama embeddings + Qdrant vector search + optional LLM summary
- Utah.css UI (checkbox / `:has()` state); `transport.js` is IPC-only
- Background: `cpal` audio scaffold, `notify` evolution watcher

```
┌──────────────────────────────────────────────────────────┐
│  Tao EventLoop + Wry WebView (utah:// assets)            │
│       │ IPC (postMessage)     │ custom protocol          │
│       ▼                       ▼                          │
│  Tokio ◄── TruthEngine ──► Ollama / Qdrant               │
│       ▲                                                  │
│  Evolution daemon · Audio daemon                           │
└──────────────────────────────────────────────────────────┘
```

---

## Requirements

| Component | Notes |
|-----------|--------|
| Rust | ≥ 1.75 (`rust-version` in `Cargo.toml`) |
| Windows | WebView2 (Edge runtime) |
| Ollama | `127.0.0.1:11434` |
| Qdrant | `127.0.0.1:6333` — **native binary by default**; Docker optional fallback |

---

## Build & install

```powershell
cargo build --release
.\scripts\install.ps1 -KnowledgePath <path>
```

Release: LTO, `codegen-units = 1`, strip.

### Zero-Click Kernel (`scripts/install.ps1`)

| Phase | Module / action |
|-------|-----------------|
| Ollama health | `Health.ps1` → `Test-OllamaHealth` |
| Qdrant ensure | `Ensure-QdrantReady` → `QdrantNative.ps1` then Docker fallback |
| Collection | `Ensure-QdrantCollection` |
| Models | `Models.ps1` → `Ensure-OllamaModels` |
| Build | `Build.ps1` → `Invoke-Cargo` (stderr-safe on PowerShell) |
| Package | `dist/` + `Launch-UtahBrowser.ps1` |

### `scripts/kernel/`

| File | Role |
|------|------|
| `Read-UtahConfig.ps1` | Parse `config/default.toml` |
| `Health.ps1` | Ollama/Qdrant probes, `Ensure-QdrantReady` |
| `QdrantNative.ps1` | Download/start Windows `qdrant.exe` |
| `Models.ps1` | Conditional `ollama pull` |
| `Build.ps1` | Release build + cache repair |
| `Cargo.ps1` | Cargo wrapper (no stderr terminate) |

### Helper scripts

| Script | Role |
|--------|------|
| `Ensure-Qdrant.ps1` | Native Qdrant only |
| `Ensure-Services.ps1` | Ollama + Qdrant (launcher) |
| `Repair-BuildEnvironment.ps1` | Cargo clean / Defender exclusions |

---

## Qdrant native path

1. `Install-QdrantNativeBinary` — GitHub release zip → `%LOCALAPPDATA%\UtahBrowser\qdrant\bin\`
2. `Start-QdrantNative` — `config.yaml`, hidden `qdrant.exe`, PID file
3. Logs: `qdrant.out.log`, `qdrant.err.log` (separate streams)
4. Rust `truth/services.rs` — on failed ping, spawns `Ensure-Qdrant.ps1` on Windows

Config pin: `UtahQdrantVersion` in `QdrantNative.ps1` (e.g. `v1.13.2`).

---

## Configuration

`config/default.toml`

| Section | Keys |
|---------|------|
| `knowledge` | `path`, `extensions` |
| `ollama` | `host`, `embed_model`, `chat_model` |
| `qdrant` | `url`, `collection`, `vector_size` |
| `truth` | `similarity_threshold`, chunk sizes |
| `evolution` | `watch_paths`, `debounce_ms` |
| `ui` | `start_url`, `window_title` |

| Env var | Overrides |
|---------|-----------|
| `UTAH_KNOWLEDGE_PATH` | `knowledge.path` |
| `OLLAMA_HOST` | `ollama.host` |
| `QDRANT_URL` | `qdrant.url` |

---

## Source layout

```
src/
  main.rs
  lib.rs / AppState
  config.rs
  engine/mod.rs       # Wry, UserEvent IPC loop
  ipc/messages.rs
  truth/
    mod.rs
    ingest.rs
    ollama.rs
    qdrant.rs
    verify.rs
    services.rs       # ensure_qdrant_ready, spawn Ensure-Qdrant.ps1
  audio/capture.rs
  evolution/watcher.rs
assets/ui/
scripts/
docs/
```

---

## IPC contract

Transport: `window.utahSend({ cmd })` → `with_ipc_handler` → `EventLoopProxy` → `handle_ipc` on main thread.

### Requests

| `cmd` | Behavior |
|-------|----------|
| `navigate` | `load_url` |
| `ensure_services` | Native/Docker Qdrant ensure + status event |
| `get_status` | Ollama/Qdrant ping, chunk count |
| `ingest_notebooks` | `ensure_qdrant_ready` → walk corpus → upsert |
| `verify_text` | `ensure_qdrant_ready` → embed → search → maybe LLM |
| `verify_active_tab` | Stub |

`transport.js` calls `ensure_services` before ingest/verify.

### Events

`status`, `verify_result`, `ingest_progress`, `error`, `evolution_proposal` via `evaluate_script`.

---

## Truth Engine pipeline

1. Ingest files from `knowledge.path`
2. Chunk (`chunk_chars`, `chunk_overlap`)
3. `POST /api/embeddings` (Ollama)
4. `PUT` points (Qdrant, cosine)
5. Verify: embed claim → search → threshold → optional `api/generate`

---

## UI (Utah.css)

- Fluid `clamp()` / `dvh` / safe-area insets
- Container queries on chrome
- &lt; 48rem: overlay tools panel; ≥ 48rem: docked grid
- Truth HUD: corner or bottom sheet

---

## Security

- Asset protocol path canonicalization under `assets/ui`
- No app-layer telemetry
- iframe `sandbox` on content frame (general browsing; tighten for hardened deploys)

---

## Planned work

- `verify_active_tab` via UI Automation
- Audio → STT → live verify
- Dynamic embedding dimension vs fixed `vector_size`
- CI: clippy, tests, GitHub Actions release

---

## References

- [Ollama API](https://github.com/ollama/ollama/blob/main/docs/api.md)
- [Qdrant REST](https://qdrant.tech/documentation/interfaces/#rest-api)
- [Wry](https://docs.rs/wry/latest/wry/)
- [Installation](../INSTALLATION.md)
- [Qdrant & services](../guides/QDRANT_AND_SERVICES.md)
