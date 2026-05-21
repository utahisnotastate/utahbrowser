"""Autonomous Code Mutagenesis — runtime error patterns → suggested logic patches."""

from __future__ import annotations

import json
import logging
import re
import time
from pathlib import Path
from typing import Any, Optional

from .config import UrmConfig, urm_root, vault_root

log = logging.getLogger("urm.mutagenesis")

ERROR_PATTERNS = [
    re.compile(r"panic!", re.I),
    re.compile(r"thread '.*' panicked", re.I),
    re.compile(r"Traceback \(most recent call last\)", re.I),
    re.compile(r"SyntaxError:", re.I),
    re.compile(r"error\[E\d+\]:", re.I),
    re.compile(r"FAILED", re.I),
]


class MutagenesisEngine:
    def __init__(self, config: UrmConfig) -> None:
        self.config = config
        self._seen: set[str] = set()
        self._proposals_dir = urm_root() / "mutagenesis"

    def scan_logs(self, repo_root: Path) -> list[dict[str, Any]]:
        proposals: list[dict[str, Any]] = []
        candidates = [
            urm_root() / "logs" / "nexus.log",
            vault_root() / "logs" / "ingestion_daemon.log",
            repo_root / "target" / "install-build.log",
        ]
        for log_path in candidates:
            if not log_path.is_file():
                continue
            for line in self._tail(log_path, 80):
                if not any(p.search(line) for p in ERROR_PATTERNS):
                    continue
                key = f"{log_path}:{line[:120]}"
                if key in self._seen:
                    continue
                self._seen.add(key)
                proposal = self._write_proposal(log_path, line, repo_root)
                if proposal:
                    proposals.append(proposal)
        return proposals

    def _tail(self, path: Path, n: int) -> list[str]:
        try:
            lines = path.read_text(encoding="utf-8", errors="replace").splitlines()
            return lines[-n:]
        except OSError:
            return []

    def _write_proposal(
        self, log_path: Path, error_line: str, repo_root: Path
    ) -> Optional[dict[str, Any]]:
        self._proposals_dir.mkdir(parents=True, exist_ok=True)
        ts = int(time.time() * 1000)
        target = self._guess_source_file(error_line, repo_root)
        proposal = {
            "ts": ts,
            "log": str(log_path),
            "error_excerpt": error_line[:500],
            "target_file": str(target) if target else None,
            "summary": (
                f"Suggested Logic Patch: investigate recurring error in "
                f"{target.name if target else 'unknown module'}. "
                "Review null handling and async boundaries."
            ),
            "patch_hint": (
                "// URM Mutagenesis scaffold — verify error path and add guard\n"
                "if (ctx.is_err()) { return Err(ctx.into()); }"
            ),
            "status": "suggested",
        }
        out = self._proposals_dir / f"proposal_{ts}.json"
        out.write_text(json.dumps(proposal, indent=2), encoding="utf-8")
        (self._proposals_dir / "latest.json").write_text(
            json.dumps(proposal, indent=2), encoding="utf-8"
        )
        log.info("Mutagenesis proposal: %s", out.name)
        return proposal

    def _guess_source_file(self, line: str, repo_root: Path) -> Optional[Path]:
        for part in line.replace("\\", "/").split():
            if part.endswith(".rs") or part.endswith(".py"):
                for candidate in (repo_root / part, repo_root / "src" / part):
                    if candidate.is_file():
                        return candidate
        return None

    def latest_proposal(self) -> Optional[dict[str, Any]]:
        latest = self._proposals_dir / "latest.json"
        if not latest.is_file():
            return None
        try:
            return json.loads(latest.read_text(encoding="utf-8"))
        except json.JSONDecodeError:
            return None
