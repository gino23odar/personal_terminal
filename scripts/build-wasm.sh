#!/usr/bin/env bash
set -euo pipefail
cd "$(dirname "$0")/.."
if [[ "$(wasm-pack --version)" != "wasm-pack 0.15.1" ]]; then
  echo 'Install wasm-pack 0.15.1: cargo install wasm-pack --version 0.15.1 --locked' >&2
  exit 1
fi
wasm-pack build crates/gino-wasm --target web --release --locked
