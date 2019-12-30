#!/usr/bin/env bash
$(nix-build ../default.nix --no-out-link \
    -A makeRustBundler \
    --argstr name "bare_metal_fractal" \
    --arg useWasmPack false \
    "$@" \
)
