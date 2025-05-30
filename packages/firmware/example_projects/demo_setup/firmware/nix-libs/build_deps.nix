{ pkgs ? import <nixpkgs> {} }:
let
  rust = pkgs.callPackage ./rust.nix {};
  rust_platform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
  flip-link = rust_platform.buildRustPackage rec {
    pname = "flip-link";
    version = "0.1.10";

    src = pkgs.fetchCrate {
      inherit pname version;
      sha256 = "sha256-TOeadOvvRSr0p7flzT8X3PMtCpOdOuEWyJDhd/348EM=";
    };
    cargoHash = "sha256-7SA8nMWyC9pdeCuczBhGfM/RCE4gpOVjsQ9u5g3lEQE=";
    
    # Linker scripts necessary for testing aren't included in the crate package.
    doCheck = false;
  };
in
[
  flip-link
  pkgs.gcc
]
