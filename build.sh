#!/usr/bin/env bash
set -e

echo "== Vercel Rust + Trunk build =="

# Fix HOME mismatch on Vercel
export HOME=/root
export CARGO_HOME=/root/.cargo
export RUSTUP_HOME=/root/.rustup
export PATH="$CARGO_HOME/bin:$PATH"

# Install Rust (only if missing)
if ! command -v cargo >/dev/null 2>&1; then
  curl https://sh.rustup.rs -sSf | sh -s -- -y --profile minimal
fi

source "$CARGO_HOME/env"

# WASM target
rustup target add wasm32-unknown-unknown

# Trunk
if ! command -v trunk >/dev/null 2>&1; then
  cargo install trunk
fi

# Build
trunk build --release --dist dist --public-url /

echo "== Build finished =="
ls -la dist
