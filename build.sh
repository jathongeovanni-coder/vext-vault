#!/usr/bin/env bash
set -e

echo "== VEXT Vault Build =="

# Use Vercel's cache directory
export CARGO_HOME="${VERCEL_CACHE_DIR:-/tmp/cache}/.cargo"
export RUSTUP_HOME="${VERCEL_CACHE_DIR:-/tmp/cache}/.rustup"
export PATH="$CARGO_HOME/bin:$PATH"

# Install Rust only if not cached
if ! command -v cargo >/dev/null 2>&1; then
  echo "Installing Rust..."
  curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
  source "$CARGO_HOME/env"
fi

# Add WASM target
rustup target add wasm32-unknown-unknown

# Install Trunk (check cache first)
if ! command -v trunk >/dev/null 2>&1; then
  echo "Installing Trunk..."
  cargo install --locked trunk
fi

# Build with release optimizations
echo "Building WASM application..."
trunk build --release --dist dist --public-url /

echo "✅ Build complete"
ls -lah dist/