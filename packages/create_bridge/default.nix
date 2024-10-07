{ pkgs ? import <nixpkgs> { }, ... }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
  rust = import ../../nix/rust.nix { pkgs = pkgs; };
  rust_platform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
  cargo_toml = builtins.fromTOML (builtins.readFile ./Cargo.toml);
in
rust_platform.buildRustPackage rec {
  pname = "create_bridge";
  version = cargo_toml.package.version;

  src = ./.;

  cargoSha256 = "sha256-Xgw4HQWa/ubCGuFB49h1mWwnZjhV4LrcXweVQi4Kw7c=";

  nativeBuildInputs = import ./build_dependencies.nix { pkgs = pkgs; };
  propigatedBuildInputs = [
  ];

  meta = with pkgs.lib; {
    description = "ROS interface for iRobot Create 2 Open Interface";
    homepage = "https://github.com/IamTheCarl/ros_assistant";
    # license = licenses.mit; # TODO a license needs to be picked.
    maintainers = with maintainers; [ "James Carl" ];
  };
}
