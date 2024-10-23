{ pkgs ? import <nixpkgs> { }, ... }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
  rust = import ../../nix/rust.nix { pkgs = pkgs; };
  rust_platform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };

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
    pkgs.openssh
  ];

  phases = [ "installPhase" ];

  postInstall = ''
    mkdir -p $out/bin
    cp ${package}/bin/cli $out/bin/rass
    wrapProgram $out/bin/rass \
      --prefix PATH : ${pkgs.nix}/bin:${pkgs.nixos-rebuild}/bin:${pkgs.openssh}/bin
  '';
}
