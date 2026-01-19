#!/usr/bin/env bash
# Exit on error
set -e

echo "--- VEXT Vault: Starting Institutional Build ---"

# --- FIX VERCEL HOME DIRECTORY ERROR ---
# Vercel's environment sometimes has a mismatch between $HOME and the user ID.
# These exports tell the Rust installer exactly where to go and to skip the check.
export HOME=/root
export RUSTUP_INIT_SKIP_PATH_CHECK=yes
export CARGO_HOME=/root/.cargo
export RUSTUP_HOME=/root/.rustup
export PATH="$CARGO_HOME/bin:$PATH"

# 1. Install Rustup & WASM Target
if ! command -v rustup >/dev/null 2>&1; then
    echo "Installing Rust toolchain..."
    # We use -y and --default-toolchain to make it non-interactive
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal --default-toolchain stable
    # Source the environment so 'cargo' works immediately
    source "$CARGO_HOME/env"
fi

echo "Adding WASM target..."
rustup target add wasm32-unknown-unknown

# 2. Install Trunk
if ! command -v trunk >/dev/null 2>&1; then
    echo "Installing Trunk (this may take a few minutes)..."
    cargo install trunk --locked
fi

# 3. The Build
echo "Executing Trunk Build..."
trunk build --release --dist dist --public-url /

echo "--- Build Successful: VEXT Vault is ready for deployment ---"
ls -lah dist