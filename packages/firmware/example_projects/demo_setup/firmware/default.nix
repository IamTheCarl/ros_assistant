{ pkgs ? import <nixpkgs> { } }:
let
  rust-overlay = builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/stable.tar.gz";
  lib = pkgs.lib;

  pkgsCross = import pkgs.path {
    config = {
      allowUnsupportedSystem = true;
    };
    system = builtins.currentSystem;
    crossSystem = {
      config = "arm-none-eabihf";
      isStatic = true;
      rustc = {
        config = "thumbv7em-none-eabihf";
      };
      gcc = {
        arch = "armv7";
        fpu = "vfp";
      };
    };
    overlays = [
      (import rust-overlay)
      (final: _prev: {
        rustToolchain = (final.rust-bin.stable.latest.default.override {
          extensions = [
            "rust-src"
            "rust-analyzer"
            "rustfmt"
            "clippy"
          ];
          targets = [
            "thumbv7em-none-eabihf"
          ];
        });
      })
    ];
  };

  pkgsBuild = pkgsCross.buildPackages;

  rustPlatform = pkgsCross.makeRustPlatform {
    cargo = pkgsBuild.rustToolchain;
    rustc = pkgsBuild.rustToolchain;
  };

  buildRustCrateForPkgs = crate: pkgsCross.buildRustCrate.override {
    rustc = pkgsBuild.rustToolchain;
    cargo = pkgsBuild.rustToolchain;
  };
  cargo_nix = import ./Cargo.nix {
    pkgs = pkgsCross;
    inherit buildRustCrateForPkgs; 
  };
in
cargo_nix.rootCrate.build
# rustPlatform.buildRustPackage rec {
#   pname = "embedded_firmware_demo";
#   version = "0.0.1";
#  
#   RUSTFLAGS = [
#     "-C"
#     "linker=${pkgsCross.stdenv.cc.targetPrefix}ld"
#   ];
# 
#   src = ./.;
#   cargoLock.lockFile = ./Cargo.lock;
# }
