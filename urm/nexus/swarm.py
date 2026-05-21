"""M5Stack / edge swarm scaffold — hands and feet of the URM."""

from __future__ import annotations

import json
import logging
import time
from dataclasses import dataclass
from typing import Any, Optional

from .config import UrmConfig

log = logging.getLogger("urm.swarm")


@dataclass
class SwarmNode:
    node_id: str
    kind: str
    online: bool
    last_ping_ms: Optional[int] = None


class M5StackSwarm:
    """
    Hardware-agnostic swarm bus. Production: serial / WiFi to M5Stack fleet.
    Today: virtual node with host fallback.
    """

    def __init__(self, config: UrmConfig) -> None:
        self.config = config
        self._state_path = config.urm_root / "swarm" / "nodes.json"
        self._nodes: list[SwarmNode] = [
            SwarmNode(node_id="host-primary", kind="pc", online=True),
            SwarmNode(node_id="m5-virtual", kind="m5stack", online=False),
        ]

    def ping_nodes(self) -> list[SwarmNode]:
        port = self.config.urm_root / "swarm" / "m5stack.port"
        if port.is_file():
            try:
                serial_port = port.read_text(encoding="utf-8").strip()
                online = bool(serial_port)
                self._nodes[1] = SwarmNode(
                    node_id="m5-primary",
                    kind="m5stack",
                    online=online,
                    last_ping_ms=int(time.time() * 1000) % 10000,
                )
            except OSError:
                pass
        self._persist()
        return self._nodes

    def dispatch(self, action: str, payload: dict[str, Any]) -> bool:
        log.info("Swarm dispatch %s -> %s", action, payload)
        bus = self.config.urm_root / "swarm" / "command.json"
        bus.parent.mkdir(parents=True, exist_ok=True)
        bus.write_text(
            json.dumps({"action": action, "payload": payload, "ts": time.time()}),
            encoding="utf-8",
        )
        return True

    def collect_state(self) -> dict[str, Any]:
        return {
            "component": "swarm",
            "nodes": [
                {"id": n.node_id, "kind": n.kind, "online": n.online}
                for n in self.ping_nodes()
            ],
        }

    def _persist(self) -> None:
        self._state_path.parent.mkdir(parents=True, exist_ok=True)
        self._state_path.write_text(
            json.dumps(
                [
                    {"id": n.node_id, "kind": n.kind, "online": n.online}
                    for n in self._nodes
                ],
                indent=2,
            ),
            encoding="utf-8",
        )
