{
  description = "Generic x86 device example";

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs?ref=nixos-unstable";
  };
  outputs = { self, nixpkgs, ... }:
  {
    ros_assistant = {
      # The device the unattended installation ISO will install the OS to.
      # You can leave this unspecified if you do not intend to do unattended installations.
      generic-x86.target_device = "/dev/sda";
    };

    nixosConfigurations.generic-x86 = nixpkgs.lib.nixosSystem {  
      system = "x86_64-linux";
      modules = [
        ../../nix_modules/basic_boot.nix
	../../nix_modules/installer_iso.nix
	../../nix_modules/installer_netboot.nix
        ({ pkgs, lib, config, ...}: {
          system.stateVersion = "25.05";

          # Networking
          systemd.network.enable = true;
          networking.useNetworkd = true;
	  networking.hostName = "generic-x86";

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
