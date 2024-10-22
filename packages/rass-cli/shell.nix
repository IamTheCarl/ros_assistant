{ pkgs ? import <nixpkgs> { }, ... }:
let
  rust = import ../../nix/rust.nix { pkgs = pkgs; };
in
pkgs.mkShell {
  buildInputs = [
    rust
    pkgs.crate2nix
  ];
}
