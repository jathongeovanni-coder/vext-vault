#!/bin/bash
set -euo pipefail

# --- FIX VERCEL RUSTUP HOME MISMATCH ---
export HOME=/root
export CARGO_HOME=/root/.cargo
export RUSTUP_HOME=/root/.rustup
export RUSTUP_INIT_SKIP_PATH_CHECK=yes
