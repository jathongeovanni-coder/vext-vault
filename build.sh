#!/bin/bash
set -e

echo "== Setting Rust env =="
export HOME=/root
export CARGO_HOME=/root/.cargo
export RUSTUP_HOME=/root/.rustup
export RUSTUP_INIT_SKIP_PATH_CHECK=yes

echo "== Installing Rust =="
curl -sSf https://sh.rustup.rs | sh -s -- -y --profile minimal
source /root/.cargo/env

echo "== Adding WASM target =="
rustup target add wasm32-unknown-unknown

echo "== Installing Trunk =="
cargo install trunk

echo "== Building with Trunk =="
trunk build --release --dist dist --public-url /

echo "== Build complete =="
ls -la dist
