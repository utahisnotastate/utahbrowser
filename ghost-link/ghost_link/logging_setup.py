"""Telemetry logging to vault logs (not stdout in production mode)."""

from __future__ import annotations

import logging
import sys
from pathlib import Path


def setup_logging(log_file: Path, verbose: bool = False) -> logging.Logger:
    log_file.parent.mkdir(parents=True, exist_ok=True)
    logger = logging.getLogger("ghost_link")
    logger.setLevel(logging.DEBUG if verbose else logging.INFO)
    logger.handlers.clear()

    fh = logging.FileHandler(log_file, encoding="utf-8")
    fh.setFormatter(
        logging.Formatter("%(asctime)s [%(levelname)s] %(name)s: %(message)s")
    )
    logger.addHandler(fh)

    if verbose:
        sh = logging.StreamHandler(sys.stderr)
        sh.setFormatter(logging.Formatter("[GHOST-LINK] %(message)s"))
        logger.addHandler(sh)

    return logger
