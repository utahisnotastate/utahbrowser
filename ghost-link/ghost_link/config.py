"""Ghost-Link configuration (env overrides, vault paths)."""

from __future__ import annotations

import os
from dataclasses import dataclass
from pathlib import Path


def vault_root() -> Path:
    return Path(os.environ.get("UTAH_VAULT", Path.home() / ".utah_browser"))


def ghost_link_root() -> Path:
    return Path(os.environ.get("GHOST_LINK_HOME", vault_root() / "ghost-link"))


@dataclass
class GhostConfig:
    ollama_host: str = "http://127.0.0.1:11434"
    vision_model: str = "llava"
    chat_model: str = "llama3.2"
    frame_interval_ms: int = 500
    buffer_seconds: float = 5.0
    entropy_threshold: float = 0.12
    audio_energy_threshold: float = 0.02
    motion_threshold: float = 8.0
    sample_rate: int = 16000
    camera_index: int = 0
    enable_camera: bool = True
    enable_audio: bool = True

    @classmethod
    def from_env(cls) -> "GhostConfig":
        return cls(
            ollama_host=os.environ.get("OLLAMA_HOST", cls.ollama_host),
            vision_model=os.environ.get("OLLAMA_VISION_MODEL", cls.vision_model),
            chat_model=os.environ.get("OLLAMA_CHAT_MODEL", cls.chat_model),
            frame_interval_ms=int(os.environ.get("GHOST_FRAME_MS", cls.frame_interval_ms)),
            buffer_seconds=float(os.environ.get("GHOST_BUFFER_SEC", cls.buffer_seconds)),
            entropy_threshold=float(os.environ.get("GHOST_ENTROPY", cls.entropy_threshold)),
            audio_energy_threshold=float(
                os.environ.get("GHOST_AUDIO_THRESHOLD", cls.audio_energy_threshold)
            ),
            motion_threshold=float(os.environ.get("GHOST_MOTION", cls.motion_threshold)),
            sample_rate=int(os.environ.get("GHOST_SAMPLE_RATE", cls.sample_rate)),
            camera_index=int(os.environ.get("GHOST_CAMERA_INDEX", cls.camera_index)),
            enable_camera=os.environ.get("GHOST_DISABLE_CAMERA", "0") != "1",
            enable_audio=os.environ.get("GHOST_DISABLE_AUDIO", "0") != "1",
        )

    def ensure_dirs(self) -> None:
        for sub in ("logs", "cache", "out"):
            (ghost_link_root() / sub).mkdir(parents=True, exist_ok=True)

    @property
    def log_file(self) -> Path:
        return ghost_link_root() / "logs" / "telemetry.log"

    @property
    def events_file(self) -> Path:
        return ghost_link_root() / "out" / "events.jsonl"

    @property
    def hints_file(self) -> Path:
        return ghost_link_root() / "out" / "hints.json"

    @property
    def prefetch_file(self) -> Path:
        return ghost_link_root() / "out" / "prefetch.json"
