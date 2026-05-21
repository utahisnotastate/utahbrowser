# Utah Browser — easy start (non-technical)

This guide is for **parents, teachers, and everyday users**. No programming experience needed.

## What you get

**Utah Browser** is a private web browser for your PC. It:

- Stays **offline-first** — your notes and checks stay on your machine
- Does **not** send your browsing to advertising companies
- Includes **Truth Guard** — compares what you read or hear against *your* notebook files (documents you trust)

Official project: [github.com/utahisnotastate/utahbrowser](https://github.com/utahisnotastate/utahbrowser)

## Before you start (one-time setup)

Ask someone comfortable with computers to install these **free** tools:

| Tool | What it does | Get it |
|------|----------------|--------|
| **Rust** | Builds the browser | [rustup.rs](https://rustup.rs/) |
| **Ollama** | Runs AI on your PC (no cloud) | [ollama.com](https://ollama.com/) |
| **Docker** (optional) | Runs the notebook search database | [docker.com](https://www.docker.com/) |

Then run the **Zero-Click installer** (automates health checks and downloads):

```powershell
cd C:\code\utahbrowser
.\scripts\install.ps1 -KnowledgePath "C:\path\to\your\notebooks"
```

When it finishes, open the app:

```powershell
.\dist\Launch-UtahBrowser.ps1
```

## Point Truth Guard at your notebooks

Your notebooks are a folder of files the browser trusts — for example markdown, text, or PDF notes.

Set the folder path when installing:

```powershell
.\scripts\install.ps1 -KnowledgePath "D:\MyNotes"
```

Or set it before each launch:

```powershell
$env:UTAH_KNOWLEDGE_PATH = "D:\MyNotes"
.\dist\Launch-UtahBrowser.ps1
```

## Daily use (3 steps)

1. **Go to a website** — type the address in the bar and press **Go**. The page opens in the large area.
2. **Load your notebooks** — open **Tools** → **Ingest Notebooks** (do this again when you add new files).
3. **Check a claim** — paste text into **Verify statement** → **Verify**. Read the result in **Truth HUD**.

### What the status lines mean

| Line | Meaning |
|------|---------|
| **Ollama: online** | AI helper is running on your PC |
| **Qdrant: online** | Notebook search database is running |
| **Knowledge** | Folder path being used for your files |
| **Chunks** | How many pieces of your notebooks were indexed |

If Ollama or Qdrant says **offline**, run the installer again or ask your helper to start those services.

## Works on any screen size

The layout adjusts automatically for small laptops, large monitors, and ultrawide screens. Use **Tools** to hide the side panel if you want more space for the website.

## Privacy in plain language

- Browsing and notebook content stay **local**
- Truth Guard uses **your** files — not a company’s cloud memory
- You can use the browser **without** turning on Truth Guard (but fact-checking needs notebooks + Ollama + Qdrant)

## When something goes wrong

| Problem | Try this |
|---------|----------|
| **Install failed while building** | [Build troubleshooting](BUILD_TROUBLESHOOTING.md) or `.\scripts\Repair-BuildEnvironment.ps1` |
| Installer says Ollama offline | Open Ollama app or run `ollama serve` |
| Qdrant offline | Start Docker, or run installer without `-SkipQdrantStart` |
| No notebooks found | Check `UTAH_KNOWLEDGE_PATH` points to a real folder |
| Verify always fails | Run **Ingest Notebooks** again after adding files |

## More detail

- Kids: [For kids](FOR_KIDS.md)
- Developers: [Technical manual](../technical/MANUAL.md)
- Full README: [README.md](../../README.md)
