#!/bin/bash

set -euo pipefail

PROJECT=bare_metal_fractal

TARGET=wasm32-unknown-unknown
BINARY=../target/$TARGET/debug/$PROJECT.wasm

cargo +nightly build --target $TARGET
cp $BINARY www/$PROJECT.wasm
mkdir -p www
ls -lh www/$PROJECT.wasm
