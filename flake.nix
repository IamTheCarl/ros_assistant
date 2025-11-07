{
  description = "A very basic flake";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    rass-cli = {
      url = "./packages/rass-cli";
      inputs = {
        nixpkgs.follows = "nixpkgs";
	rust-overlay.follows = "rust-overlay";
      };
    };
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, rass-cli, flake-utils, ... }: {
    rosify = ./packages/rosify_package;
    rass-modules = rass-cli + "/nix_modules";
  } // flake-utils.lib.eachDefaultSystem (system:
  {
    packages = {
      rass-cli = rass-cli.packages.${system}.default;
    };
  });
}
