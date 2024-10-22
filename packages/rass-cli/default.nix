{ pkgs ? import <nixpkgs> { }, ... }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
  rust = import ../../nix/rust.nix { pkgs = pkgs; };
  rust_platform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };

  ros_tunnel = (import ../ros_tunnel { pkgs = pkgs; });

  cargo_nix = pkgs.callPackage ./Cargo.nix { };
  package = cargo_nix.rootCrate.build;
in
pkgs.stdenv.mkDerivation {
  pname = package.name;
  version = package.version;
  nativeBuildInputs = [
    pkgs.makeWrapper
  ];

  propagatedBuildInputs = [
    pkgs.nix
    pkgs.nixos-rebuild
    pkgs.openssh
    ros_tunnel
  ];

  phases = [ "installPhase" ];

  postInstall = ''
    mkdir -p $out/bin
    cp ${package}/bin/cli $out/bin/rass
    wrapProgram $out/bin/rass \
      --prefix PATH : ${pkgs.nix}/bin:${pkgs.nixos-rebuild}/bin:${pkgs.openssh}/bin:${ros_tunnel}/bin
  '';
}
