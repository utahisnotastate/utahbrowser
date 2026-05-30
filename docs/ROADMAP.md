# Roadmap & experimental modules

Shipped vs planned for Utah Browser. This keeps the public README honest and separates the **browser binary** from optional experiments.

## Shipped (v1.0-GENESIS)

| Area | Status | Doc |
|------|--------|-----|
| Unified compositor (single WebView2 + iframe) | Done | [UNIFIED_COMPOSITOR_AND_PHASE3.md](technical/UNIFIED_COMPOSITOR_AND_PHASE3.md) |
| Truth Guard (ingest + verify) | Done | [MANUAL.md](technical/MANUAL.md) |
| Native Qdrant bootstrap | Done | [QDRANT_AND_SERVICES.md](guides/QDRANT_AND_SERVICES.md) |
| Intent prefetch + memory buffer | Done | [UNIFIED_COMPOSITOR_AND_PHASE3.md](technical/UNIFIED_COMPOSITOR_AND_PHASE3.md) |
| WASM extension host (wasmi) | Scaffold | Same |
| Ghost-Chrome UI + fluidic CSS | Done | [SOVEREIGN_INTELLIGENCE_STACK.md](SOVEREIGN_INTELLIGENCE_STACK.md) |
| Sentinel-Core (launcher, delayed daemons) | Done | Architecture report §Sentinel |

## Optional (install separately)

| Module | Path | Notes |
|--------|------|-------|
| Ghost-Link | `ghost-link/` | Sensory daemon; `install_ghost_link.ps1` |
| URM Nexus | `urm/` | Orchestrator scaffold; `install_urm.ps1` |
| Evolution watcher | `src/evolution/` | Off by default; AppData proposals |

## Planned / not yet productized

| Feature | Notes |
|---------|-------|
| `verify_active_tab` | DOM/UI Automation for live page verify |
| Audio → STT → verify | `cpal` scaffold exists |
| Full extension compile pipeline | Today: stub Wasm + vibe-create |
| Cross-origin intent prefetch | Limited to chrome DOM today |
| CI (clippy, tests, release Action) | Manual builds only |

## Local-only experiments (not in git)

The following may exist on a developer machine but are **not tracked** in this repository:

- `utah_core/` — PyQt/OpenGL research prototype (not part of `cargo build`)
- Private applications in other folders or repos — keep out of `utahbrowser`

Add `utah_core/` to your local `.git/info/exclude` if you keep prototypes beside the clone.
