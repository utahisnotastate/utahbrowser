# Repository scope

**This public repository contains Utah Browser only.**

[github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser) is the open-source desktop browser: Rust + Wry (WebView2), local Truth Engine, unified compositor shell, and optional developer subsystems (Ghost-Link, URM) documented separately.

## What belongs here

| In scope | Examples |
|----------|----------|
| Browser binary and launcher | `src/`, `utah-browser.exe`, `UtahBrowser.cmd` |
| Shell UI assets | `assets/ui/` |
| Install and build scripts | `scripts/Build-Demo.ps1`, `install.ps1` |
| Browser configuration | `config/default.toml`, `config/demo.toml` |
| Browser documentation | `docs/` |

## What does **not** belong here

- Separate applications (games, gambling, poker, or other private products)
- Personal notebook corpora or secrets (use `UTAH_KNOWLEDGE_PATH` locally; never commit)
- Built `dist/` or release zips (gitignored)
- Experimental Python prototypes not wired into the browser (see [ROADMAP.md](ROADMAP.md))

If you maintain other projects under the same GitHub organization, keep them in **their own repositories** with their own README and issues.

## Optional subsystems (in-repo, not required to browse)

These folders ship with the monorepo for advanced setups but are **off by default**:

| Folder | Purpose | Default |
|--------|---------|---------|
| `ghost-link/` | Webcam/mic sensory daemon | Off in `demo.toml` |
| `urm/` | Nexus orchestrator scaffold | Off in demo; optional install |
| `scripts/Lazarus-Daemon.ps1` | Dev relaunch loop | Manual only |
| `scripts/Millennium-Pipeline.ps1` | Periodic `cargo update` helper | Manual only |

They are **not** gambling or poker software and are unrelated to any private apps you may host elsewhere.

## Knowledge path default

`config/default.toml` uses a neutral public Documents path. Override on your machine:

```powershell
$env:UTAH_KNOWLEDGE_PATH = "D:\MyNotes"
.\scripts\install.ps1 -KnowledgePath $env:UTAH_KNOWLEDGE_PATH
```

Do not point the public repo config at private project directories.

## GitHub repository description (maintainers)

Suggested **About** text for this repo on GitHub:

> Privacy-first Windows browser (Rust + WebView2). Local Truth Engine with Ollama + Qdrant. Unified compositor shell. No cloud telemetry.

Do not use the About field to advertise unrelated private applications.
