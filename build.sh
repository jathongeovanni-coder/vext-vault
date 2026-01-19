#!/usr/bin/env bash
# Exit on error
set -e

echo "--- VEXT Vault: Starting Institutional Build ---"

# 1. Setup Environment Paths
# We ensure the script knows where to look for 'cargo' and 'trunk'
export PATH="$HOME/.cargo/bin:$PATH"

# 2. Install Rustup & WASM Target
# If Rust isn't there, we install the minimal version to save time
if ! command -v rustup >/dev/null 2>&1; then
    echo "Installing Rust toolchain..."
    curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
    # Source the environment so 'cargo' works immediately
    source "$HOME/.cargo/env"
fi

echo "Adding WASM target..."
rustup target add wasm32-unknown-unknown

# 3. Install Trunk
# FIXED: Changed the '}' to 'fi' so the script runs correctly on Vercel
if ! command -v trunk >/dev/null 2>&1; then
    echo "Installing Trunk (this may take a few minutes)..."
    cargo install trunk --locked
fi

# 4. The Build
# This takes your 'src' and 'index.html' and creates the final 'dist' folder
echo "Executing Trunk Build..."
trunk build --release --dist dist --public-url /

echo "--- Build Successful: VEXT Vault is ready for deployment ---"
# List the files so we can see them in the Vercel logs
ls -lah dist