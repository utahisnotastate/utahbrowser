# Ghost-Link Sovereign Engine

Ghost-Link turns your PC into an **ambient intelligence node**: webcam and microphone feed a local Python daemon that writes sensory events and prefetch hints into the Utah vault. No cloud APIs.

## Install (Windows)

```powershell
cd C:\code\utahbrowser
.\scripts\install_ghost_link.ps1 -StartNow
# Optional: start at logon
.\scripts\install_ghost_link.ps1 -RegisterStartup -StartNow
```

Or with the full Zero-Click Kernel:

```powershell
.\scripts\install.ps1 -InstallGhostLink -GhostLinkStartup
```

## Install (Linux / macOS)

```bash
cd ghost-link
chmod +x run.sh
./run.sh
# Foreground debug:
./run.sh --foreground
```

## Vault layout

| Path | Purpose |
|------|---------|
| `~/.utah_browser/ghost-link/logs/telemetry.log` | Daemon telemetry |
| `~/.utah_browser/ghost-link/out/events.jsonl` | Reasoning events |
| `~/.utah_browser/ghost-link/out/prefetch.json` | Utah Browser Time-Loop hints |

## Environment

| Variable | Default |
|----------|---------|
| `UTAH_VAULT` | `~/.utah_browser` |
| `GHOST_LINK_HOME` | `$UTAH_VAULT/ghost-link` |
| `OLLAMA_HOST` | `http://127.0.0.1:11434` |
| `OLLAMA_VISION_MODEL` | `llava` |
| `GHOST_ENTROPY` | `0.12` |
| `GHOST_DISABLE_CAMERA` | `0` |

## Utah Browser integration

- On `sync_browser`, the Rust shell reads `prefetch.json` and queues URLs in the prefetch kernel.
- IPC command `get_ghost_link_status` returns daemon activity and recent events.

## Vision model

Pull a local vision model before first use:

```bash
ollama pull llava
```

If Ollama or the vision model is offline, the daemon still runs using entropy/heuristic summaries.
