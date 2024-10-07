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

  cargoHash = "sha256-iX901ZpDPfAUqEJ9DeIpsDPmSER6p61iAR1C7QY1f5Y=";

  nativeBuildInputs = [
    rust_platform.bindgenHook
    # (pkgs.rosPackages.humble.buildEnv
    #   {
    #     paths = [
    #       pkgs.rosPackages.humble.ros-core
    #     ];
    #   })
    pkgs.rosPackages.humble.ros-core
  ];
  propigatedBuildInputs = [
  ];

  meta = with pkgs.lib; {
    description = "ROS interface for iRobot Create 2 Open Interface";
    homepage = "https://github.com/IamTheCarl/ros_assistant";
    # license = licenses.mit; # TODO a license needs to be picked.
    maintainers = with maintainers; [ "James Carl" ];
  };
}


# { pkgs ? import <nixpkgs> { } }:
# let
#   pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
#   cargo_nix = pkgs.callPackage ./Cargo.nix { };
#   crateOverrides = pkgs.defaultCrateOverrides // {
#     create_bridge = attrs: {
#       buildInputs = (attrs.buildInputs or [ ]) ++ [ ];
#       nativeBuildInputs = (attrs.nativeBuildInputs or [ ]) ++ [ (import ./build_dependencies.nix { pkgs = pkgs; }) ];
#     };
#   };
# in
# cargo_nix.rootCrate.build.override {
#   crateOverrides = crateOverrides;
# }
