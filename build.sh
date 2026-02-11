#!/usr/bin/env bash
set -euo pipefail

ROOT_DIR="$(cd "$(dirname "$0")" && pwd)"

echo "Building foxd..."

echo "Building console..."
cd "$ROOT_DIR/console"
pnpm install 
pnpm build
echo "Console built."

echo "Building daemon..."
cd "$ROOT_DIR/daemon"
cargo build --release
echo "Daemon built."

BINARY="$ROOT_DIR/daemon/target/release/foxd"
if [ -f "$BINARY" ]; then
    echo "Binary ready: $BINARY"
else
    echo "Build failed: binary not found."
    exit 1
fi
