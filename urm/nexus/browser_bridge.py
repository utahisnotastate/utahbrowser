"""Browser bridge — overlay injection and tab snapshot I/O for the Rust shell."""

from __future__ import annotations

import json
import logging
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Optional

from .config import UrmConfig

log = logging.getLogger("urm.browser")


@dataclass
class BrowserOverlay:
    message: str
    severity: str = "info"
    visible: bool = True

    def to_dict(self) -> dict[str, Any]:
        return {
            "message": self.message,
            "severity": self.severity,
            "visible": self.visible,
        }


class BrowserController:
    """Reads/writes JSON bridges consumed by Utah Browser (Wry chrome)."""

    def __init__(self, config: UrmConfig) -> None:
        self.config = config

    def inject_overlay(self, message: str, severity: str = "warn") -> None:
        path = self.config.browser_overlay
        path.parent.mkdir(parents=True, exist_ok=True)
        payload = BrowserOverlay(message=message, severity=severity, visible=True).to_dict()
        path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
        log.info("Overlay injected: %s", message[:80])

    def clear_overlay(self) -> None:
        path = self.config.browser_overlay
        if path.is_file():
            path.unlink(missing_ok=True)

    def read_tabs_snapshot_hint(self) -> Optional[dict[str, Any]]:
        zones = self.config.zones_manifest
        if not zones.is_file():
            return None
        try:
            return json.loads(zones.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            return None

    def collect_state(self) -> dict[str, Any]:
        state: dict[str, Any] = {"component": "browser", "overlay": None, "zones": None}
        if self.config.browser_overlay.is_file():
            try:
                state["overlay"] = json.loads(
                    self.config.browser_overlay.read_text(encoding="utf-8")
                )
            except json.JSONDecodeError:
                pass
        hint = self.read_tabs_snapshot_hint()
        if hint:
            state["zones"] = hint
        return state

    def apply_state(self, snapshot: dict[str, Any]) -> None:
        overlay = snapshot.get("browser", {}).get("overlay")
        if overlay and overlay.get("visible"):
            self.inject_overlay(
                overlay.get("message", ""),
                overlay.get("severity", "info"),
            )
        else:
            self.clear_overlay()
