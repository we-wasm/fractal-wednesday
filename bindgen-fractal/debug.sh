#!/usr/bin/env bash
$(nix-build --no-out-link --arg debug true)
