#!/usr/bin/env python3
"""Entry point: Utah Unified Reality Manifold Nexus Orchestrator."""

from __future__ import annotations

import argparse
import asyncio
import logging
import sys
from pathlib import Path

# Ensure repo root on path when run as script
ROOT = Path(__file__).resolve().parent
if str(ROOT) not in sys.path:
    sys.path.insert(0, str(ROOT))

from nexus.config import UrmConfig, urm_root
from nexus.logging_setup import setup_logging
from nexus.orchestrator import NexusOrchestrator


def main() -> int:
    parser = argparse.ArgumentParser(description="Utah URM Nexus Orchestrator")
    parser.add_argument("--restore", action="store_true", help="Restore latest snapshot and exit")
    parser.add_argument("--verbose", action="store_true")
    args = parser.parse_args()

    config = UrmConfig.from_env()
    config.ensure_dirs()
    setup_logging(config, verbose=args.verbose)

    orchestrator = NexusOrchestrator(config)

    if args.restore:
        ok = orchestrator.restore_latest_snapshot()
        print("[NEXUS] Restore:", "OK" if ok else "no snapshot")
        return 0 if ok else 1

    try:
        asyncio.run(orchestrator.run_integration_loop())
    except KeyboardInterrupt:
        orchestrator.shutdown()
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
