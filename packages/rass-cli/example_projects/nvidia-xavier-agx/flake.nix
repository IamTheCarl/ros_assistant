{
  description = "NVidia Xavier AGX example";

  inputs = {
    # General Nixpkgs
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";

    # Jetpack specific drivers
    jetpack.url = "github:anduril/jetpack-nixos";

    # Used to target all major platforms when creating development shells
    flake-utils.url  = "github:numtide/flake-utils";
  };
  outputs = { self, nixpkgs, jetpack, flake-utils, ... }:
  {
    nixosConfigurations.nvidia-xavier-agx = nixpkgs.lib.nixosSystem {  
      system = "aarch64";
      modules = [
        (jetpack + "/modules/default.nix")
        ../../nix_modules/basic_boot.nix
        ({ pkgs, lib, config, ...}: {
          system.stateVersion = "25.05";
    
          # Configure up our hardware with NVidia stuff.
          hardware.nvidia-jetpack.enable = true;
          hardware.nvidia-jetpack.som = "xavier-agx";
          hardware.nvidia-jetpack.carrierBoard = "devkit";

          # Networking
          systemd.network.enable = true;
          networking.useNetworkd = true;

          # SSH
          systemd.services.sshd.wantedBy = pkgs.lib.mkForce [ "multi-user.target" ];
          services.openssh.enable = true;
          services.openssh.settings.PermitRootLogin = "yes";
          users.extraUsers.root.openssh.authorizedKeys.keys = lib.splitString "\n" (builtins.readFile ./public-keys);

          # Install system packages.
          environment.systemPackages = [
            pkgs.neovim
          ];
	})
      ];
    };
  } // (flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
      };
    in
    {
      # Provides development shell with tools needed to boostrap the devkit.
      devShells.default = pkgs.mkShell {
        # Flash utility for installing UEFI firmware.
        buildInputs = [
	  jetpack.packages.${system}.flash-xavier-agx-devkit
	  pkgs.usbutils
	];
      };
    }
  ));
} 
