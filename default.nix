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

  makeWeb = { name }:
    wrap {
      inherit name;
      paths = [ caddy ];
      script = ''
        caddy --root ${./. + "/${name}/www"}
      '';
    };

  makeRustBundler = { name, debug ? false, useWasmPack ? true, speed ? false }:
    let opt_flag = if speed then "-Oz" else "-O3";
    in wrap {
      inherit name;
      paths = [ wasm-pack wasm-strip wabt binaryen rWasm ];
      script = let

        cargo_script = let
          BINARY = toString (./target/wasm32-unknown-unknown
            + (if debug then "/debug" else "/release") + "/${name}.wasm");
        in ''
          #!/usr/bin/env bash
          cargo build --target wasm32-unknown-unknown ${
            optionalString (!debug) "--release"
          }
          ${optionalString (!debug) "wasm-strip ${BINARY}"}
          mkdir -p www
          wasm-opt -o www/${name}.wasm ${opt_flag} ${BINARY}
        '';

        wasm_pack_script = let BINARY = "${name}_bg.wasm";
        in ''
          #!/usr/bin/env bash
          wasm-pack build -t web ${optionalString debug "--dev"} .
          ${optionalString (!debug) "wasm-strip pkg/${BINARY}"}
          mkdir -p www
          wasm-opt -o www/${BINARY} ${opt_flag} pkg/${BINARY}
          cp pkg/${name}.js www/${name}.js
        '';
      in if useWasmPack then wasm_pack_script else cargo_script;
    };
in {
  inherit buildInputs RUST_SRC_PATH RUST_BACKTRACE;
  inherit rWasm wrap makeWeb makeRustBundler;
}
