#!/usr/bin/env bash
# Utah Browser Zero-Click Installer (Master Manifest)
# Target: Sovereign edge deployment — Linux / macOS

set -euo pipefail

echo "[UTAH-KERNEL] Initiating Reality Manifestation..."

ROOT="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
VAULT="${HOME}/.utah_browser"

# 1. Environment check
if ! command -v cargo &>/dev/null; then
  echo "[!] Rust/Cargo not detected. Install from https://rustup.rs"
  exit 1
fi

# 2. Workspace / vault seeding
mkdir -p "${VAULT}/vault" "${VAULT}/cache/tabs" "${VAULT}/extensions" "${VAULT}/logs"
echo "[UTAH-KERNEL] Vault: ${VAULT}"

# 3. Optional Python venv for ingestion daemons (future evolution pipeline)
if command -v python3 &>/dev/null; then
  if [ ! -d "${VAULT}/env" ]; then
    python3 -m venv "${VAULT}/env"
  fi
  # shellcheck disable=SC1091
  source "${VAULT}/env/bin/activate" 2>/dev/null || true
  pip install --quiet --upgrade pip 2>/dev/null || true
fi

# 4. Build release binary
cd "${ROOT}"
cargo build --release

# 5. Ollama health (local inference node)
OLLAMA_HOST="${OLLAMA_HOST:-http://127.0.0.1:11434}"
if curl -sf "${OLLAMA_HOST}/api/tags" >/dev/null; then
  echo "[OK] Ollama online at ${OLLAMA_HOST}"
else
  echo "[WARN] Start Ollama: ollama serve"
fi

# 6. Qdrant (semantic bookmarks + Truth Engine)
QDRANT_URL="${QDRANT_URL:-http://127.0.0.1:6333}"
if curl -sf "${QDRANT_URL}/collections" >/dev/null; then
  echo "[OK] Qdrant online at ${QDRANT_URL}"
else
  echo "[WARN] Qdrant offline — run scripts/Ensure-Qdrant.ps1 on Windows or start qdrant manually"
fi

# 7. Lazarus daemon scaffold (self-healing relaunch)
cat > "${VAULT}/daemon.sh" <<'EOF'
#!/usr/bin/env bash
BIN="${UTAH_BROWSER_BIN:-./target/release/utah-browser}"
while true; do
  "${BIN}" || true
  sleep 5
done
EOF
chmod +x "${VAULT}/daemon.sh"

# 8. Dist bundle
DIST="${ROOT}/dist"
mkdir -p "${DIST}"
cp "${ROOT}/target/release/utah-browser" "${DIST}/" 2>/dev/null || \
  cp "${ROOT}/target/release/utah-browser.exe" "${DIST}/" 2>/dev/null || true
cp -R "${ROOT}/config" "${ROOT}/assets" "${DIST}/" 2>/dev/null || true
cp "${ROOT}/deploy_utah.sh" "${DIST}/" 2>/dev/null || true

echo "[UTAH-KERNEL] Manifestation complete."
echo "  Binary: ${ROOT}/target/release/utah-browser"
echo "  Vault:  ${VAULT}"
echo "  Launch: ${DIST}/utah-browser (after copy) or cargo run --release"
