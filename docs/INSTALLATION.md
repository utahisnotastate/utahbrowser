# Installation guide

**Repository:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

This guide describes the **Zero-Click Kernel** (`scripts/install.ps1`) end to end.

---

## Prerequisites

| Requirement | Why |
|-------------|-----|
| **Windows 10/11** with **WebView2** | Renders the browser shell (usually preinstalled) |
| **Rust** ([rustup.rs](https://rustup.rs/)) | Compiles `utah-browser.exe` |
| **Ollama** ([ollama.com](https://ollama.com/)) | Local embeddings + optional summaries |
| **Internet (first run)** | One-time Qdrant binary download (~50 MB); Ollama model pull |

**Docker is not required.** Qdrant runs as a **native Windows binary** downloaded automatically.

---

## One-command install

From the **repository root** (recommended):

```powershell
cd C:\code\utahbrowser
.\scripts\install.ps1 -KnowledgePath "C:\path\to\your\notebooks"
```

Running from `scripts\` also works; the installer resolves the project root automatically.

---

## What the installer does (in order)

| Phase | Action |
|-------|--------|
| **1. Ollama** | Ping `http://127.0.0.1:11434/api/tags` |
| **2. Qdrant** | Check port 6333; if offline → download [Qdrant release](https://github.com/qdrant/qdrant/releases) → start `qdrant.exe` → create collection |
| **3. Models** | `ollama pull` for `embed_model` and `chat_model` from `config/default.toml` |
| **4. Build** | `cargo build --release` (auto-repair on cache errors) |
| **5. Qdrant recheck** | Ensure API still up before packaging |
| **6. Dist** | Copy exe, config, assets, scripts → `dist/` |

---

## Outputs in `dist/`

| File | Purpose |
|------|---------|
| `utah-browser.exe` | The browser |
| `Launch-UtahBrowser.ps1` | Ensures services, loads env, starts exe |
| `utah.env.ps1` | `OLLAMA_HOST`, `QDRANT_URL`, optional `UTAH_KNOWLEDGE_PATH` |
| `health-report.json` | Last health/pull results |
| `config/`, `assets/`, `scripts/` | Runtime bundle |

---

## Installer flags

| Flag | Effect |
|------|--------|
| `-KnowledgePath <path>` | Sets notebook folder for Truth Guard (requires Qdrant to start) |
| `-SkipBuild` | Health + models only; no compile |
| `-SkipHealth` | Build/package only; skip Ollama/Qdrant |
| `-SkipPull` | Do not run `ollama pull` |
| `-SkipQdrantStart` | Do not auto-start Qdrant (native or Docker) |
| `-ForcePull` | Re-download Ollama models even if present |
| `-RepairOnly` | `cargo clean` + rebuild only |

---

## After install

```powershell
.\dist\Launch-UtahBrowser.ps1
```

Or from repo root during development:

```powershell
cargo run --release
```

---

## Qdrant data locations

| Path | Contents |
|------|----------|
| `%LOCALAPPDATA%\UtahBrowser\qdrant\` | Native binary, storage, config, logs |
| `%LOCALAPPDATA%\UtahBrowser\qdrant\bin\qdrant.exe` | Downloaded server |
| `qdrant.out.log` / `qdrant.err.log` | Process logs |

Project-local override: `.utah-browser\qdrant\` when using a dev tree.

---

## Troubleshooting

| Issue | Doc |
|-------|-----|
| Rust build failed | [BUILD_TROUBLESHOOTING.md](guides/BUILD_TROUBLESHOOTING.md) |
| Qdrant won't start | [QDRANT_AND_SERVICES.md](guides/QDRANT_AND_SERVICES.md) |
| General use | [FOR_EVERYONE.md](guides/FOR_EVERYONE.md) |

Quick repairs:

```powershell
.\scripts\Repair-BuildEnvironment.ps1
.\scripts\Ensure-Qdrant.ps1
.\scripts\install.ps1 -KnowledgePath "C:\path\to\notebooks"
```

---

## Exit codes

| Code | Meaning |
|------|---------|
| `0` | Success (Ollama + Qdrant healthy when health checks ran) |
| `2` | Dist may exist but Ollama or Qdrant was unhealthy |
