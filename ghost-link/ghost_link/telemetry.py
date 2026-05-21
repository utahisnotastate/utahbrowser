"""Standardized telemetry frames + vault broadcast (theme, nexus bridge)."""

from __future__ import annotations

import json
import logging
from dataclasses import asdict, dataclass
from datetime import datetime, timezone
from pathlib import Path
from typing import Any, Optional

from .config import ghost_link_root

logger = logging.getLogger("ghost_link.telemetry")


@dataclass
class TelemetryFrame:
    """Unified peripheral snapshot for Ghost-Link / Utah Browser."""

    ts: str
    audio_rms: float
    motion: float
    entropy: float
    theme_mode: str  # "focus" | "calm"
    camera_active: bool
    mic_active: bool
    extra: Optional[dict[str, Any]] = None

    def serialize(self) -> bytes:
        return json.dumps(asdict(self), indent=2).encode("utf-8")

    @classmethod
    def from_sensory(
        cls,
        audio_rms: float,
        motion: float,
        entropy: float,
        *,
        camera_active: bool = True,
        mic_active: bool = True,
    ) -> "TelemetryFrame":
        # Haptic/sensory hook: loud room → high-contrast focus palette
        if audio_rms >= 0.06 or entropy >= 0.2:
            theme = "focus"
        else:
            theme = "calm"
        return cls(
            ts=datetime.now(timezone.utc).isoformat(),
            audio_rms=round(audio_rms, 5),
            motion=round(motion, 3),
            entropy=round(entropy, 4),
            theme_mode=theme,
            camera_active=camera_active,
            mic_active=mic_active,
        )


def broadcast_to_vault(frame: TelemetryFrame) -> None:
    """Write latest frame + theme for Rust UI (no shared-memory driver required)."""
    root = ghost_link_root()
    out = root / "out"
    out.mkdir(parents=True, exist_ok=True)

    frame_path = out / "telemetry_frame.json"
    frame_path.write_bytes(frame.serialize())

    theme = {
        "ts": frame.ts,
        "mode": frame.theme_mode,
        "audio_rms": frame.audio_rms,
        "entropy": frame.entropy,
        "accent": "#ff6b4a" if frame.theme_mode == "focus" else "#3dd68c",
        "contrast": "high" if frame.theme_mode == "focus" else "soft",
    }
    (out / "theme.json").write_text(json.dumps(theme, indent=2), encoding="utf-8")

    logger.debug("telemetry broadcast theme=%s rms=%.4f", frame.theme_mode, frame.audio_rms)
