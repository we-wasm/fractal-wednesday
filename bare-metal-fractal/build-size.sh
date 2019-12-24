#!/usr/bin/env bash

set -euo pipefail

PROJECT=bare_metal_fractal

TARGET=wasm32-unknown-unknown
BINARY=../target/$TARGET/release/$PROJECT.wasm

# https://rustwasm.github.io/docs/book/reference/code-size.html
cargo +nightly build --target $TARGET --release
wasm-strip $BINARY
mkdir -p www
wasm-opt -o www/$PROJECT.wasm -O3 $BINARY 
ls -lh www/$PROJECT.wasm
