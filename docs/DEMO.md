# Utah Browser — Demo (stable build)

**For evaluators, sponsors, and anyone who just wants it to run.**  
Repository: [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

The **demo profile** uses a simplified config and the **unified compositor** (one WebView2: Ghost-Chrome frame + content iframe). Legacy dual-WebView mode is opt-in only (`UTAH_LEGACY_DUAL=1`).

---

## Requirements (demo)

| Item | Required? |
|------|-----------|
| Windows 10/11 + **WebView2** | Yes |
| **Rust** (only if building from source) | Build path only |
| **Ollama** | Optional (Truth Guard needs it) |
| **Qdrant** | Optional (Truth Guard needs it) |
| Docker | No |

---

## Fastest demo (3 steps)

### 1. Clone

```powershell
git clone https://github.com/utahisnotastate/utahbrowser.git
cd utahbrowser
```

### 2. Build the demo package

```powershell
.\scripts\Build-Demo.ps1
```

This produces `dist\` with:

- `UtahBrowser.cmd` — **double-click this** (no PowerShell quoting issues)
- `utah-browser.exe`
- Demo `config\default.toml` (evolution off, Ghost-Link off)
- `DEMO.txt` — quick reference in the folder

### 3. Launch

```powershell
cd dist
.\UtahBrowser.cmd
```

Or double-click **`UtahBrowser.cmd`** in File Explorer.

**Do not** run `Utah Browser.exe` without quotes in PowerShell — the space breaks the command. Use `UtahBrowser.cmd` instead.

---

## What you should see

- A window titled **Utah Browser // DEMO**
- **Google** (or your configured home page) in the main view
- **Ghost-Chrome** tab strip and URL bar in the same window (no second WebView engine)
- Log line should read **`shell ready (unified)`**

Use the **◈** button (or dashboard IPC) for the full Utah app shell (`index.html`). Legacy dual HWND mode: set `UTAH_LEGACY_DUAL=1` before launch (see [DEMO_RELEASE.md](DEMO_RELEASE.md)).

---

## Logs (if something fails)

| Log | Path |
|-----|------|
| Primary | `%APPDATA%\UtahBrowser\logs\browser.log` |
| Mirror | `dist\logs\browser.log` (after run) |
| Temp | `%TEMP%\utah-browser.log` |

Look for: `shell ready`, `boot FAILED`, `PANIC`.

---

## Optional: Truth Guard

1. Install [Ollama](https://ollama.com/) and run `ollama serve`
2. Run once: `.\scripts\Ensure-Services.ps1 -ProjectRoot .\dist`
3. In the browser, open the Truth view (full UI build) or use developer `main` branch

---

## Demo vs development

| Track | Command | Launch | UI |
|-------|---------|--------|-----|
| **Demo (stable)** | `Build-Demo.ps1` | `UtahBrowser.cmd` | Unified compositor + demo.toml |
| **Latest (main)** | `Build-Standalone.ps1` | `UtahBrowser.cmd` | Unified compositor + full config |

See [DEMO_RELEASE.md](DEMO_RELEASE.md) for maintainer release steps.

---

## Support

PayPal: **utah@utahcreates.com** (see [README](../README.md)).
