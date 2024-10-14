{ pkgs ? import <nixpkgs> { } }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
in
pkgs.mkShell {
  buildInputs = [
    (pkgs.rosPackages.humble.buildEnv {
      paths = [
        pkgs.rosPackages.humble.ros-core
        (import ./. { pkgs = pkgs; })
      ];
    })
  ];

  # Set environment variables
  shellHook = ''
    export OPENSSL_NO_VENDOR=1
  '';
}
