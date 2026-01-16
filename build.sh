#!/bin/bash
set -e

# Fix HOME mismatch on Vercel
export HOME=/root
export CARGO_HOME=/root/.cargo
export RUSTUP_HOME=/root/.rustup
export RUSTUP_INIT_SKIP_PATH_CHECK=yes

# Ensure cargo is available
source /root/.cargo/env

# Build WASM app
trunk build --release --dist dist --public-url /
