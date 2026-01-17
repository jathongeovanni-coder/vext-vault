#!/usr/bin/env bash
set -e

echo "== VEXT Vault Vercel Build =="

# Use Vercel cache if available
export CARGO_HOME="$PWD/.cargo"
export RUSTUP_HOME="$PWD/.rustup"
export PATH="$CARGO_HOME/bin:$PATH"

# Install Rust if missing
if ! command -v cargo >/dev/null 2>&1; then
  echo "Installing Rust..."
  curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
fi

# 🔴 THIS IS THE FIX 🔴
# Force-load cargo into PATH for non-interactive shell
if [ -f "$CARGO_HOME/env" ]; then
  source "$CARGO_HOME/env"
elif [ -f "$HOME/.cargo/env" ]; then
  source "$HOME/.cargo/env"
fi

rustup target add wasm32-unknown-unknown

# Install trunk (fast path)
if ! command -v trunk >/dev/null 2>&1; then
  echo "Installing trunk..."
  wget -qO- https://github.com/trunk-rs/trunk/releases/download/v0.21.4/trunk-x86_64-unknown-linux-gnu.tar.gz \
    | tar -xzf- -C "$CARGO_HOME/bin"
fi

echo "Running trunk build..."
trunk build --release --dist dist --public-url /

echo "✅ Build complete"
