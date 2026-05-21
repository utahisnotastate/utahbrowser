# Demo release process (maintainers)

How to publish a **stable demo** snapshot on [GitHub](https://github.com/utahisnotastate/utahbrowser) that anyone can run without reading the full dev docs.

---

## Version lines

| Line | Git ref | Purpose |
|------|---------|---------|
| **Development** | `main` | Sentinel-Core, dual WebView, Ghost-Link, vault APIs |
| **Demo stable** | `demo/v1.0` branch + tag `v1.0-demo` | Safe-mode profile, minimal moving parts |

The demo is intentionally a **simpler runtime path** (not an old commit frozen in time unless you choose to tag one).

---

## Cut a demo release (Windows)

```powershell
cd C:\code\utahbrowser
git checkout main
git pull

# Build demo dist/
.\scripts\Build-Demo.ps1

# Smoke test
cd dist
.\UtahBrowser.cmd
# Confirm %APPDATA%\UtahBrowser\logs\browser.log shows "demo mode" or "single-safe"

# Commit demo tooling on main (if not already)
git add -A
git commit -m "Release: demo build profile and documentation"
git push origin main

# Publish demo branch (optional snapshot)
git checkout -B demo/v1.0
git push -u origin demo/v1.0 --force-with-lease

# Annotated tag for GitHub Releases UI
git tag -a v1.0-demo -m "Stable demo: safe mode, UtahBrowser.cmd, sovereign logs"
git push origin v1.0-demo
```

---

## GitHub Release (manual)

1. Open [Releases](https://github.com/utahisnotastate/utahbrowser/releases) → **Draft new release**
2. Choose tag **`v1.0-demo`**
3. Title: **Utah Browser v1.0 Demo (Windows)**
4. Attach zip: run `.\scripts\Build-Demo.ps1 -Zip` → upload `release\UtahBrowser-Demo-win64.zip`
5. Paste release notes from `docs/DEMO.md` (Requirements + 3 steps)

---

## What `Build-Demo.ps1` does

1. `cargo build --release` (`utah-browser`, `utah-launch`)
2. Copies `config/demo.toml` → `dist/config/default.toml`
3. Sets `UTAH_DEMO_MODE=1` in `UtahBrowser.cmd` (forces safe mode)
4. Copies assets, scripts/kernel, launchers (`UtahBrowser.cmd`, `UtahBrowser.exe`)
5. Writes `dist/DEMO.txt`
6. Optional `-Zip` → `release/UtahBrowser-Demo-win64.zip`

---

## Demo environment variables

| Variable | Effect |
|----------|--------|
| `UTAH_DEMO_MODE=1` | Force single WebView safe mode |
| `UTAH_BROWSER_HOME` | Set by launcher to `dist` folder |
| `UTAH_KNOWLEDGE_PATH` | Override notebook path |

---

## Document map (updated stack)

| Doc | Audience |
|-----|----------|
| [DEMO.md](DEMO.md) | Anyone running the demo |
| [INSTALLATION.md](INSTALLATION.md) | Full Zero-Click install |
| [SOVEREIGN_INTELLIGENCE_STACK.md](SOVEREIGN_INTELLIGENCE_STACK.md) | Architecture / Ghost-Link / vault |
| [technical/CRASH_ON_LAUNCH_REPORT.md](technical/CRASH_ON_LAUNCH_REPORT.md) | Launch failures |
| [technical/SYSTEM_ARCHITECTURE_REPORT.md](technical/SYSTEM_ARCHITECTURE_REPORT.md) | Full system design |
