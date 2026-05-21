#!/usr/bin/env bash
# Ghost-Link Zero-Click launcher (Linux / macOS)
set -euo pipefail

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VAULT="${UTAH_VAULT:-$HOME/.utah_browser}"
GHOST_HOME="${GHOST_LINK_HOME:-$VAULT/ghost-link}"
VENV="$GHOST_HOME/env"

export UTAH_VAULT="$VAULT"
export GHOST_LINK_HOME="$GHOST_HOME"

mkdir -p "$GHOST_HOME/logs" "$GHOST_HOME/out"

if [ ! -d "$VENV" ]; then
  echo "[GHOST-LINK] Creating venv at $VENV"
  python3 -m venv "$VENV"
fi
# shellcheck disable=SC1091
source "$VENV/bin/activate"
pip install -q -r "$ROOT/requirements.txt"

cd "$ROOT"
if [ "${1:-}" = "--foreground" ] || [ "${GHOST_VERBOSE:-}" = "1" ]; then
  exec python -m ghost_link "$@"
fi

nohup python -m ghost_link >> "$GHOST_HOME/logs/stdout.log" 2>&1 &
echo "[GHOST-LINK] Daemon PID $! — logs: $GHOST_HOME/logs/telemetry.log"
