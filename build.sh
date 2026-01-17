#!/usr/bin/env bash
set -euo pipefail

echo "== VEXT Vault Vercel Build =="

export CARGO_HOME="${VERCEL_CACHE_DIR:-/tmp}/.cargo"
export RUSTUP_HOME="${VERCEL_CACHE_DIR:-/tmp}/.rustup"
export PATH="$CARGO_HOME/bin:$PATH"

# Install Rust if missing
if ! command -v cargo >/dev/null 2>&1; then
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs \
    | sh -s -- -y --profile minimal
  source "$CARGO_HOME/env"
fi

rustup target add wasm32-unknown-unknown

# Install trunk (prebuilt)
if ! command -v trunk >/dev/null 2>&1; then
  curl -Ls https://github.com/trunk-rs/trunk/releases/download/v0.21.4/trunk-x86_64-unknown-linux-gnu.tar.gz \
    | tar -xz -C "$CARGO_HOME/bin"
fi

echo "Building WASM…"
trunk build --release --dist dist --public-url /

echo "Build output:"
ls -lah dist/
