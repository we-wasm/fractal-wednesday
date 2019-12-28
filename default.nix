{ pkgs ? import ./nix { }, channel ? "nightly", date ? "2019-12-20", ... }:
with builtins;
with pkgs;
with lib;

let
  rPkg = rustChannelOf { inherit date channel; };
  rWasm = rPkg.rust.override { targets = [ "wasm32-unknown-unknown" ]; };

  # Create a shell script that bakes in its dependencies
  # <http://chriswarbo.net/projects/nixos/useful_hacks.html>
  wrap = { paths ? [ ], vars ? { }, file ? null, script ? null, name ? "wrap" }:
    assert file != null || script != null
      || abort "wrap needs 'file' or 'script' argument";
    with rec {
      set = n: v:
        "--set ${escapeShellArg (escapeShellArg n)} "
        + "'\"'${escapeShellArg (escapeShellArg v)}'\"'";
      args = (map (p: "--prefix PATH : ${p}/bin") paths)
        ++ (attrValues (mapAttrs set vars));
    };
    runCommand name {
      f = if file == null then writeScript name script else file;
      buildInputs = [ makeWrapper ];
    } ''
      makeWrapper "$f" "$out" ${toString args}
    '';

  buildInputs = [ caddy cargo-web wasm-pack wabt binaryen rWasm ];
  RUST_BACKTRACE = 1;
  # This is for Nix/NixOS compatibility with RLS/rust-analyzer
  RUST_SRC_PATH = "${rPkg.rust-src}/lib/rustlib/src/rust/src";

  makeRustBundler = { name, debug ? false }:
    wrap {
      inherit name;
      paths = [ wasm-pack wasm-strip wabt binaryen rWasm ];
      script = let BINARY = "${name}_bg.wasm";
      in ''
        #!/usr/bin/env bash
        wasm-pack build -t web ${optionalString debug "--dev"} .
        wasm-strip pkg/${BINARY}
        mkdir -p www
        wasm-opt -o www/${BINARY} -O3 pkg/${BINARY}
        cp pkg/${name}.js www/${name}.js
      '';
    };
in {
  inherit buildInputs RUST_SRC_PATH RUST_BACKTRACE;
  inherit rWasm wrap makeRustBundler;
}
