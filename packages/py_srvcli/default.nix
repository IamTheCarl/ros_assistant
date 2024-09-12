{ pkgs ? import ../../nix/ros.nix { } }:
pkgs.rosPackages.humble.buildRosPackage {
  pname = "py_srvcli";
  version = "1.0.0";
  src = ./.;

  buildType = "ament_python";

  # Nix packages to be provided in the runtime environment.
  propagatedBuildInputs = [
    (import ./python.nix { pkgs = pkgs; })
  ];

  meta = {
    description = "Provides the `add_two_ints` service demo";
  };
}
