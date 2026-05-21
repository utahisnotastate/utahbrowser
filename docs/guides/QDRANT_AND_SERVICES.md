# Qdrant & local services

How Utah Browser runs **Ollama** and **Qdrant** without Docker by default.

**Repository:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

---

## Overview

| Service | Role | Default URL |
|---------|------|-------------|
| **Ollama** | Embeddings + optional text summaries | `http://127.0.0.1:11434` |
| **Qdrant** | Vector search over notebook chunks | `http://127.0.0.1:6333` |

Both must be reachable before **Ingest Notebooks** or **Verify** in the UI.

---

## Qdrant without Docker (default)

The installer and launcher use **`Ensure-QdrantReady`**:

1. Ping `http://127.0.0.1:6333` (`/readyz`, `/healthz`, or `/collections`)
2. If offline → download official Windows binary from [Qdrant releases](https://github.com/qdrant/qdrant/releases) (pinned in `scripts/kernel/QdrantNative.ps1`)
3. Write config + start `qdrant.exe` in the background
4. Wait until API responds (up to 60 seconds)
5. Create collection `utah_notebooks` (from `config/default.toml`) if missing
6. If native start fails **and** Docker is installed → try container `utah-qdrant`

### Storage location

```
%LOCALAPPDATA%\UtahBrowser\qdrant\
  bin\qdrant.exe
  storage\          # vector data
  config.yaml
  qdrant.pid
  qdrant.out.log
  qdrant.err.log
```

---

## Manual Qdrant commands

```powershell
cd C:\code\utahbrowser

# Start or install native Qdrant only
.\scripts\Ensure-Qdrant.ps1

# Ollama + Qdrant (same as launcher)
.\scripts\Ensure-Services.ps1
```

---

## Ollama

Install from [ollama.com](https://ollama.com/). Keep the app running or use:

```powershell
ollama serve
```

Models (from `config/default.toml`, auto-pulled by installer):

- `nomic-embed-text` (embeddings)
- `llama3.2` (optional verification summaries)

---

## When the UI starts services

| Moment | Behavior |
|--------|----------|
| `Launch-UtahBrowser.ps1` | Runs `Ensure-Services.ps1` before exe |
| App load | `transport.js` calls `ensure_services` IPC |
| Ingest / Verify | Rust calls `ensure_qdrant_ready`; may run `Ensure-Qdrant.ps1` on Windows |

---

## Docker fallback (optional)

If native download or start fails:

```powershell
docker run -d -p 6333:6333 --name utah-qdrant --restart unless-stopped qdrant/qdrant
```

Requires Docker Desktop running.

To **disable all** auto-start (including native):

```powershell
.\scripts\install.ps1 -SkipQdrantStart
```

---

## Common errors

| Message | Fix |
|---------|-----|
| RedirectStandardOutput / RedirectStandardError same | Fixed in current repo — `git pull` and re-run |
| Download failed | Check firewall/proxy for GitHub releases |
| API not ready after start | Read `qdrant.err.log`; port 6333 may be in use |
| Docker not installed | Not an error if native path succeeds |

---

## Utahnetes?

[Utahnetes](https://github.com/utahisnotastate/utahnetes) is a **peer-to-peer WASM swarm** for LAN demos. It does **not** store notebook embeddings and cannot replace Qdrant in Utah Browser.

---

## See also

- [Installation](../INSTALLATION.md)
- [Build troubleshooting](BUILD_TROUBLESHOOTING.md)
- [Technical manual](../technical/MANUAL.md)
