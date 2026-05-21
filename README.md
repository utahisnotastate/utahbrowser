# Utah Browser // V1.0-GENESIS

[![Repository](https://img.shields.io/badge/GitHub-utahisnotastate%2Futahbrowser-blue)](https://github.com/utahisnotastate/utahbrowser)

Privacy-first, offline-native desktop browser with a local **Truth Engine** (your notebooks + Ollama + Qdrant). Built in Rust with [Wry](https://github.com/tauri-apps/wry); UI uses **Utah.css** (no JavaScript for layout state).

## Documentation

| Who you are | Start here |
|-------------|------------|
| Kids & families | [Guide for kids](docs/guides/FOR_KIDS.md) |
| Everyday users | [Easy start (non-technical)](docs/guides/FOR_EVERYONE.md) |
| Developers & IT | [Technical manual](docs/technical/MANUAL.md) |
| Build errors (Windows) | [Build troubleshooting](docs/guides/BUILD_TROUBLESHOOTING.md) |
| All docs | [docs/README.md](docs/README.md) |

## Features

- Native shell (no Electron) — memory-safe Rust backend
- **Truth Guard** — ingest PDF/Markdown/text notebooks, verify claims locally
- **Zero-Click Kernel** — installer with Ollama/Qdrant health checks and model auto-pull
- Responsive UI — phones to ultrawide monitors
- No telemetry, no cloud APIs in the application layer

## Quick start

**Prerequisites:** [Rust](https://rustup.rs/), [Ollama](https://ollama.com/), [Qdrant](https://qdrant.tech/) (Docker: `docker run -p 6333:6333 qdrant/qdrant`)

```powershell
git clone https://github.com/utahisnotastate/utahbrowser.git
cd utahbrowser
.\scripts\install.ps1 -KnowledgePath "C:\path\to\your\notebooks"
.\dist\Launch-UtahBrowser.ps1
```

## Zero-Click Kernel

```powershell
.\scripts\install.ps1 -KnowledgePath "C:\path\to\utahisnotastate"
```

| Flag | Effect |
|------|--------|
| `-SkipBuild` | Health + pull only |
| `-SkipHealth` | Build/package only |
| `-SkipPull` | Skip `ollama pull` |
| `-SkipQdrantStart` | Do not auto-start Docker `utah-qdrant` |
| `-ForcePull` | Re-pull models even if present |
| `-RepairOnly` | Fix Cargo cache and rebuild only |

Outputs: `dist/utah-browser.exe`, `health-report.json`, `Launch-UtahBrowser.ps1`, `utah.env.ps1`.

**Build failed (zerovec / error 4551)?** Run `.\scripts\Repair-BuildEnvironment.ps1` or see [BUILD_TROUBLESHOOTING.md](docs/guides/BUILD_TROUBLESHOOTING.md).

## Configuration

Edit `config/default.toml` or set:

| Variable | Effect |
|----------|--------|
| `UTAH_KNOWLEDGE_PATH` | Notebook corpus directory |
| `OLLAMA_HOST` | Ollama API base URL |
| `QDRANT_URL` | Qdrant REST URL |

## Project layout

```
src/           # Rust engine, truth, audio, evolution
assets/ui/     # Shell HTML, Utah.css, transport.js
config/        # default.toml
scripts/       # install.ps1, kernel/
docs/          # guides for kids, everyone, and developers
```

## License

MIT — see [LICENSE](LICENSE).
