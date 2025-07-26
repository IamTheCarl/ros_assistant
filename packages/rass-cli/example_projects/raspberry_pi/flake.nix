{
  description = "Raspberry Pi example";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };
  outputs = { self, nixpkgs, ... }:
  {
    nixosConfigurations.raspberry-pi = nixpkgs.lib.nixosSystem {  
      system = "aarch64-linux";
      modules = [
        (nixpkgs + "/nixos/modules/installer/sd-card/sd-image-aarch64.nix")
        ../../nix_modules/raspberry_pi.nix	
	../../nix_modules/installer_iso.nix
        ({ pkgs, lib, config, ...}: {
          system.stateVersion = "25.05";
    
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
  };
}
