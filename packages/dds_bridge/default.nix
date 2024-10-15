{ pkgs ? import <nixpkgs> { } }:
let
  rust = import ../../nix/rust.nix { pkgs = pkgs; };
  rust_platform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
  messages = [ ];

  addDeps = list: { ... }: {
    nativeBuildInputs = list ++ (import ./build_dependencies.nix {
      pkgs = pkgs;
      rust = rust;
      rust_platform = rust_platform;
    });
  };
  cargo_nix = pkgs.callPackage ./Cargo.nix { };
in
cargo_nix.rootCrate.build.overrideAttrs { }
