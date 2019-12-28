{ pkgs ? import ./nix { } }:
pkgs.stdenv.mkShell {
  inherit (import ./default.nix { }) buildInputs RUST_SRC_PATH RUST_BACKTRACE;
}
