<div align="center">

# Utah Browser

**Privacy-first desktop browser for Windows** — Rust + WebView2, local Truth Engine, unified compositor shell.

[![GitHub](https://img.shields.io/badge/GitHub-utahisnotastate%2Futahbrowser-blue)](https://github.com/utahisnotastate/utahbrowser)

[Demo](docs/DEMO.md) · [Install](docs/INSTALLATION.md) · [Docs hub](docs/README.md) · [Overview](docs/OVERVIEW.md)

</div>

---

## What is this?

Utah Browser is an **open-source browser shell** that keeps AI-assisted fact-checking on **your hardware**:

- **Browse** with a single WebView2 instance (Ghost-Chrome tabs + content iframe)
- **Truth Guard** — index your notebooks and verify claims via **Ollama** + **Qdrant**
- **No cloud telemetry** in application code

**Repository:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

> This repo contains **the browser only**. Other private or experimental apps belong in separate repositories. See [docs/REPOSITORY_SCOPE.md](docs/REPOSITORY_SCOPE.md).

---

## Documentation

| Audience | Document |
|----------|----------|
| **Run the demo (fastest)** | [docs/DEMO.md](docs/DEMO.md) |
| Product overview | [docs/OVERVIEW.md](docs/OVERVIEW.md) |
| Install (full stack) | [docs/INSTALLATION.md](docs/INSTALLATION.md) |
| Configuration | [docs/CONFIGURATION.md](docs/CONFIGURATION.md) |
| Unified compositor & prefetch | [docs/technical/UNIFIED_COMPOSITOR_AND_PHASE3.md](docs/technical/UNIFIED_COMPOSITOR_AND_PHASE3.md) |
| Architecture handoff | [docs/technical/SYSTEM_ARCHITECTURE_REPORT.md](docs/technical/SYSTEM_ARCHITECTURE_REPORT.md) |
| Launch / crash issues | [docs/technical/CRASH_ON_LAUNCH_REPORT.md](docs/technical/CRASH_ON_LAUNCH_REPORT.md) |
| Everyday users | [docs/guides/FOR_EVERYONE.md](docs/guides/FOR_EVERYONE.md) |
| Kids & families | [docs/guides/FOR_KIDS.md](docs/guides/FOR_KIDS.md) |
| Ollama & Qdrant | [docs/guides/QDRANT_AND_SERVICES.md](docs/guides/QDRANT_AND_SERVICES.md) |
| All docs | [docs/README.md](docs/README.md) |
| Roadmap | [docs/ROADMAP.md](docs/ROADMAP.md) |

---

## Features (shipped)

- **Unified compositor** — one WebView2; chrome HTML + content iframe (no dual-engine contention)
- **Intent prefetch** — bookmark/cursor hints → DNS warm + RAM buffer (`utah://localhost/buffer/…`)
- **WASM extensions** — wasmi sandbox under `%APPDATA%\UtahBrowser\extensions\`
- **Truth Guard** — ingest PDF/Markdown/text; verify against your corpus
- **Zero-Click Kernel** — `install.ps1`: Ollama health, native Qdrant, model pull, release build
- **Responsive UI** — Utah.css layout; Ghost-Chrome / fluidic styling
- **No telemetry** — local services only

Optional (documented, off in demo): Ghost-Link sensory daemon, URM Nexus orchestrator.

---

## Quick start — Demo

```powershell
git clone https://github.com/utahisnotastate/utahbrowser.git
cd utahbrowser
.\scripts\Build-Demo.ps1
cd dist
.\UtahBrowser.cmd
```

Expect log line **`shell ready (unified)`**. See [docs/DEMO.md](docs/DEMO.md).

---

## Quick start — Full install

**Prerequisites:** [Rust](https://rustup.rs/), [Ollama](https://ollama.com/). Docker optional.

```powershell
git clone https://github.com/utahisnotastate/utahbrowser.git
cd utahbrowser
.\scripts\install.ps1 -KnowledgePath "C:\path\to\your\notebooks"
cd dist
.\UtahBrowser.cmd
```

---

## Project layout

```
src/              # Rust: engine, truth, sentinel, vault, browser core
assets/ui/        # Shell HTML, CSS, compositor JS
config/           # default.toml, demo.toml
ghost-link/       # Optional sensory daemon (Python)
urm/              # Optional Nexus orchestrator (Python)
scripts/          # Build-Demo.ps1, install.ps1, deploy_sovereign.ps1
docs/             # User and technical documentation
```

Runtime logs: `%APPDATA%\UtahBrowser\logs\browser.log`

---

## Support

PayPal: [utah@utahcreates.com](mailto:utah@utahcreates.com)

---

## License

MIT — see [LICENSE](LICENSE).
