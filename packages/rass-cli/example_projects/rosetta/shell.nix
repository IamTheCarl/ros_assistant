{ pkgs ? import <nixpkgs> { }, ... }:
let
  pkgs = import ../../../../nix/ros.nix { pkgs = pkgs; };
in
pkgs.mkShell {
  buildInputs = [
    (import ../.. { pkgs = pkgs; })
    (pkgs.rosPackages.humble.buildEnv
      {
        paths = [
          pkgs.rosPackages.humble.ros-core
          pkgs.rosPackages.humble.joy
          pkgs.rosPackages.humble.teleop-twist-joy
          pkgs.rosPackages.humble.teleop-twist-keyboard
          pkgs.rosPackages.humble.rviz2
          (import ../../../create_bridge_interface { pkgs = pkgs; })
        ];
      })
  ];

  shellHook = ''
    eval "$(register-python-argcomplete ros2)"
  '';

}
