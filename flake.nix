{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    nhome-cli = {
      url = "./packages/nhome-cli";
      inputs = {
        nixpkgs.follows = "nixpkgs";
	rust-overlay.follows = "rust-overlay";
      };
    };
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, nhome-cli, flake-utils, ... }: {
    rosify = ./packages/rosify_package;
    nix-modules = nhome-cli + "/nix_modules";
  } // flake-utils.lib.eachDefaultSystem (system:
  {
    packages = {
      nhome-cli = nhome-cli.packages.${system}.default;
    };
  });
}
