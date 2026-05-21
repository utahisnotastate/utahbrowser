"""Nexus Orchestrator — central nervous system of the Utah Unified Reality Manifold."""

from __future__ import annotations

import asyncio
import json
import logging
import time
from typing import Any

from .browser_bridge import BrowserController
from .config import UrmConfig, read_hardware_id, urm_root
from .inference_shard import InferenceShardRouter
from .mutagenesis import MutagenesisEngine
from .sensory_bridge import SensoryDaemon
from .snapshots import RealitySnapshotEngine
from .swarm import M5StackSwarm
from .vault_bridge import KnowledgeVault

log = logging.getLogger("urm.nexus")


class NexusOrchestrator:
    """Coordinates Ghost-Link, Browser overlays, Vault validation, Swarm, and snapshots."""

    def __init__(self, config: UrmConfig | None = None) -> None:
        self.config = config or UrmConfig.from_env()
        self.config.ensure_dirs()
        self.browser = BrowserController(self.config)
        self.sensory = SensoryDaemon(self.config)
        self.vault = KnowledgeVault(self.config)
        self.snapshots = RealitySnapshotEngine(self.config)
        self.shard = InferenceShardRouter(self.config)
        self.mutagenesis = MutagenesisEngine(self.config)
        self.swarm = M5StackSwarm(self.config)
        self.state: dict[str, Any] = {
            "status": "ACTIVE",
            "coherence": 1.0,
            "hardware_id": read_hardware_id(),
        }
        self._persist_hardware_id()
        self._running = True

    def _persist_hardware_id(self) -> None:
        path = self.config.hardware_id_path
        path.parent.mkdir(parents=True, exist_ok=True)
        if not path.is_file():
            path.write_text(self.state["hardware_id"], encoding="utf-8")

    def _write_state(self) -> None:
        payload = {
            **self.state,
            "shard": self.shard.metrics().to_dict(),
            "swarm": self.swarm.collect_state(),
            "sensory_active": self.sensory.is_daemon_active(),
            "snapshots": self.snapshots.list_snapshots(5),
            "updated": time.time(),
        }
        self.config.nexus_state.parent.mkdir(parents=True, exist_ok=True)
        self.config.nexus_state.write_text(
            json.dumps(payload, indent=2), encoding="utf-8"
        )

    async def run_integration_loop(self) -> None:
        log.info("[NEXUS] Synchronizing Neural Manifold at %s", urm_root())
        interval = 1.0 / max(self.config.poll_hz, 1.0)
        mutagenesis_counter = 0

        while self._running:
            try:
                sensory_data = await self.sensory.get_latest_frame_data()

                node = self.shard.record_task(
                    complexity=sensory_data.entropy + 0.1
                )
                if sensory_data.is_significant():
                    fact_check = await self.vault.validate(sensory_data.content)
                    self.shard.complete_task(node, 0.5)
                    if fact_check.conflict:
                        self.browser.inject_overlay(
                            fact_check.warning_msg,
                            severity="warn",
                        )
                        self.state["coherence"] = max(
                            0.3, float(self.state.get("coherence", 1.0)) - 0.05
                        )
                    else:
                        self.state["coherence"] = min(
                            1.0, float(self.state.get("coherence", 1.0)) + 0.02
                        )
                else:
                    self.shard.complete_task(node, 0.1)

                if self.snapshots.should_snapshot(self.config.snapshot_interval_sec):
                    self.snapshots.capture(
                        nexus_state=dict(self.state),
                        browser=self.browser.collect_state(),
                        sensory=self.sensory.collect_state(),
                        vault=self.vault.collect_state(),
                        swarm=self.swarm.collect_state(),
                        shard=self.shard.metrics().to_dict(),
                    )

                mutagenesis_counter += 1
                if mutagenesis_counter >= int(self.config.poll_hz * 30):
                    mutagenesis_counter = 0
                    proposals = self.mutagenesis.scan_logs(self.config.repo_root)
                    if proposals:
                        latest = proposals[-1]
                        self.browser.inject_overlay(
                            latest.get("summary", "Logic patch suggested."),
                            severity="info",
                        )

                self.shard.decay()
                self._write_state()
            except Exception as e:
                log.exception("Nexus loop error: %s", e)
                self.state["status"] = "DEGRADED"

            await asyncio.sleep(interval)

    def shutdown(self) -> None:
        self._running = False
        self.state["status"] = "SHUTDOWN"
        self._write_state()
        self.browser.clear_overlay()
        log.info("[NEXUS] Disengaging Reality Manifold. Safe shutdown complete.")

    def restore_latest_snapshot(self) -> bool:
        snap = self.snapshots.restore_latest()
        if not snap:
            return False
        self.browser.apply_state(snap)
        self.state["coherence"] = 1.0
        self.state["status"] = "RESTORED"
        self._write_state()
        return True
