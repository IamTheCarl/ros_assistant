{ pkgs ? import <nixpkgs> { }, pname, version, to_rosify, message_packages ? [ ], description, license }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
in
pkgs.rosPackages.humble.buildRosPackage rec {
  inherit pname version;

  src = ./.;

  buildType = "ament_cmake";
  buildInputs = [ pkgs.rosPackages.humble.ament-cmake ];
  nativeBuildInputs = [ pkgs.envsubst ];

  NODE_NAME = pname;
  VERSION = version;
  DESCRIPTION = description;
  LICENSE = license;

  propagatedBuildInputs = [
    to_rosify
  ] ++ message_packages;

  # Replace symbols with the correct values in the package.xml.
  # Make the symbolic link to the executable available to the build
  # environment.
  postUnpack = ''
    mv $sourceRoot/package.xml $sourceRoot/package_template.xml
    envsubst < $sourceRoot/package_template.xml > $sourceRoot/package.xml
    cat $sourceRoot/package.xml
    ln -s ${to_rosify}/bin/${pname} $sourceRoot/${pname}
  '';
}
