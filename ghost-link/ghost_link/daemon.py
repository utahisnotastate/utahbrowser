"""Ghost-Link Intelligence Daemon — entropy-gated async reasoning over void buffers."""

from __future__ import annotations

import json
import logging
import os
import signal
import sys
import threading
import time
from datetime import datetime, timezone
from typing import Optional

from .config import GhostConfig, ghost_link_root
from .logging_setup import setup_logging
from .ollama_vlm import describe_scene, ping
from .siphon import PeripheralSiphon
from .telemetry import TelemetryFrame, broadcast_to_vault
from .void_buffer import SensoryVoid

logger = logging.getLogger("ghost_link.daemon")


class GhostLinkDaemon:
    def __init__(self, config: GhostConfig, verbose: bool = False) -> None:
        self.config = config
        self.verbose = verbose
        self.void = SensoryVoid(config.sample_rate, config.buffer_seconds)
        self._stop = threading.Event()
        self._last_reason = 0.0
        self._cooldown_sec = 8.0
        self._recent_audio = 0.0
        self._recent_motion = 0.0

    def run(self) -> None:
        self.config.ensure_dirs()
        setup_logging(self.config.log_file, verbose=self.verbose)
        logger.info("Ghost-Link Sovereign Engine starting — vault %s", ghost_link_root())

        if os.name != "nt":
            try:
                os.setsid()
            except OSError:
                pass

        siphon = PeripheralSiphon(self.config, self.void)
        siphon.on_audio_energy(self._note_audio)
        siphon.on_motion(self._note_motion)
        siphon.start()

        reason_thread = threading.Thread(target=self._reason_loop, daemon=True)
        reason_thread.start()

        def handle_sig(*_args) -> None:
            self._stop.set()

        signal.signal(signal.SIGINT, handle_sig)
        if hasattr(signal, "SIGTERM"):
            signal.signal(signal.SIGTERM, handle_sig)

        logger.info("Intelligence daemon listening (entropy threshold=%.3f)", self.config.entropy_threshold)
        while not self._stop.is_set():
            time.sleep(0.5)

        siphon.stop()
        logger.info("Ghost-Link daemon stopped")

    def _note_audio(self, rms: float) -> None:
        self._recent_audio = max(self._recent_audio * 0.9, rms)
        frame = TelemetryFrame.from_sensory(
            rms,
            self._recent_motion,
            self.void.entropy(rms, self._recent_motion),
            camera_active=self.config.enable_camera,
            mic_active=self.config.enable_audio,
        )
        broadcast_to_vault(frame)

    def _note_motion(self, motion: float) -> None:
        self._recent_motion = max(self._recent_motion * 0.85, motion)

    def _reason_loop(self) -> None:
        while not self._stop.is_set():
            time.sleep(0.25)
            audio_rms = self.void.audio.recent_rms()
            motion = max(self._recent_motion, self.void.video.max_motion())
            entropy = self.void.entropy(audio_rms, motion)

            wake_audio = audio_rms >= self.config.audio_energy_threshold
            wake_motion = motion >= self.config.motion_threshold
            if not self.void.should_reason(self.config.entropy_threshold, audio_rms, motion):
                if not (wake_audio or wake_motion):
                    continue

            now = time.time()
            if now - self._last_reason < self._cooldown_sec:
                continue
            self._last_reason = now

            trigger = "entropy" if entropy >= self.config.entropy_threshold else "wake"
            if wake_motion and wake_audio:
                trigger = "audio_visual"
            elif wake_motion:
                trigger = "motion"
            elif wake_audio:
                trigger = "audio"

            self._execute_reasoning(trigger, audio_rms, motion, entropy)

    def _execute_reasoning(
        self, trigger: str, audio_rms: float, motion: float, entropy: float
    ) -> None:
        frame = self.void.video.latest()
        summary: Optional[str] = None
        if frame and ping(self.config.ollama_host):
            summary = describe_scene(
                self.config,
                frame.jpeg_bytes,
                "Describe the user's workspace activity.",
                audio_rms,
                motion,
            )
        else:
            from .ollama_vlm import fallback_text

            summary = fallback_text(self.config, audio_rms, motion)

        event = {
            "ts": datetime.now(timezone.utc).isoformat(),
            "trigger": trigger,
            "entropy": round(entropy, 4),
            "audio_rms": round(audio_rms, 5),
            "motion": round(motion, 3),
            "summary": summary,
        }
        self._append_event(event)
        self._emit_prefetch_hint(summary or "")
        logger.info("Reasoning cycle [%s] entropy=%.3f", trigger, entropy)

    def _append_event(self, event: dict) -> None:
        path = self.config.events_file
        path.parent.mkdir(parents=True, exist_ok=True)
        with path.open("a", encoding="utf-8") as f:
            f.write(json.dumps(event) + "\n")

    def _emit_prefetch_hint(self, summary: str) -> None:
        """Bridge to Utah Browser Time-Loop prefetch (local JSON)."""
        hint = {
            "ts": datetime.now(timezone.utc).isoformat(),
            "prefetch": True,
            "summary": summary[:500],
            "suggested_url": "https://www.google.com/search?q=utah+browser+knowledge",
        }
        for path in (self.config.hints_file, self.config.prefetch_file):
            path.parent.mkdir(parents=True, exist_ok=True)
            path.write_text(json.dumps(hint, indent=2), encoding="utf-8")


def main(argv: Optional[list[str]] = None) -> int:
    verbose = "--verbose" in (argv or sys.argv) or os.environ.get("GHOST_VERBOSE") == "1"
    config = GhostConfig.from_env()
    GhostLinkDaemon(config, verbose=verbose).run()
    return 0
