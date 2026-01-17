#!/bin/bash
set -e

# --- Fix HOME mismatch on Vercel ---
export HOME=/root
export CARGO_HOME=/root/.cargo
export RUSTUP_HOME=/root/.rustup
export RUSTUP_INIT_SKIP_PATH_CHECK=yes

# --- Install Rust (minimal) ---
curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
source /root/.cargo/env

# --- Add WASM target ---
rustup target add wasm32-unknown-unknown

# --- Install Trunk ---
cargo install trunk --locked

# --- Build Leptos app ---
trunk build --release --dist dist --public-url /
