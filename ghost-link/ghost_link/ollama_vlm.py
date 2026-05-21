"""Local Ollama vision-language inference (no cloud APIs)."""

from __future__ import annotations

import base64
import json
import logging
from typing import Any, Optional

import requests

from .config import GhostConfig

logger = logging.getLogger("ghost_link.vlm")


def ping(host: str) -> bool:
    try:
        r = requests.get(f"{host.rstrip('/')}/api/tags", timeout=4)
        return r.status_code == 200
    except requests.RequestException:
        return False


def describe_scene(
    config: GhostConfig,
    jpeg_bytes: bytes,
    prompt: str,
    audio_rms: float,
    motion: float,
) -> Optional[str]:
    """Send latest frame to local VLM; returns text summary or None on failure."""
    host = config.ollama_host.rstrip("/")
    b64 = base64.b64encode(jpeg_bytes).decode("ascii")
    user_prompt = (
        f"{prompt}\n\n"
        f"Context: audio_energy={audio_rms:.4f}, motion={motion:.2f}. "
        "Respond in 2-3 sentences. Suggest one useful next action for the user."
    )

    body: dict[str, Any] = {
        "model": config.vision_model,
        "messages": [
            {
                "role": "user",
                "content": user_prompt,
                "images": [b64],
            }
        ],
        "stream": False,
    }

    try:
        r = requests.post(f"{host}/api/chat", json=body, timeout=120)
        if r.status_code != 200:
            logger.warning("vlm chat failed %s: %s", r.status_code, r.text[:200])
            return fallback_text(config, audio_rms, motion)
        data = r.json()
        msg = data.get("message") or {}
        content = msg.get("content") or data.get("response")
        if content:
            return str(content).strip()
    except requests.RequestException as e:
        logger.warning("vlm request error: %s", e)

    return fallback_text(config, audio_rms, motion)


def fallback_text(config: GhostConfig, audio_rms: float, motion: float) -> str:
    """Offline heuristic when vision model unavailable."""
    if motion > config.motion_threshold and audio_rms > config.audio_energy_threshold:
        return "Active workspace: motion and audio detected. Consider opening your knowledge panel."
    if motion > config.motion_threshold:
        return "Visual activity detected — user may be navigating or gesturing."
    if audio_rms > config.audio_energy_threshold:
        return "Audio activity detected — possible speech or ambient sound."
    return "Ambient monitoring — low sensory entropy."
