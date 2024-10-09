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

  executable = cargo_nix.rootCrate.build.overrideAttrs { };
in
pkgs.rosPackages.humble.buildRosPackage {
  pname = "create_bridge";
  version = "0.0.0";

  src = ./ros_stuff;

  buildType = "ament_cmake";
  buildInputs = [ pkgs.rosPackages.humble.ament-cmake ];

  # We need to depend on the executable being installed at runtime.
  propagatedBuildInputs = [ executable ];

  # Make the symbolic link to the executable available to the build
  # environment.
  postUnpack = ''
    ln -s ${executable}/bin/create_bridge $sourceRoot/create_bridge
  '';
}
    
