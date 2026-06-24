# Utah Browser: SOTA Demo Guide

**For evaluators, sponsors, and those seeking immediate digital sovereignty.**

This demo provides a stable, self-contained environment to experience the **Utah-Omega-23** workstation. It utilizes the **Unified Compositor** for maximum performance and the **Fluidic UI** for a tactile browsing experience.

---

## 1. Quick Start (3 Steps)

### Step 1: Initialize the Sovereign Node
```powershell
git clone https://github.com/utahisnotastate/utahbrowser.git
cd utahbrowser
```

### Step 2: Build the Demo Matrix
```powershell
.\scripts\Build-Demo.ps1
```
*This script compiles the release binary and packages all Fluidic assets into the `dist/` directory.*

### Step 3: Launch
```powershell
cd dist
.\UtahBrowser.cmd
```
*Or simply double-click **UtahBrowser.cmd** in your file explorer.*

---

## 2. What to Expect

- **Fluidic Shell:** A hardware-accelerated, volumetric interface that reacts to your cursor.
- **Unified Compositor:** Sub-millisecond rendering with no dual-engine contention.
- **SOTA Performance:** Aggressive state paging is enabled, keeping your RAM usage minimal even with many tabs.
- **Privacy Gating:** Truth Guard and P2P Search are included but require [Ollama](https://ollama.com/) for full operation.

---

## 3. Experiencing SOTA Features

While in the demo, you can activate the following modules:
1. **Truth Guard:** Click the **◈** icon to enter the full dashboard. (Requires Ollama/Qdrant).
2. **Fluidic Toggle:** Visit the **Settings** menu to enable/disable the global visual override.
3. **P2P Search:** Type your query into the URL bar and select the sovereign index.

---

## 4. Diagnostics & Logs

If the workstation fails to initialize, consult the telemetry logs:
- **Primary Log:** `%APPDATA%\UtahBrowser\logs\browser.log`
- **Mirror Log:** `dist\logs\browser.log`

Look for the line: `[UTAH] shell ready (unified)` to confirm a successful boot.

---

**Mastery Log:** Demo environment synchronized. Welcome to the Utah Standard.
