# Configuration reference

Utah Browser reads `config/default.toml` from the runtime bundle (`dist/config/` after build). Environment variables override selected keys.

---

## File locations

| Context | Path |
|---------|------|
| Development | `<repo>/config/default.toml` |
| After `Build-Demo.ps1` | `dist/config/default.toml` (demo profile) |
| User data (logs, vault, WebView2) | `%APPDATA%\UtahBrowser\` |

---

## Sections

### `[knowledge]`

| Key | Default | Description |
|-----|---------|-------------|
| `path` | Public Documents placeholder | Folder of `.md`, `.txt`, `.pdf` for Truth Guard |
| `extensions` | `md`, `txt`, `markdown`, `pdf` | Ingest file types |

### `[ollama]` / `[qdrant]` / `[truth]`

Local AI and vector search. See [QDRANT_AND_SERVICES.md](guides/QDRANT_AND_SERVICES.md).

### `[browser]`

| Key | Default | Description |
|-----|---------|-------------|
| `prefetch_enabled` | `true` | Intent-resolution prefetch queue |
| `prefetch_buffer_max_mb` | `8` | RAM cap for `utah://localhost/buffer/*` |
| `suspend_on_switch` | `true` | Serialize inactive tabs to vault cache |
| `bookmarks_collection` | `utah_bookmarks` | Qdrant collection for semantic bookmarks |

### `[ui]`

| Key | Default | Description |
|-----|---------|-------------|
| `start_url` | `https://www.google.com` | Home / new-tab URL |
| `window_title` | `Utah Browser // V1.0-GENESIS` | Window caption |

### `[ghost_link]` / `[evolution]` / `[urm]`

Optional subsystems. Demo profile disables Ghost-Link and evolution. URM is driven by Python install scripts, not the Rust config struct today.

---

## Environment variables

| Variable | Overrides |
|----------|-----------|
| `UTAH_KNOWLEDGE_PATH` | `[knowledge].path` |
| `UTAH_BROWSER_HOME` | Runtime root (exe-relative assets) |
| `OLLAMA_HOST` | `[ollama].host` |
| `QDRANT_URL` | `[qdrant].url` |
| `UTAH_DEMO_MODE=1` | Demo logging + `demo.toml` when built via `Build-Demo.ps1` |
| `UTAH_LEGACY_DUAL=1` | Opt-in two-WebView mode (legacy) |

---

## Demo profile

`config/demo.toml` is copied to `dist/config/default.toml` by `Build-Demo.ps1`:

- Evolution off, Ghost-Link off, URM off
- Prefetch on (4 MiB buffer cap)
- Window title: `Utah Browser // DEMO`

---

## See also

- [INSTALLATION.md](INSTALLATION.md)
- [DEMO.md](DEMO.md)
- [REPOSITORY_SCOPE.md](REPOSITORY_SCOPE.md)
