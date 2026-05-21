<div align="center">

# **SPONSORSHIP & SUPPORT**

## **You can pay $150 million to be the default home page for this GitHub release.**

### **Preferred bid** for the `.exe` and paid/donation release

### **PayPal: [utah@utahcreates.com](mailto:utah@utahcreates.com)**

**I'm kind of broke — any bit helps. Thank you for supporting Utah Browser.**

</div>

---

# Utah Browser // V1.0-GENESIS

[![Repository](https://img.shields.io/badge/GitHub-utahisnotastate%2Futahbrowser-blue)](https://github.com/utahisnotastate/utahbrowser)

Privacy-first, offline-native desktop browser with a local **Truth Engine** (your notebooks + Ollama + Qdrant). Built in Rust with [Wry](https://github.com/tauri-apps/wry) (system WebView2 on Windows—not Chromium/Electron); UI uses **Utah.css** (layout state in CSS, not app logic scripts).

**Repository:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

---

## Documentation

| Who you are | Start here |
|-------------|------------|
| **Demo (stable, easy run)** | **[Demo guide](docs/DEMO.md)** — `UtahBrowser.cmd`, safe mode |
| Kids & families | [Guide for kids](docs/guides/FOR_KIDS.md) |
| Everyday users | [Easy start](docs/guides/FOR_EVERYONE.md) |
| **Installation** | [Install guide](docs/INSTALLATION.md) |
| **Ollama & Qdrant** | [Services guide](docs/guides/QDRANT_AND_SERVICES.md) |
| Sovereign stack (dev) | [Sovereign Intelligence Stack](docs/SOVEREIGN_INTELLIGENCE_STACK.md) |
| Developers & IT | [Technical manual](docs/technical/MANUAL.md) |
| Launch / crash issues | [Crash on launch report](docs/technical/CRASH_ON_LAUNCH_REPORT.md) |
| Architecture handoff | [System architecture report](docs/technical/SYSTEM_ARCHITECTURE_REPORT.md) |
| Build errors (Windows) | [Build troubleshooting](docs/guides/BUILD_TROUBLESHOOTING.md) |
| All docs | [docs/README.md](docs/README.md) |

---

## Features

- **Native shell** — Rust + Wry; no Electron bloat
- **Truth Guard** — ingest PDF/Markdown/text; verify claims against your corpus
- **Zero-Click Kernel** — one installer: Ollama health, **native Qdrant** (no Docker), model pull, release build
- **Responsive UI** — Utah.css from phone-sized windows to ultrawide monitors
- **No telemetry** — local services only in application code

---

## Quick start — Demo (anyone)

**Stable demo** — single WebView, double-click launcher, no PowerShell quoting issues:

```powershell
git clone https://github.com/utahisnotastate/utahbrowser.git
cd utahbrowser
.\scripts\Build-Demo.ps1
cd dist
.\UtahBrowser.cmd
```

See **[docs/DEMO.md](docs/DEMO.md)**. Maintainers: **[docs/DEMO_RELEASE.md](docs/DEMO_RELEASE.md)** + tag `v1.0-demo`.

---

## Quick start — Full install (developers)

**Prerequisites:** [Rust](https://rustup.rs/), [Ollama](https://ollama.com/). **Docker optional.**

```powershell
git clone https://github.com/utahisnotastate/utahbrowser.git
cd utahbrowser
.\scripts\deploy_sovereign.ps1 -KnowledgePath "C:\path\to\your\notebooks"
cd dist
.\UtahBrowser.cmd
```

Or classic installer: `.\scripts\install.ps1 -KnowledgePath "C:\path\to\notebooks"`

---

## Zero-Click Kernel

```powershell
.\scripts\install.ps1 -KnowledgePath "C:\knowledgebase"
```

| Flag | Effect |
|------|--------|
| `-KnowledgePath` | Notebook folder for Truth Guard |
| `-SkipBuild` | Health + models only |
| `-SkipHealth` | Build/package only |
| `-SkipPull` | Skip `ollama pull` |
| `-SkipQdrantStart` | Do not auto-start Qdrant (native or Docker) |
| `-ForcePull` | Re-pull Ollama models |
| `-RepairOnly` | Repair Cargo cache and rebuild |

**Outputs:** `dist/utah-browser.exe`, `UtahBrowser.cmd`, `health-report.json`

**Helpers:** `Build-Demo.ps1`, `Build-Standalone.ps1`, `deploy_sovereign.ps1`, `Ensure-Services.ps1`, `Repair-BuildEnvironment.ps1`

**Logs (runtime):** `%APPDATA%\UtahBrowser\logs\browser.log` (not inside `dist/` during normal use)

**Build failed?** [BUILD_TROUBLESHOOTING.md](docs/guides/BUILD_TROUBLESHOOTING.md)

**Qdrant issues?** [QDRANT_AND_SERVICES.md](docs/guides/QDRANT_AND_SERVICES.md)

---

## Configuration

`config/default.toml` or environment variables:

| Variable | Effect |
|----------|--------|
| `UTAH_KNOWLEDGE_PATH` | Notebook corpus directory |
| `OLLAMA_HOST` | Ollama API URL |
| `QDRANT_URL` | Qdrant REST URL |

---

## Project layout

```
src/              # Rust: engine, truth, sentinel, vault, ghost_link
assets/ui/        # Shell HTML, mockup.css, fluidic UI
config/           # default.toml, demo.toml
ghost-link/       # Python peripheral daemon
scripts/          # Build-Demo.ps1, install.ps1, deploy_sovereign.ps1
docs/             # DEMO.md, technical reports, guides
urm/              # Optional Nexus orchestrator (Python)
```

---

## License

MIT — see [LICENSE](LICENSE).
