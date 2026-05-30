<div align="center">

<!-- Brand row: Chief Spy centered -->
<table>
  <tr>
    <td align="center"><img src="assets/branding/utah-occult-agency.png" alt="Utah Occult Agency" width="110" /></td>
    <td align="center"><img src="assets/branding/utah-intelligence-agency.png" alt="Utah Intelligence Agency" width="110" /></td>
    <td align="center"><img src="assets/branding/chief-spy.png" alt="Chief Spy" width="150" /></td>
    <td align="center"><img src="assets/branding/ufw-hoodlums.png" alt="UFW Hoodlums" width="110" /></td>
    <td align="center"><img src="assets/branding/bellas-flying-squirrels.png" alt="Bella's Flying Squirrels" width="110" /></td>
  </tr>
</table>

<br />

# Utah Browser

**Privacy-first desktop browser for Windows** — Rust + WebView2, local Truth Engine, unified compositor shell.

[![GitHub](https://img.shields.io/badge/GitHub-utahisnotastate%2Futahbrowser-blue)](https://github.com/utahisnotastate/utahbrowser)

[Demo](docs/DEMO.md) · [Install](docs/INSTALLATION.md) · [Docs hub](docs/README.md) · [Overview](docs/OVERVIEW.md)

</div>

---

## RIP Google Search (1998–2026)

**Killed by the Chief Spy, Utah Hans.**  
**Browser wars won by Utah.**

Utah credits ad-industry capture of the browser stack as the primary catalyst for search-engine obsolescence—not better algorithms, but better *incentives*. When the product is the ad, the index serves the advertiser. Utah Browser inverts that contract.

---

## Why Utah Search is SOTA — the Truth-Lens paradigm

Classic search engines (Google, Bing) are **document harvesters**. They crawl the public web, rank by popularity and ad yield, and return what the platform is paid to prioritize.

**Utah Search is a semantic mesh ingestor**—local, private, and aligned to *your* corpus, not a global ad graph.

| Principle | Document harvesters | Utah Search |
|-----------|---------------------|-------------|
| **Indexing model** | Crawl the whole web | Capture geometric intent from *your* browsing and notebooks |
| **History** | Server-side logs and profiles | Private **Qdrant** vectors on **your** hardware |
| **Relevance** | SEO—keywords, backlinks, spend | **Entity alignment**—meaning matched to your indexed knowledge |
| **Answer shape** | Ten blue links | **Zero-latency synthesis** from your local semantic mesh via Truth Guard |

1. **No crawling.** We do not build a planet-scale spider. Your browser ingests what *you* choose—pages you visit and files you trust.
2. **Private vector mesh.** Search history becomes a local embedding index, not a remote behavioral dossier.
3. **Entity alignment vs. SEO.** Rankings follow semantic proximity to *your* knowledge graph, not keyword games.
4. **Zero-latency synthesis.** You query indexed intelligence on-device; Ollama + Qdrant synthesize answers from chunks you already own—no round trip through a ad-optimized ranking layer.

This is the **Truth-Lens**: search as verification against sovereign data, not discovery through a commercial filter.

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
assets/branding/  # README seals and crests (Chief Spy, UIA, etc.)
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
