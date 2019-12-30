#!/usr/bin/env bash
parent_path=$( cd "$(dirname "${BASH_SOURCE[0]}")" ; pwd -P )
name="$1"
shift

# Usage: ./run.sh bare-metal-wasm
$(nix-build "$parent_path/default.nix" --no-out-link \
    -A makeWeb \
    --argstr name "$name" \
    "$@" \
)
