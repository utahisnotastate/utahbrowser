"""Nexus logging to URM vault (not stdout in production)."""

from __future__ import annotations

import logging
import sys

from .config import UrmConfig


def setup_logging(config: UrmConfig, verbose: bool = False) -> logging.Logger:
    config.ensure_dirs()
    logger = logging.getLogger("urm")
    logger.setLevel(logging.DEBUG if verbose else logging.INFO)
    logger.handlers.clear()

    fh = logging.FileHandler(config.telemetry_log, encoding="utf-8")
    fh.setFormatter(
        logging.Formatter("%(asctime)s [%(levelname)s] %(name)s: %(message)s")
    )
    logger.addHandler(fh)

    if verbose:
        sh = logging.StreamHandler(sys.stderr)
        sh.setFormatter(logging.Formatter("[NEXUS] %(message)s"))
        logger.addHandler(sh)

    return logger
