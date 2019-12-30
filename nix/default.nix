{ sources ? import ./sources.nix }:

let moz = import sources.nixpkgs-mozilla;

in import sources.nixpkgs {
  overlays = [ moz ];
  config.allowUnfree = true;
}
