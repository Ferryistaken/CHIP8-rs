#!/usr/bin/env bash
set -euo pipefail

export PATH="$HOME/.cargo/bin:$PATH"
export CARGO_TERM_COLOR=always

# Ensure stable Rust (Netlify already has Rust; this is idempotent)
rustup toolchain install stable -y >/dev/null 2>&1 || true
rustup default stable

# Install/overwrite wasm-pack so we control the version
# (-f matters, otherwise the old one remains)
curl -sSf https://rustwasm.github.io/wasm-pack/installer/init.sh | sh -s -- -f

# Build WASM - NOTE: no --out-dir (avoid unstable cargo flag path)
# Also: pass cargo flags *after* `--`
wasm-pack build --release --target web -- --features web

# Assemble dist
rm -rf dist
mkdir -p dist
cp -r static/* dist/
cp -r pkg dist/pkg
[ -d roms ] && mkdir -p dist/roms && cp -r roms/* dist/roms || true

