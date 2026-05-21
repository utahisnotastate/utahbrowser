# Utah Browser documentation

**Project:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

Utah Browser is a privacy-first desktop shell (Rust + Wry) with a local **Truth Engine**: your notebooks, [Ollama](https://ollama.com/), and [Qdrant](https://qdrant.tech/) — no cloud APIs in the app layer.

---

## Choose your guide

| Audience | Guide | Topics |
|----------|-------|--------|
| **Kids & families** | [For kids](guides/FOR_KIDS.md) | What the browser does, Truth Guard in simple words |
| **Everyone (non-technical)** | [Easy start](guides/FOR_EVERYONE.md) | Install, launch, daily use |
| **Developers & IT** | [Technical manual](technical/MANUAL.md) | Architecture, IPC, config, source layout |
| **[Installation](INSTALLATION.md)** | Full install walkthrough | Zero-Click Kernel phases, flags, outputs |
| **[Qdrant & services](guides/QDRANT_AND_SERVICES.md)** | Ollama + Qdrant | Native binary (no Docker), auto-start, logs |
| **Build errors (Windows)** | [Build troubleshooting](guides/BUILD_TROUBLESHOOTING.md) | Rust compile, zerovec, error 4551, Qdrant |

---

## Fastest path (Windows)

```powershell
git clone https://github.com/utahisnotastate/utahbrowser.git
cd utahbrowser
.\scripts\install.ps1 -KnowledgePath "C:\path\to\your\notebooks"
.\dist\Launch-UtahBrowser.ps1
```

**You need:** [Rust](https://rustup.rs/), [Ollama](https://ollama.com/), internet once for Qdrant binary download. **Docker is optional.**

---

## Helper scripts

| Script | Purpose |
|--------|---------|
| `scripts/install.ps1` | Build, health checks, model pull, package `dist/` |
| `scripts/Ensure-Qdrant.ps1` | Start or install native Qdrant only |
| `scripts/Ensure-Services.ps1` | Ollama check + Qdrant ensure (used by launcher) |
| `scripts/Repair-BuildEnvironment.ps1` | Fix Cargo cache / Defender exclusions |

---

## What is *not* required

| Tool | Notes |
|------|--------|
| **Docker Desktop** | Optional fallback only; native Qdrant is default |
| **[Utahnetes](https://github.com/utahisnotastate/utahnetes)** | Separate LAN swarm project; does not replace Qdrant |
| **Chromium / Electron** | Utah Browser uses Wry (system WebView2 on Windows) |

---

## License

MIT — see [LICENSE](../LICENSE).
