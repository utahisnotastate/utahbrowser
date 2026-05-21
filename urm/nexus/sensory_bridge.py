"""Ghost-Link sensory bridge — reads daemon output from the vault."""

from __future__ import annotations

import json
import logging
from dataclasses import dataclass
from pathlib import Path
from typing import Any, Optional

from .config import UrmConfig

log = logging.getLogger("urm.sensory")


@dataclass
class SensoryFrame:
    content: str
    entropy: float = 0.0
    trigger: str = ""
    audio_rms: float = 0.0
    motion: float = 0.0

    def is_significant(self, threshold: float = 0.12) -> bool:
        return self.entropy >= threshold or self.trigger in (
            "audio",
            "motion",
            "audio_visual",
            "entropy",
        )


class SensoryDaemon:
    """Non-blocking reader for Ghost-Link event stream."""

    def __init__(self, config: UrmConfig) -> None:
        self.config = config
        self._last_line = 0

    async def get_latest_frame_data(self) -> SensoryFrame:
        return self._read_latest_event()

    def _read_latest_event(self) -> SensoryFrame:
        path = self.config.ghost_events
        if not path.is_file():
            return SensoryFrame(content="")
        lines = path.read_text(encoding="utf-8").strip().splitlines()
        if not lines:
            return SensoryFrame(content="")
        try:
            ev = json.loads(lines[-1])
        except json.JSONDecodeError:
            return SensoryFrame(content="")
        summary = ev.get("summary") or ""
        return SensoryFrame(
            content=summary,
            entropy=float(ev.get("entropy", 0)),
            trigger=str(ev.get("trigger", "")),
            audio_rms=float(ev.get("audio_rms", 0)),
            motion=float(ev.get("motion", 0)),
        )

    def collect_state(self) -> dict[str, Any]:
        frame = self._read_latest_event()
        return {
            "component": "ghost_link",
            "latest": {
                "content": frame.content,
                "entropy": frame.entropy,
                "trigger": frame.trigger,
            },
        }

    def is_daemon_active(self) -> bool:
        return self.config.ghost_events.is_file() or (
            self.config.vault_root / "ghost-link" / "logs" / "telemetry.log"
        ).is_file()

    @property
    def vault_root(self) -> Path:
        from .config import vault_root

        return vault_root()
