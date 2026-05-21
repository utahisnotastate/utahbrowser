# Utah Browser — Demo (stable build)

**For evaluators, sponsors, and anyone who just wants it to run.**  
Repository: [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

The **demo profile** uses a simplified config and **safe mode** (single WebView) for maximum stability on Windows. Full features (dual chrome + dashboard strip) remain on the `main` branch for developers.

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
- In demo mode the app uses **single WebView (safe mode)** — reliable browsing without the dual chrome strip

To try the full Utah dashboard UI later, delete safe mode state and restart without demo mode (see [DEMO_RELEASE.md](DEMO_RELEASE.md)).

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

| Track | Branch / tag | Launch | UI |
|-------|----------------|--------|-----|
| **Demo (stable)** | `demo/v1.0` or `Build-Demo.ps1` | `UtahBrowser.cmd` | Single WebView safe mode |
| **Latest (main)** | `main` | `UtahBrowser.cmd` after `Build-Standalone.ps1` | Dual WebView + dashboard |

See [DEMO_RELEASE.md](DEMO_RELEASE.md) for how maintainers cut demo releases.

---

## Support

PayPal: **utah@utahcreates.com** (see [README](../README.md)).
