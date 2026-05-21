"""Predictive Reality Snapshots — zero-loss manifold state every N seconds."""

from __future__ import annotations

import json
import logging
import time
from pathlib import Path
from typing import Any, Optional

from .config import UrmConfig

log = logging.getLogger("urm.snapshots")


class RealitySnapshotEngine:
    def __init__(self, config: UrmConfig) -> None:
        self.config = config
        self._last_snapshot = 0.0
        self._latest_path: Optional[Path] = None

    def should_snapshot(self, interval_sec: float) -> bool:
        now = time.time()
        if now - self._last_snapshot >= interval_sec:
            self._last_snapshot = now
            return True
        return False

    def capture(
        self,
        nexus_state: dict[str, Any],
        browser: dict[str, Any],
        sensory: dict[str, Any],
        vault: dict[str, Any],
        swarm: dict[str, Any],
        shard: dict[str, Any],
    ) -> Path:
        ts = int(time.time() * 1000)
        payload = {
            "ts": ts,
            "ts_iso": time.strftime("%Y-%m-%dT%H:%M:%SZ", time.gmtime()),
            "nexus": nexus_state,
            "browser": browser,
            "sensory": sensory,
            "vault": vault,
            "swarm": swarm,
            "inference_shard": shard,
        }
        self.config.snapshots_dir.mkdir(parents=True, exist_ok=True)
        path = self.config.snapshots_dir / f"snapshot_{ts}.json"
        path.write_text(json.dumps(payload, indent=2), encoding="utf-8")
        latest = self.config.snapshots_dir / "latest.json"
        latest.write_text(json.dumps(payload, indent=2), encoding="utf-8")
        self._latest_path = latest
        log.info("Reality snapshot: %s", path.name)
        return path

    def restore_latest(self) -> Optional[dict[str, Any]]:
        latest = self.config.snapshots_dir / "latest.json"
        if not latest.is_file():
            return None
        try:
            return json.loads(latest.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            return None

    def list_snapshots(self, limit: int = 10) -> list[str]:
        if not self.config.snapshots_dir.is_dir():
            return []
        files = sorted(
            self.config.snapshots_dir.glob("snapshot_*.json"),
            key=lambda p: p.stat().st_mtime,
            reverse=True,
        )
        return [p.name for p in files[:limit]]
