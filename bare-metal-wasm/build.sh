#!/usr/bin/env bash
$(nix-build ../default.nix --no-out-link \
    -A makeRustBundler \
    --argstr name "bare_metal_wasm" \
    --arg useWasmPack false \
    "$@" \
)
