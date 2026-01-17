#!/usr/bin/env bash
set -e

echo "== VEXT Vault Vercel Build =="

# Vercel-safe HOME
export HOME=/tmp
export CARGO_HOME="$HOME/.cargo"
export RUSTUP_HOME="$HOME/.rustup"
export PATH="$CARGO_HOME/bin:$PATH"

# Install Rust if missing
if ! command -v cargo >/dev/null 2>&1; then
  echo "Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal
fi

# Source cargo env ONLY if it exists
if [ -f "$HOME/.cargo/env" ]; then
  source "$HOME/.cargo/env"
fi

# WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk from source (glibc-safe)
if ! command -v trunk >/dev/null 2>&1; then
  echo "Installing Trunk from source..."
  cargo install trunk --locked
fi

echo "Building WASM…"
trunk build --release --dist dist --public-url /

echo "✅ Build complete"
