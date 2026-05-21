#!/usr/bin/env python3
"""
ZEO-CORE ingestion daemon — watches cognitive zones and signals incremental re-index.
Runs offline; pairs with Utah Browser Semantic Binding Engine.
"""

from __future__ import annotations

import argparse
import json
import logging
import sys
import time
from pathlib import Path

LOG_FMT = "%(asctime)s [ZEO-CORE] %(message)s"


def vault_root() -> Path:
    import os

    return Path(os.environ.get("UTAH_VAULT", Path.home() / ".utah_browser"))


def watch_manifest_path() -> Path:
    return vault_root() / "vault" / "ingestion_watch.json"


def zones_manifest_path() -> Path:
    return vault_root() / "vault" / "zones.json"


def load_watch_paths() -> list[str]:
    paths: list[str] = []
    zm = zones_manifest_path()
    if zm.is_file():
        try:
            data = json.loads(zm.read_text(encoding="utf-8"))
            for z in data.get("zones", []):
                p = z.get("path")
                if p and Path(p).exists():
                    paths.append(str(Path(p).resolve()))
        except json.JSONDecodeError:
            pass
    wm = watch_manifest_path()
    if wm.is_file():
        try:
            extra = json.loads(wm.read_text(encoding="utf-8"))
            for p in extra.get("paths", []):
                if p and Path(p).exists():
                    resolved = str(Path(p).resolve())
                    if resolved not in paths:
                        paths.append(resolved)
        except json.JSONDecodeError:
            pass
    return paths


def save_watch_paths(paths: list[str]) -> None:
    wm = watch_manifest_path()
    wm.parent.mkdir(parents=True, exist_ok=True)
    wm.write_text(json.dumps({"paths": paths}, indent=2), encoding="utf-8")


def add_path(path: str) -> None:
    paths = load_watch_paths()
    resolved = str(Path(path).resolve())
    if resolved not in paths:
        paths.append(resolved)
        save_watch_paths(paths)
    print(f"[ZEO-CORE] Cognitive zone bound for watch: {resolved}")


def setup_logging() -> None:
    log_dir = vault_root() / "logs"
    log_dir.mkdir(parents=True, exist_ok=True)
    logging.basicConfig(
        level=logging.INFO,
        format=LOG_FMT,
        handlers=[
            logging.FileHandler(log_dir / "ingestion_daemon.log", encoding="utf-8"),
        ],
    )


def start_daemon(extra_path: str | None = None) -> None:
    try:
        from watchdog.events import FileSystemEventHandler
        from watchdog.observers import Observer
    except ImportError:
        print("[ZEO-CORE] watchdog not installed — pip install watchdog", file=sys.stderr)
        sys.exit(1)

    if extra_path:
        add_path(extra_path)

    paths = load_watch_paths()
    if not paths:
        print("[ZEO-CORE] No zones to watch. Bind folders in Calibration Console.")
        return

    setup_logging()
    log = logging.getLogger("ingestion")

    class IngestionHandler(FileSystemEventHandler):
        def on_modified(self, event) -> None:
            if event.is_directory:
                return
            src = event.src_path
            if src.endswith((".md", ".txt", ".markdown", ".pdf")):
                log.info("File change detected — incremental update: %s", src)
                signal_file = vault_root() / "vault" / "ingest_signal.json"
                signal_file.parent.mkdir(parents=True, exist_ok=True)
                signal_file.write_text(
                    json.dumps({"path": src, "ts": time.time()}),
                    encoding="utf-8",
                )

    observer = Observer()
    for p in paths:
        observer.schedule(IngestionHandler(), p, recursive=True)
        log.info("Binding cognitive zone: %s", p)

    observer.start()
    print(f"[ZEO-CORE] Ingestion daemon watching {len(paths)} zone(s)")
    try:
        while True:
            time.sleep(1)
    except KeyboardInterrupt:
        observer.stop()
    observer.join()


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description="Utah Browser ingestion daemon")
    parser.add_argument("--add-path", help="Add folder to watch list and start")
    parser.add_argument("--list", action="store_true", help="List watched paths")
    args = parser.parse_args()

    if args.list:
        for p in load_watch_paths():
            print(p)
        sys.exit(0)

    start_daemon(args.add_path)
