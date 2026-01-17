#!/usr/bin/env bash
set -e

echo "== VEXT Vault Vercel Build =="

# --- FIX VERCEL HOME BUG ---
export HOME=/root
export RUSTUP_INIT_SKIP_PATH_CHECK=yes

# --- Cache paths ---
export CARGO_HOME=/root/.cargo
export RUSTUP_HOME=/root/.rustup
export PATH="$CARGO_HOME/bin:$PATH"

# --- Install Rust (only if missing) ---
if ! command -v cargo >/dev/null 2>&1; then
  echo "Installing Rust..."
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi

# --- Load cargo env ---
source "$CARGO_HOME/env"

# --- WASM target ---
rustup target add wasm32-unknown-unknown

# --- Install Trunk (compile-safe version) ---
if ! command -v trunk >/dev/null 2>&1; then
  echo "Installing Trunk..."
  cargo install trunk --locked
fi

# --- Build ---
echo "Building WASM…"
trunk build --release --dist dist --public-url /

echo "✅ Build complete"
ls -lah dist
