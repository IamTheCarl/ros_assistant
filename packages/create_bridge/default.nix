{ pkgs ? import <nixpkgs> { } }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
  executable = import ./executable.nix { };
in
pkgs.rosPackages.humble.buildRosPackage {
  pname = "create_bridge";
  version = "0.0.0";

  src = ./ros_stuff;

  buildType = "ament_cmake";
  buildInputs = [ pkgs.rosPackages.humble.ament-cmake ];

  # We need to depend on the executable being installed at runtime.
  propagatedBuildInputs = [
    executable
    (import ../create_bridge_interface { pkgs = pkgs; })
  ];

  # Make the symbolic link to the executable available to the build
  # environment.
  postUnpack = ''
    ln -s ${executable}/bin/create_bridge $sourceRoot/create_bridge
  '';
}
    
