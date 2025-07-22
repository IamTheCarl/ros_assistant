{
  description = "RASS command line tool";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
	    bashInteractive
            openssl
            pkg-config
            (rust-bin.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rust-analyzer"
                "rustfmt"
                "clippy"
              ];
	    })
          ];

	  shellHook = ''
            export SHELL=${pkgs.bashInteractive}/bin/bash
          '';
        };

	packages.default = with pkgs;
          let
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
          };
      }
    );
}
