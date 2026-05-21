# Sovereign Intelligence Stack

Utah Browser V1.0 — edge-native, local-only AI shell.

## Modules

| Module | Path | Role |
|--------|------|------|
| Sentinel-Core | `src/sentinel.rs`, `src/bin/utah_launch.rs` | Orphan launcher, delayed daemons, `shell.ready` |
| Sovereign data | `%APPDATA%/UtahBrowser` | Logs, recovery, evolution, WebView2 profile |
| Ghost-Link | `ghost-link/` | Webcam + mic siphon → `theme.json`, events |
| Truth Engine | `src/truth/` | Ollama + Qdrant RAG + verify |
| Context injection | `src/vault/` | `vault/inject/queue.jsonl` + IPC |
| Memory anchors | `src/browser/memory_anchor.rs` | URL + scroll session snapshots |
| Fluidic UI | `assets/ui/fluidic.css`, `fluidic-paint.js` | Container queries + optional Paint worklet |

## Deploy

```powershell
.\scripts\deploy_sovereign.ps1 -KnowledgePath "C:\path\to\notebooks" -InstallGhostLink
```

```bash
./scripts/deploy_sovereign.sh
```

## IPC (new)

- `inject_context` — push text into vault queue
- `ingest_context_queue` — embed queue into Qdrant
- `create_memory_anchor` — save tab URL + scroll
- `list_memory_anchors`
- Event `sensory_theme` — Ghost-Link ambient palette

## Launch

```powershell
cd dist
.\UtahBrowser.cmd
```

Logs: `%APPDATA%\UtahBrowser\logs\browser.log`
