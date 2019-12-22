# $ cargo-web build --target=wasm32-unknown-unknown
# $ cargo-web start --target=wasm32-unknown-unknown
# $ cargo-web deploy --target=wasm32-unknown-unknown
{ pkgs ? import ./nix { }, channel ? "nightly", date ? "2019-12-20", ... }:
let
  rPkg = pkgs.rustChannelOf { inherit date channel; };
  rWasm = rPkg.rust.override { targets = [ "wasm32-unknown-unknown" ]; };

in pkgs.mkShell {
  buildInputs = with pkgs; [ python3 cargo-web wasm-pack wabt rWasm ];

  # This is for Nix/NixOS compatibility with RLS/rust-analyzer
  RUST_SRC_PATH = "${rPkg.rust-src}/lib/rustlib/src/rust/src";
}
