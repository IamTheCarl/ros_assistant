{ pkgs ? import ../../nix/ros.nix { } }:
pkgs.rosPackages.humble.buildRosPackage {
  pname = "create_bridge_interface";
  version = "0.1.0";
  src = ./.;

  buildType = "ament_cmake";
  buildInputs = [ pkgs.rosPackages.humble.ament-cmake pkgs.rosPackages.humble.rosidl-default-generators ];
  propagatedBuildInputs = [ ];
  nativeBuildInputs = [ pkgs.rosPackages.humble.ament-cmake pkgs.rosPackages.humble.rosidl-default-generators ];

  meta = {
    description = "Provides messages and services for interacting with Home Assistant";
  };
}
