# Utah Browser documentation

**Project:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

Privacy-first Windows browser (Rust + WebView2) with a local **Truth Engine** (Ollama + Qdrant). No cloud APIs in the application layer.

---

## Start here

| I want to… | Read this |
|------------|-----------|
| **Run a stable demo in 3 steps** | **[DEMO.md](DEMO.md)** → `Build-Demo.ps1` → `UtahBrowser.cmd` |
| Understand the product | [OVERVIEW.md](OVERVIEW.md) |
| Install everything (dev) | [INSTALLATION.md](INSTALLATION.md) |
| Configure paths and flags | [CONFIGURATION.md](CONFIGURATION.md) |
| What belongs in this repo | [REPOSITORY_SCOPE.md](REPOSITORY_SCOPE.md) |
| Shipped vs planned features | [ROADMAP.md](ROADMAP.md) |
| Unified compositor + Phase 3 | [technical/UNIFIED_COMPOSITOR_AND_PHASE3.md](technical/UNIFIED_COMPOSITOR_AND_PHASE3.md) |
| Fix launch crashes | [technical/CRASH_ON_LAUNCH_REPORT.md](technical/CRASH_ON_LAUNCH_REPORT.md) |
| Hand off to another engineer / AI | [technical/SYSTEM_ARCHITECTURE_REPORT.md](technical/SYSTEM_ARCHITECTURE_REPORT.md) |
| Publish a demo release | [DEMO_RELEASE.md](DEMO_RELEASE.md) |

---

## Guides by audience

| Audience | Guide |
|----------|-------|
| Kids & families | [For kids](guides/FOR_KIDS.md) |
| Everyday users | [Easy start](guides/FOR_EVERYONE.md) |
| Ollama & Qdrant | [QDRANT_AND_SERVICES.md](guides/QDRANT_AND_SERVICES.md) |
| Ghost-Link (optional) | [GHOST_LINK.md](guides/GHOST_LINK.md) |
| URM / Nexus (optional) | [URM_NEXUS.md](guides/URM_NEXUS.md) |
| Calibration console | [CALIBRATION_CONSOLE.md](guides/CALIBRATION_CONSOLE.md) |
| Sovereign stack summary | [SOVEREIGN_INTELLIGENCE_STACK.md](SOVEREIGN_INTELLIGENCE_STACK.md) |
| Developers | [Technical manual](technical/MANUAL.md) |
| Build errors | [BUILD_TROUBLESHOOTING.md](guides/BUILD_TROUBLESHOOTING.md) |

---

## Build scripts

| Script | Output |
|--------|--------|
| `scripts/Build-Demo.ps1` | Stable **demo** `dist/` (unified compositor, `demo.toml`) |
| `scripts/Build-Standalone.ps1` | Full **dev** `dist/` (latest `main`) |
| `scripts/install.ps1` | Zero-Click Kernel (Ollama + Qdrant + build) |
| `scripts/deploy_sovereign.ps1` | Install + optional Ghost-Link + standalone |

---

## Launch (Windows)

```powershell
cd dist
.\UtahBrowser.cmd
```

Use **`UtahBrowser.cmd`** — not `Utah Browser.exe` unquoted in PowerShell (the space breaks the command).

**Logs:** `%APPDATA%\UtahBrowser\logs\browser.log`

---

## Technical index

| Topic | Document |
|-------|----------|
| Developer reference (shorter) | [technical/MANUAL.md](technical/MANUAL.md) |
| Full architecture | [technical/SYSTEM_ARCHITECTURE_REPORT.md](technical/SYSTEM_ARCHITECTURE_REPORT.md) |
| Compositor & prefetch | [technical/UNIFIED_COMPOSITOR_AND_PHASE3.md](technical/UNIFIED_COMPOSITOR_AND_PHASE3.md) |
| Crash investigation | [technical/CRASH_ON_LAUNCH_REPORT.md](technical/CRASH_ON_LAUNCH_REPORT.md) |
