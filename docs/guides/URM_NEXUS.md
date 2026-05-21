# Utah Unified Reality Manifold (URM)

The **Nexus Orchestrator** unifies Utah Browser, Ghost-Link, semantic vault zones, M5Stack swarm scaffold, and autonomous mutagenesis into one local nervous system.

## Components

| Domain | Role | Path |
|--------|------|------|
| Eyes | Utah Browser (Rust/Wry) | `target/release/utah-browser.exe` |
| Ears/Skin | Ghost-Link daemon | `ghost-link/` |
| Brain | ZEO vault + zones | `~/.utah_browser/vault/` |
| Hands/Feet | M5Stack swarm bus | `urm/swarm/` |
| Nexus | Integration loop | `urm/nexus_orchestrator.py` |

## Install

```powershell
.\scripts\install_urm.ps1 -StartNow -StartGhostLink
# Commercial layout:
.\scripts\install_urm.ps1 -UseProgramData -RegisterStartup -StartNow
```

Full stack:

```powershell
.\scripts\install.ps1 -InstallURM -InstallGhostLink -UrmStartup
```

## Launch everything

```powershell
.\scripts\Launch-URM.ps1
```

## SOTA features

1. **Reality Snapshots** — every 60s to `urm/snapshots/latest.json`; restore with `python nexus_orchestrator.py --restore` or IPC `restore_urm_snapshot`.
2. **Inference sharding** — host vs edge load in `nexus/state.json` (M5Stack offload scaffold).
3. **Code mutagenesis** — log scan → `urm/mutagenesis/latest.json` + browser overlay.

## Browser integration

- Toolbar **URM** button → `get_urm_status`
- Nexus overlay banner when Truth conflicts or mutagenesis proposes patches
- `DismissUrmOverlay` IPC clears banner

## Hardware licensing scaffold

Motherboard UUID written to `urm/licensing/hardware_id.txt` on first run (local only, no cloud).

## PyInstaller (optional)

```powershell
pip install pyinstaller
cd urm
pyinstaller --onefile nexus_orchestrator.py --name utah-urm-nexus
```

Sign the resulting `.exe` for distribution under `C:\ProgramData\Utah_URM`.
