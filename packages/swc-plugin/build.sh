#!/usr/bin/env bash
# Build revguard_swc_plugin.wasm with the active rustc (prefer latest stable).
# Fallback: RUSTUP_TOOLCHAIN=1.90.0 ./build.sh
set -euo pipefail
cd "$(dirname "$0")"
unset CARGO_TARGET_DIR
rustc --version >&2
rustup target add wasm32-wasip1 >/dev/null
cargo build --target wasm32-wasip1 --release
cp -f target/wasm32-wasip1/release/revguard_swc_plugin.wasm .
ls -la revguard_swc_plugin.wasm
