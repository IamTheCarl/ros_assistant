{ pkgs ? import ../../nix/ros.nix { } }:
pkgs.rosPackages.humble.buildRosPackage {
  pname = "home_assistant_bridge";
  version = "0.1.0";
  src = ./.;

  buildType = "ament_python";

  # Nix packages to be provided in the runtime environment.
  propagatedBuildInputs = [
    (import ../../packages/homeassistant_interface { })
    (import ./python.nix { pkgs = pkgs; })
  ];

  meta = {
    description = "Provides integration between ROS and Home Assistant";
  };
}
