# Utah Browser // V1.0-GENESIS

[![Repository](https://img.shields.io/badge/GitHub-utahisnotastate%2Futahbrowser-blue)](https://github.com/utahisnotastate/utahbrowser)

Privacy-first, offline-native desktop browser with a local **Truth Engine** (your notebooks + Ollama + Qdrant). Built in Rust with [Wry](https://github.com/tauri-apps/wry) (system WebView2 on Windows—not Chromium/Electron); UI uses **Utah.css** (layout state in CSS, not app logic scripts).

**Repository:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

---

## Documentation

| Who you are | Start here |
|-------------|------------|
| Kids & families | [Guide for kids](docs/guides/FOR_KIDS.md) |
| Everyday users | [Easy start](docs/guides/FOR_EVERYONE.md) |
| **Installation** | [Install guide](docs/INSTALLATION.md) |
| **Ollama & Qdrant** | [Services guide](docs/guides/QDRANT_AND_SERVICES.md) |
| Developers & IT | [Technical manual](docs/technical/MANUAL.md) |
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

## Quick start (Windows)

**Prerequisites:** [Rust](https://rustup.rs/), [Ollama](https://ollama.com/). **Docker optional.** Internet once for Qdrant binary download.

```powershell
git clone https://github.com/utahisnotastate/utahbrowser.git
cd utahbrowser
.\scripts\install.ps1 -KnowledgePath "C:\path\to\your\notebooks"
.\dist\Launch-UtahBrowser.ps1
```

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

**Outputs:** `dist/utah-browser.exe`, `Launch-UtahBrowser.ps1`, `health-report.json`, `utah.env.ps1`

**Helpers:** `Ensure-Qdrant.ps1`, `Ensure-Services.ps1`, `Repair-BuildEnvironment.ps1`

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
src/              # Rust: engine, truth, audio, evolution
assets/ui/        # Shell HTML, Utah.css, transport.js (IPC only)
config/           # default.toml
scripts/          # install.ps1, Ensure-*.ps1, kernel/
docs/             # guides for all audiences
```

---

## License

MIT — see [LICENSE](LICENSE).
