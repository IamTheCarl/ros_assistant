{ pkgs, rust, rust_platform }:
[
  rust
  rust_platform.bindgenHook
  pkgs.crate2nix

  (pkgs.rosPackages.humble.buildEnv
    {
      paths = [
        pkgs.rosPackages.humble.ros-core
        pkgs.rosPackages.humble.geometry-msgs
        (import ../create_bridge_interface { pkgs = pkgs; })
      ];
    })
]
