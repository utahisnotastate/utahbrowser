"""Hardware-agnostic inference sharding — host PC vs edge (M5Stack) load routing."""

from __future__ import annotations

import logging
import time
from dataclasses import dataclass
from typing import Any

import requests

from .config import UrmConfig

log = logging.getLogger("urm.shard")


@dataclass
class ShardMetrics:
    host_load: float
    edge_load: float
    active_node: str
    ollama_online: bool

    def to_dict(self) -> dict[str, Any]:
        return {
            "host_load": round(self.host_load, 3),
            "edge_load": round(self.edge_load, 3),
            "active_node": self.active_node,
            "ollama_online": self.ollama_online,
        }


class InferenceShardRouter:
    """
    Transparently prefers host Ollama; shifts notional load to edge when host saturated.
    Edge offload is a scaffold until M5Stack inference firmware ships.
    """

    def __init__(self, config: UrmConfig) -> None:
        self.config = config
        self._host_queue = 0.0
        self._edge_queue = 0.0

    def record_task(self, complexity: float = 1.0) -> str:
        metrics = self.metrics()
        if metrics.edge_load + 0.2 < metrics.host_load and metrics.edge_load < 0.85:
            self._edge_queue += complexity
            return "edge"
        self._host_queue += complexity
        return "host"

    def complete_task(self, node: str, complexity: float = 1.0) -> None:
        if node == "edge":
            self._edge_queue = max(0.0, self._edge_queue - complexity)
        else:
            self._host_queue = max(0.0, self._host_queue - complexity)

    def metrics(self) -> ShardMetrics:
        host_load = min(1.0, self._host_queue / 10.0)
        edge_load = min(1.0, self._edge_queue / 5.0)
        ollama = self._ping_ollama()
        active = "host" if ollama and host_load <= edge_load else "edge"
        if not ollama and edge_load < 0.5:
            active = "edge"
        return ShardMetrics(
            host_load=host_load,
            edge_load=edge_load,
            active_node=active,
            ollama_online=ollama,
        )

    def _ping_ollama(self) -> bool:
        try:
            r = requests.get(
                f"{self.config.ollama_host.rstrip('/')}/api/tags",
                timeout=2,
            )
            return r.status_code == 200
        except requests.RequestException:
            return False

    def decay(self) -> None:
        self._host_queue *= 0.92
        self._edge_queue *= 0.88
