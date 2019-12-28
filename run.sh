#!/usr/bin/env bash

# Usage: ./run.sh bare-metal-wasm
$(nix-build ../default.nix --no-out-link -A makeWeb --argstr name "$1" "$@")
