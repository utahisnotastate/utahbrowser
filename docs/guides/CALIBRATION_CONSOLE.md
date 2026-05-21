# Calibration Console (Semantic Binding Engine)

The **Calibration Console** replaces static settings menus with live **cognitive zone** bindings. Folders map to your knowledge manifold in real time — no service restart.

## Open console

Click **Calibrate** in the chrome toolbar (or navigate to `#settings-console`).

## Features

| Feature | Description |
|---------|-------------|
| **Map Cognitive Zone** | Native OS folder picker (`rfd`) — instant bind + ingest |
| **Priority weight** | Slider 0.1–5.0 multiplies RAG similarity for that zone |
| **Direct-Mapping** | Skip Qdrant copy; read from disk at verify time (zero-copy) |
| **Health indicator** | Green / yellow / red dot from file readability scan |
| **Sanitize** | One-click removal of unreadable files in a zone |
| **Telemetry** | Ollama/Qdrant status, embed latency, vector point count |

## Vault storage

`~/.utah_browser/vault/zones.json`

## Background ingestion

```powershell
python scripts/ingestion_daemon.py --add-path "C:\your\notes"
```

Watches bound zones and writes `ingest_signal.json` on file changes.

## IPC commands

`bind_knowledge_zone`, `get_calibration_console`, `set_zone_weight`, `sanitize_zone`, `ingest_zone`, etc.
