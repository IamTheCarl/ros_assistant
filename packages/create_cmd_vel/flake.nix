{
  description = "Carpet Puncher flake";

  inputs = {
    nix-ros-overlay.url = "github:lopsided98/nix-ros-overlay/master";
    nixpkgs.follows = "nix-ros-overlay/nixpkgs"; # We use the nixpkgs from the nix-ros-overlay.
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url  = "github:numtide/flake-utils";
  };

  outputs = { self, nixpkgs, nix-ros-overlay, rust-overlay, flake-utils, ... }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [
	  nix-ros-overlay.overlays.default
	  (import rust-overlay)
	];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
	messages = [
          pkgs.rosPackages.humble.ros-core
          pkgs.rosPackages.humble.geometry-msgs
          (pkgs.callPackage ../create_bridge_interface { })
	];
        build_dependencies = [
	  pkgs.rustPlatform.bindgenHook
	];
      in
      {
        devShells.default = with pkgs; mkShell {
          buildInputs = [
	    bashInteractive
            openssl
            pkg-config
	    crate2nix
            (rust-bin.stable.latest.default.override {
              extensions = [
                "rust-src"
                "rust-analyzer"
                "rustfmt"
                "clippy"
              ];
	    })
            (pkgs.rosPackages.humble.buildEnv {
              paths = build_dependencies ++ messages;
            })
          ];

	  shellHook = ''
            export SHELL=${pkgs.bashInteractive}/bin/bash
          '';
        };


	packages.default =
          let
            addDeps = list: { ... }: {
              nativeBuildInputs = list ++ build_dependencies;
            };
            custom_crate_config = pkgs: pkgs.buildRustCrate.override {
              defaultCrateOverrides = pkgs.defaultCrateOverrides // {
                r2r_rcl = addDeps [ pkgs.rosPackages.humble.ros-core ];
                r2r_msg_gen = addDeps ([ pkgs.rosPackages.humble.ros-core ] ++ messages);
                r2r_actions = addDeps [ pkgs.rosPackages.humble.ros-core ];
                r2r = addDeps ([ pkgs.rosPackages.humble.ros-core ] ++ messages);
              };
            };
            cargo_nix = pkgs.callPackage ./Cargo.nix {
              buildRustCrateForPkgs = custom_crate_config;
            };
          
            executable = cargo_nix.rootCrate.build.overrideAttrs {
              propagatedBuildInputs = messages;
            };
          in
	(pkgs.callPackage ../rosify_package {
          pname = "create_cmd_vel";
          version = executable.version;
          to_rosify = executable;
          message_packages = messages;
          description = "Converts velocity commands into a format the iRobot Create 2 can understand";
          license = "Apache-2.0";
        });
      }
    );
}
