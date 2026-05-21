"""URM vault paths and environment configuration."""

from __future__ import annotations

import os
import platform
from dataclasses import dataclass
from pathlib import Path


def program_data_root() -> Path:
    if os.name == "nt":
        base = os.environ.get("PROGRAMDATA", r"C:\ProgramData")
        return Path(base) / "Utah_URM"
    return Path.home() / ".utah_browser" / "urm"


def vault_root() -> Path:
    return Path(os.environ.get("UTAH_VAULT", Path.home() / ".utah_browser"))


def urm_root() -> Path:
    return Path(os.environ.get("URM_HOME", program_data_root() if os.environ.get("URM_USE_PROGRAMDATA") else vault_root() / "urm"))


@dataclass
class UrmConfig:
    poll_hz: float = 10.0
    snapshot_interval_sec: float = 60.0
    ollama_host: str = "http://127.0.0.1:11434"
    repo_root: Path = Path(__file__).resolve().parents[2]

    @classmethod
    def from_env(cls) -> "UrmConfig":
        return cls(
            poll_hz=float(os.environ.get("URM_POLL_HZ", "10")),
            snapshot_interval_sec=float(os.environ.get("URM_SNAPSHOT_SEC", "60")),
            ollama_host=os.environ.get("OLLAMA_HOST", cls.ollama_host),
            repo_root=Path(os.environ.get("UTAH_REPO", cls.repo_root)),
        )

    def ensure_dirs(self) -> None:
        for sub in (
            "logs",
            "snapshots",
            "nexus",
            "mutagenesis",
            "swarm",
            "licensing",
        ):
            (urm_root() / sub).mkdir(parents=True, exist_ok=True)
        (vault_root() / "vault").mkdir(parents=True, exist_ok=True)

    @property
    def nexus_state(self) -> Path:
        return urm_root() / "nexus" / "state.json"

    @property
    def browser_overlay(self) -> Path:
        return urm_root() / "nexus" / "overlay.json"

    @property
    def telemetry_log(self) -> Path:
        return urm_root() / "logs" / "nexus.log"

    @property
    def snapshots_dir(self) -> Path:
        return urm_root() / "snapshots"

    @property
    def hardware_id_path(self) -> Path:
        return urm_root() / "licensing" / "hardware_id.txt"

    @property
    def ingest_signal(self) -> Path:
        return vault_root() / "vault" / "ingest_signal.json"

    @property
    def ghost_events(self) -> Path:
        return vault_root() / "ghost-link" / "out" / "events.jsonl"

    @property
    def zones_manifest(self) -> Path:
        return vault_root() / "vault" / "zones.json"


def read_hardware_id() -> str:
    if platform.system() == "Windows":
        try:
            import subprocess

            out = subprocess.check_output(
                ["wmic", "csproduct", "get", "uuid"],
                stderr=subprocess.DEVNULL,
                text=True,
                timeout=5,
            )
            lines = [ln.strip() for ln in out.splitlines() if ln.strip() and ln.strip().lower() != "uuid"]
            if lines:
                return lines[0]
        except Exception:
            pass
    return platform.node()
