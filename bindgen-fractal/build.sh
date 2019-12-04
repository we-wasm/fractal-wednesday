#!/bin/bash

set -e

PACKAGE=bindgen_fractal
BINARY=pkg/${PACKAGE}_bg.wasm

wasm-pack build --target=web .
wasm-strip $BINARY
mkdir -p www
wasm-opt -o www/${PACKAGE}_bg.wasm -O3 $BINARY 
#cp $BINARY www/${PACKAGE}_bg.wasm
cp pkg/${PACKAGE}.js www/${PACKAGE}.js
ls -lh www/${PACKAGE}_bg.wasm
ls -lh www/${PACKAGE}.js
