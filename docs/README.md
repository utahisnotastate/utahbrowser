# Utah Browser documentation

**Project:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

Privacy-first desktop shell (Rust + Wry) with a local **Truth Engine**: notebooks, [Ollama](https://ollama.com/), [Qdrant](https://qdrant.tech/) — no cloud APIs in the app layer.

---

## Start here

| I want to… | Read this |
|------------|-----------|
| **Run a stable demo now** | **[DEMO.md](DEMO.md)** → `Build-Demo.ps1` → `UtahBrowser.cmd` |
| Install everything (dev) | [INSTALLATION.md](INSTALLATION.md) |
| Understand the full stack | [SOVEREIGN_INTELLIGENCE_STACK.md](SOVEREIGN_INTELLIGENCE_STACK.md) |
| Fix launch crashes | [technical/CRASH_ON_LAUNCH_REPORT.md](technical/CRASH_ON_LAUNCH_REPORT.md) |
| Hand off to another engineer / AI | [technical/SYSTEM_ARCHITECTURE_REPORT.md](technical/SYSTEM_ARCHITECTURE_REPORT.md) |
| Publish a demo release | [DEMO_RELEASE.md](DEMO_RELEASE.md) |

---

## Guides by audience

| Audience | Guide |
|----------|-------|
| Kids & families | [For kids](guides/FOR_KIDS.md) |
| Everyone | [Easy start](guides/FOR_EVERYONE.md) |
| Ollama & Qdrant | [QDRANT_AND_SERVICES.md](guides/QDRANT_AND_SERVICES.md) |
| Ghost-Link peripherals | [GHOST_LINK.md](guides/GHOST_LINK.md) |
| URM / Nexus | [URM_NEXUS.md](guides/URM_NEXUS.md) |
| Calibration console | [CALIBRATION_CONSOLE.md](guides/CALIBRATION_CONSOLE.md) |
| Developers | [Technical manual](technical/MANUAL.md) |
| Build errors | [BUILD_TROUBLESHOOTING.md](guides/BUILD_TROUBLESHOOTING.md) |

---

## Build scripts

| Script | Output |
|--------|--------|
| `scripts/Build-Demo.ps1` | Stable **demo** `dist/` (safe mode, `demo.toml`) |
| `scripts/Build-Standalone.ps1` | Full **dev** `dist/` (latest main features) |
| `scripts/deploy_sovereign.ps1` | Install + Ghost-Link + standalone |
| `scripts/install.ps1` | Zero-Click Kernel (full install) |

---

## Launch (Windows)

```powershell
cd dist
.\UtahBrowser.cmd
```

**Never** type `.\Utah Browser.exe` in PowerShell without quotes — use `UtahBrowser.cmd`.

---

## License

MIT — [LICENSE](../LICENSE)
