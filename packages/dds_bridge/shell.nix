{ pkgs ? import <nixpkgs> { } }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
  rust = import ../../nix/rust.nix { pkgs = pkgs; };
  rust_platform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
in
pkgs.mkShell {
  buildInputs = import ./build_dependencies.nix
    {
      pkgs = pkgs;
      rust = rust;
      rust_platform = rust_platform;
    } ++ [
    (pkgs.rosPackages.humble.buildEnv
      {
        paths = [
          pkgs.rosPackages.humble.ros-core
          pkgs.rosPackages.humble.joy
        ];
      })
  ];

  # Set environment variables
  shellHook = ''
    export OPENSSL_NO_VENDOR=1
  '';
}
