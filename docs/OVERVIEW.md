# Utah Browser — product overview

Utah Browser is a **privacy-first desktop web shell for Windows** built in Rust. It uses the system **WebView2** runtime (via [Wry](https://github.com/tauri-apps/wry)) instead of bundling Chromium or Electron.

## What it does today

1. **Browse the web** — unified compositor: Ghost-Chrome header + content iframe in one engine.
2. **Truth Guard** — ingest your local notebooks (PDF/Markdown/text) and verify statements against that corpus using **Ollama** + **Qdrant** on your machine.
3. **Stay local** — no application-layer telemetry; services are localhost-only.

## Architecture at a glance

```
┌─────────────────────────────────────────┐
│  Tao window + one WebView2 (compositor) │
│    utah:// assets  │  iframe → https:// │
└──────────┬──────────────────────────────┘
           │ JSON IPC
           ▼
┌─────────────────────────────────────────┐
│  Rust engine + Tokio                    │
│    Tabs · Bookmarks · Prefetch buffer   │
│    Truth Engine · Vault · Extensions    │
└──────────┬──────────────────────────────┘
           │
     Ollama / Qdrant (local)
```

## Who this is for

| Audience | Start here |
|----------|------------|
| Evaluators / sponsors | [DEMO.md](DEMO.md) |
| Everyday users | [guides/FOR_EVERYONE.md](guides/FOR_EVERYONE.md) |
| Developers | [technical/MANUAL.md](technical/MANUAL.md) |
| Incident response | [technical/CRASH_ON_LAUNCH_REPORT.md](technical/CRASH_ON_LAUNCH_REPORT.md) |

## Repository boundaries

This repo is **browser-only**. See [REPOSITORY_SCOPE.md](REPOSITORY_SCOPE.md) for what belongs in public source control.

## Support

Optional donations: [utah@utahcreates.com](mailto:utah@utahcreates.com) (see root [README](../README.md)).
