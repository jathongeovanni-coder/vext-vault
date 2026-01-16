#!/bin/sh
set -e

# Install Rust
curl https://sh.rustup.rs -sSf | sh -s -- -y
export PATH="$HOME/.cargo/bin:$PATH"

# Add wasm target
rustup target add wasm32-unknown-unknown

# Install trunk
cargo install trunk --locked

# Build WASM
trunk build --release
