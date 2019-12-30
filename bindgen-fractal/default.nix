{ name ? "bindgen_fractal", debug ? false }:

(import ../default.nix { }).makeRustBundler { inherit name debug; }
