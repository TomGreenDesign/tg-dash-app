#!/usr/bin/env bash
set -euo pipefail
SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
IMAGE="mokimo-tauri-dev"

docker build -t "$IMAGE" -f "${SCRIPT_DIR}/Dockerfile.dev" "$SCRIPT_DIR" 2>/dev/null

docker run --rm \
  --name "tauri-build-dash" \
  -v "${SCRIPT_DIR}:/workspace" \
  -v "tauri-cargo-registry:/usr/local/cargo/registry" \
  -v "tauri-cargo-git:/usr/local/cargo/git" \
  -e "TAURI_SIGNING_PRIVATE_KEY=${TAURI_SIGNING_PRIVATE_KEY:-}" \
  -e "TAURI_SIGNING_PRIVATE_KEY_PASSWORD=${TAURI_SIGNING_PRIVATE_KEY_PASSWORD:-}" \
  "$IMAGE" \
  bash -c "cd /workspace && npm install && npx tauri build $*"

echo "Build artifacts in: ${SCRIPT_DIR}/src-tauri/target/release/bundle/"
