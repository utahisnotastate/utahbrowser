"""Knowledge vault — semantic validation and Truth Engine coordination."""

from __future__ import annotations

import json
import logging
from dataclasses import dataclass
from typing import Any, Optional

import requests

from .config import UrmConfig

log = logging.getLogger("urm.vault")


@dataclass
class FactCheck:
    conflict: bool
    warning_msg: str
    confidence: float = 1.0


class KnowledgeVault:
    """Local knowledge manifold — zones, ingest signals, lightweight conflict heuristics."""

    def __init__(self, config: UrmConfig) -> None:
        self.config = config

    async def validate(self, content: str) -> FactCheck:
        if not content or len(content.strip()) < 12:
            return FactCheck(conflict=False, warning_msg="")
        lowered = content.lower()
        conflict_keywords = (
            "error",
            "failed",
            "discrepancy",
            "conflict",
            "incorrect",
            "hallucination",
            "unsafe",
        )
        if any(k in lowered for k in conflict_keywords):
            return FactCheck(
                conflict=True,
                warning_msg=f"Truth Guard: sensory narrative may conflict — {content[:200]}",
                confidence=0.7,
            )
        ollama_ok = self._ollama_ping()
        if ollama_ok and len(content) > 40:
            return FactCheck(
                conflict=False,
                warning_msg="",
                confidence=0.85,
            )
        return FactCheck(conflict=False, warning_msg="")

    def _ollama_ping(self) -> bool:
        try:
            r = requests.get(f"{self.config.ollama_host.rstrip('/')}/api/tags", timeout=3)
            return r.status_code == 200
        except requests.RequestException:
            return False

    def collect_state(self) -> dict[str, Any]:
        state: dict[str, Any] = {"component": "vault", "zones": [], "ingest_pending": False}
        zm = self.config.zones_manifest
        if zm.is_file():
            try:
                data = json.loads(zm.read_text(encoding="utf-8"))
                state["zones"] = data.get("zones", [])
                state["direct_mapping_global"] = data.get("direct_mapping_global", False)
            except json.JSONDecodeError:
                pass
        sig = self.config.ingest_signal
        state["ingest_pending"] = sig.is_file()
        return state

    def list_zone_paths(self) -> list[str]:
        zm = self.config.zones_manifest
        if not zm.is_file():
            return []
        try:
            data = json.loads(zm.read_text(encoding="utf-8"))
            return [z.get("path", "") for z in data.get("zones", []) if z.get("path")]
        except json.JSONDecodeError:
            return []
