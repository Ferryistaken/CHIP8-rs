#!/usr/bin/env bash
set -euo pipefail

# install rustup non-interactively and make cargo tools available
curl https://sh.rustup.rs -sSf | sh -s -- -y
export PATH="$HOME/.cargo/bin:$PATH"

# ensure a stable toolchain is present
rustup default stable

# install wasm-pack
curl https://rustwasm.github.io/wasm-pack/installer/init.sh -sSf | sh
export PATH="$HOME/.cargo/bin:$PATH"

# build the wasm package and assemble dist
wasm-pack build --target web --out-dir pkg
rm -rf dist
mkdir dist
cp -r static/* dist
cp -r pkg dist/pkg
