{ pkgs ? import <nixpkgs> { } }:
let
  pkgs_cross = import pkgs.path {
    config = {
      allowUnsupportedSystem = true;
    };
    crossSystem = pkgs.lib.systems.examples.armhf-embedded // {
      extensions = {
        sharedLibrary = ".so";
      };
      rustc = {
        config = "thumbv7em-none-eabihf";
      };
    };
  };
  rust = pkgs_cross.callPackage ./nix-libs/rust.nix {};
  rust_platform = pkgs_cross.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
in
rust_platform.buildRustPackage rec {
  pname = "embedded_firmware_demo";
  version = "0.0.1";

  RUSTFLAGS = [
    "-C"
    "linker=${pkgs_cross.stdenv.cc.targetPrefix}ld"
  ];

  src = ./.;
  cargoLock.lockFile = ./Cargo.lock;
}
