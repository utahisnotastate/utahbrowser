# SOTA Installation Guide: The Zero-Click Kernel

**Welcome to the Utah-Omega-23 Deployment Matrix.**

This guide details the procedure for deploying the Utah Browser Sovereign Workstation. We utilize a **Zero-Click Kernel** (`scripts/install.ps1`) to orchestrate the environment, services, and compilation in a single execution.

---

## 1. Prerequisites

Before initiating the kernel, ensure your host machine meets the following cryptographic and operational standards:

| Requirement | Purpose | Source |
|-------------|---------|--------|
| **Windows 10/11** | Core OS with WebView2 Runtime | Pre-installed |
| **Rust Toolchain** | Native performance & memory safety | [rustup.rs](https://rustup.rs/) |
| **Ollama** | Local Large Language Model (LLM) host | [ollama.com](https://ollama.com/) |
| **Git** | Source control and P2P updates | [git-scm.com](https://git-scm.com/) |

**Note:** Unlike inferior browsers, Utah **does not require Docker**. We utilize a native Windows binary for Qdrant to ensure zero-latency local vector search.

---

## 2. Standard Installation (Recommended)

Execute the following commands in an elevated PowerShell terminal to initialize the workstation:

```powershell
# Clone the sovereign repository
git clone https://github.com/utahisnotastate/utahbrowser.git
cd utahbrowser

# Execute the Zero-Click Kernel
# Optional: Use -KnowledgePath to point Truth Guard to your existing notebooks
.\scripts\install.ps1 -KnowledgePath "C:\MyNotebooks"
```

### What the Kernel Orchestrates:
1. **Ollama Health Check:** Verifies the local LLM server is responding.
2. **Qdrant Bootstrap:** Downloads the SOTA native Windows binary, initializes the database, and creates the sovereign history collection.
3. **Model Pull:** Automatically retrieves the required embedding and chat models (`nomic-embed-text`, `llama3`, etc.).
4. **Rust Compilation:** Executes a high-performance release build with optimized LTO.
5. **Distribution Assembly:** Packages the binaries, assets, and configurations into the `dist/` folder.

---

## 3. Specialized Build Targets

Depending on your operational requirements, choose the appropriate build script:

| Goal | Script | Outcome |
|------|--------|---------|
| **Stable Demo** | `.\scripts\Build-Demo.ps1` | A safe, self-contained demo environment with pre-set configurations. |
| **Full Workstation** | `.\scripts\Build-Standalone.ps1` | The complete SOTA build with all experimental features enabled. |
| **Enterprise Deployment** | `.\scripts\deploy_sovereign.ps1` | A portable, zero-dependency package ready for distribution. |

---

## 4. Post-Installation Launch

Navigate to the distribution directory and execute the master launcher:

```powershell
cd dist
.\UtahBrowser.cmd
```

**Launch Options:**
- `.\UtahBrowser.cmd`: Standard optimized launcher.
- `.\Launch-UtahBrowser.ps1`: PowerShell-native launcher with advanced logging.
- `cargo run --release`: For real-time development and debugging.

---

## 5. Directory Structure & Logs

| Path | Description |
|------|-------------|
| `dist/` | The compiled, portable workstation. |
| `%LOCALAPPDATA%\UtahBrowser\qdrant\` | Native Qdrant binary and local vector data. |
| `%APPDATA%\UtahBrowser\logs\` | Runtime diagnostics and IPC nexus logs. |
| `%APPDATA%\UtahBrowser\extensions\` | Your vibe-coded WASM extensions. |

---

## 6. Troubleshooting

| Issue | Resolution |
|-------|------------|
| **Rust Build Error** | Run `.\scripts\Repair-BuildEnvironment.ps1` to clear caches. |
| **Ollama Offline** | Ensure the Ollama tray icon is visible or run `ollama serve`. |
| **Qdrant Connection Failed** | Run `.\scripts\Ensure-Qdrant.ps1` to re-initialize the native service. |
| **UI Not Rendering** | Ensure Microsoft Edge WebView2 Runtime is up to date. |

For advanced diagnostics, consult [docs/technical/CRASH_ON_LAUNCH_REPORT.md](technical/CRASH_ON_LAUNCH_REPORT.md).

---

**Mastery Log:** Installation matrix synchronized. You are now ready to reclaim your sovereignty.
