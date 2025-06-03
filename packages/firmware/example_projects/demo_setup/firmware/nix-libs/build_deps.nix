{ pkgs ? import <nixpkgs> {} }:
let
  rust = pkgs.callPackage ./rust.nix {};
  rust_platform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
in
[]
