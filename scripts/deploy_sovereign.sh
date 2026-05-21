#!/usr/bin/env bash
# Utah Sovereign Node — Linux/macOS deployment
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$ROOT"

echo "==> MANIFESTING SOVEREIGN NODE"
export UTAH_KNOWLEDGE_PATH="${UTAH_KNOWLEDGE_PATH:-$HOME/knowledge}"

if command -v python3 >/dev/null; then
  python3 -m venv "$ROOT/.venv" 2>/dev/null || true
  # shellcheck disable=SC1091
  [ -f "$ROOT/.venv/bin/activate" ] && source "$ROOT/.venv/bin/activate"
fi

if command -v ollama >/dev/null; then
  echo "==> Priming local model (phi3:mini)"
  ollama pull phi3:mini || true
fi

if [ -d "$ROOT/ghost-link" ]; then
  pip install -q -r "$ROOT/ghost-link/requirements.txt" 2>/dev/null || true
fi

cargo build --release --bin utah-browser --bin utah-launch

mkdir -p "$ROOT/dist"
cp target/release/utah-browser "$ROOT/dist/" 2>/dev/null || cp target/release/utah-browser.exe "$ROOT/dist/" 2>/dev/null || true
cp -r config assets "$ROOT/dist/"

echo "==> NODE ONLINE — run: UTAH_BROWSER_HOME=$ROOT/dist ./target/release/utah-browser"
