{ pkgs ? import <nixpkgs> { } }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
in
pkgs.mkShell {
  buildInputs = [
    pkgs.rosPackages.humble.ros-core
    pkgs.rosPackages.humble.example-interfaces
    (import ../../packages/homeassistant_interface { })
    pkgs.colcon
    (import ./python.nix {
      pkgs = pkgs;
    })
  ];

  shellHook = ''
    eval "$(register-python-argcomplete ros2)"
    eval "$(register-python-argcomplete colcon)"
    eval "$(register-python-argcomplete rosidl)"
  '';
}
