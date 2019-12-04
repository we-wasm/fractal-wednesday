#!/bin/bash

set -euo pipefail

PROJECT=bare_metal_fractal

TARGET=wasm32-unknown-unknown
BINARY=../target/$TARGET/release/$PROJECT.wasm

cargo +nightly build --target $TARGET --release
wasm-strip $BINARY
mkdir -p www
wasm-opt -o www/$PROJECT.wasm -Oz $BINARY 
ls -lh www/$PROJECT.wasm
