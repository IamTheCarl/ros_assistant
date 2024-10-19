{ pkgs ? import <nixpkgs> { } }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
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
  custom_crate_config = pkgs: pkgs.buildRustCrate.override {
    defaultCrateOverrides = pkgs.defaultCrateOverrides // {
      r2r_rcl = addDeps [ ];
      r2r_msg_gen = addDeps (messages);
      r2r_actions = addDeps [ ];
      r2r = addDeps (messages);
    };
  };
  cargo_nix = pkgs.callPackage ./Cargo.nix {
    buildRustCrateForPkgs = custom_crate_config;
  };
in
cargo_nix.rootCrate.build.overrideAttrs { }
