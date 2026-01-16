#!/bin/bash
set -euo pipefail

# --- FIX VERCEL HOME / RUSTUP MISMATCH ---
export HOME=/root
export CARGO_HOME=/root/.cargo
export RUSTUP_HOME=/root/.rustup
export RUSTUP_INIT_SKIP_PATH_CHECK=yes
export PATH="$CARGO_HOME/bin:$PATH"

echo "[1/4] Installing Rust (if missing)"
if ! command -v cargo >/dev/null 2>&1; then
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi

echo "[2/4] Adding WASM target"
rustup target add wasm32-unknown-unknown

echo "[3/4] Installing Trunk (pinned)"
if ! command -v trunk >/dev/null 2>&1; then
  cargo install trunk --locked
fi

echo "[4/4] Building with Trunk → Vercel static output"
trunk build \
  --release \
  --dist /vercel/output/static \
  --public-url /

echo "✅ Build complete"
