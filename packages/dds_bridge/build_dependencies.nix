{ pkgs, rust, rust_platform }:
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
