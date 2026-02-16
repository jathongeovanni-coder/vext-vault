#!/usr/bin/env bash
# Exit on error
set -e

echo "--- VEXT Vault: Starting Institutional Build ---"

# --- FIX VERCEL PERMISSIONS & PATHS ---
# We use the current directory for cargo/rustup to avoid permission issues in /root
export CARGO_HOME="$core/.cargo"
export RUSTUP_HOME="$core/.rustup"
export PATH="$CARGO_HOME/bin:$PATH"

# 1. Install/Configure Rust
if ! command -v rustup >/dev/null 2>&1; then
    echo "Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
    source "$CARGO_HOME/env"
else
    echo "Rustup found. Ensuring stable toolchain is default..."
    rustup default stable
fi

echo "Adding WASM target..."
rustup target add wasm32-unknown-unknown

# 2. Install Trunk build tool
if ! command -v trunk >/dev/null 2>&1; then
    echo "Installing Trunk..."
    # Using --locked ensures we get a version compatible with our lockfile
    cargo install trunk --locked
fi

# 3. Execute Build
echo "Building VEXT Vault WASM Bundle..."
# We ensure trunk knows where the distribution folder is
trunk build --release --dist dist --public-url /

echo "--- Build Successful ---"
ls -lah dist