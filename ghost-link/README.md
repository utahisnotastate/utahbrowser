# Ghost-Link Sovereign Engine

Offline peripheral sensory daemon for Utah Browser (UTAH-OMEGA-23).

## Quick start

**Windows**

```powershell
..\scripts\install_ghost_link.ps1 -StartNow
```

**Linux / macOS**

```bash
./run.sh
```

## Architecture

| Component | File | Role |
|-----------|------|------|
| Peripheral Siphon | `ghost_link/siphon.py` | Camera + mic threads, motion-aware frame skip |
| Void Buffer | `ghost_link/void_buffer.py` | 5s circular audio/video RAM rings |
| Intelligence Daemon | `ghost_link/daemon.py` | Entropy-gated Ollama VLM reasoning |
| Utah Bridge | `out/prefetch.json` | Time-Loop hints for the Rust browser |

See [docs/guides/GHOST_LINK.md](../docs/guides/GHOST_LINK.md).
