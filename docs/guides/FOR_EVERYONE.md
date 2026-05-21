# Utah Browser — easy start (non-technical)

For **parents, teachers, and everyday users**. No programming background needed.

**Project:** [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

---

## What you get

**Utah Browser** is a private web browser for your PC that:

- Keeps browsing and notes **on your machine** (offline-first)
- Does **not** phone home to ad companies for Truth Guard
- Includes **Truth Guard** — checks statements against *your* notebook files

It uses the same kind of web engine Windows already has (WebView2), wrapped in a small Rust app—not a giant Chromium download.

---

## What you need (one-time)

| Tool | Purpose | Get it |
|------|---------|--------|
| **Rust** | Builds the browser once | [rustup.rs](https://rustup.rs/) |
| **Ollama** | Local AI on your PC | [ollama.com](https://ollama.com/) |

**You do not need Docker.** The installer downloads **Qdrant** (the notebook search database) as a normal Windows program and starts it for you. Internet is needed **once** for that download (~50 MB).

Detailed steps: [Installation guide](../INSTALLATION.md) · [Qdrant & services](QDRANT_AND_SERVICES.md)

---

## Install (ask a technical helper or follow along)

```powershell
cd C:\code\utahbrowser
.\scripts\install.ps1 -KnowledgePath "C:\path\to\your\notebooks"
```

Use your real notes folder instead of `C:\path\to\your\notebooks` (for example `C:\knowledgebase`).

When it finishes:

```powershell
.\dist\Launch-UtahBrowser.ps1
```

The launcher checks Ollama and Qdrant before opening the window.

---

## Point Truth Guard at your notebooks

Supported files: `.md`, `.txt`, `.markdown`, `.pdf` in one folder.

```powershell
.\scripts\install.ps1 -KnowledgePath "D:\MyNotes"
```

Or set before launch:

```powershell
$env:UTAH_KNOWLEDGE_PATH = "D:\MyNotes"
.\dist\Launch-UtahBrowser.ps1
```

---

## Daily use

1. **Browse** — type a web address, press **Go**.
2. **Ingest** — **Tools** → **Ingest Notebooks** (run again after adding new files).
3. **Verify** — paste text in **Verify statement** → **Verify** → read **Truth HUD**.

### Status lines

| Line | Meaning |
|------|---------|
| **Ollama: online** | Local AI is running |
| **Qdrant: online** | Notebook search database is running |
| **Knowledge** | Which folder is configured |
| **Chunks** | How many notebook pieces were indexed |

---

## Works on any screen size

The layout adapts to small laptops, large monitors, and ultrawide displays. Toggle **Tools** to hide the side panel for more browsing space.

---

## Privacy (plain language)

- Truth Guard uses **your** files on **your** PC
- No cloud notebook upload is built into the app
- You can browse without ingesting, but fact-checking needs Ollama + Qdrant + notebooks

---

## When something goes wrong

| Problem | What to do |
|---------|------------|
| Install failed while **building** | [Build troubleshooting](BUILD_TROUBLESHOOTING.md) or `.\scripts\Repair-BuildEnvironment.ps1` |
| **Qdrant** failed | `.\scripts\Ensure-Qdrant.ps1` then [Qdrant guide](QDRANT_AND_SERVICES.md) |
| **Ollama** offline | Open Ollama app or run `ollama serve` |
| No notebooks found | Check the folder path exists and has supported files |
| Verify always fails | Run **Ingest Notebooks** again |

```powershell
git pull
.\scripts\install.ps1 -KnowledgePath "C:\knowledgebase"
```

---

## More reading

- Kids: [For kids](FOR_KIDS.md)
- Install details: [Installation](../INSTALLATION.md)
- Developers: [Technical manual](../technical/MANUAL.md)
- All docs: [docs/README.md](../README.md)
