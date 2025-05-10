{ pkgs ? import <nixpkgs> { }, ... }:
let
  rust-overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/stable.tar.gz");
  pkgs = import <nixpkgs> { overlays = [ rust-overlay ]; };
  rust = pkgs.rust-bin.stable.latest.default.override {
    extensions = [
      "rust-src"
      "rust-analyzer"
      "rustfmt"
      "clippy"
    ];
    targets = [ ];
  };
in
pkgs.mkShell {
  buildInputs = [ ];
}
