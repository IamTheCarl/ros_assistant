{ pkgs }:
let
  rust = import ../../nix/rust.nix { pkgs = pkgs; };
  rust_platform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
in
[
  rust
  rust_platform.bindgenHook
  pkgs.crate2nix

  (pkgs.rosPackages.humble.buildEnv
    {
      paths = [
        pkgs.rosPackages.humble.ros-core
      ];
    })
]
