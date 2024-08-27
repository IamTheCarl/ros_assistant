{ config, lib, pkgs, ... }:
let
  getSshKeys = username:
    lib.splitString "\n"
      (builtins.readFile
        (builtins.fetchurl
          "https://github.com/${username}.keys"));
in
{
  # Include configuration to generate a deployable boot drive.
  imports = [
    <nixpkgs/nixos/modules/installer/sd-card/sd-image-aarch64.nix>
  ];
  
  options = {
    # ROS Assistant configuration
    ros_assistant = {
      arch = "aarch64";
      image_output = "config.system.build.sdImage";
    };
  };

  config = {
    system.stateVersion = "24.04";

    boot.kernelPackages = pkgs.linuxPackages_rpi4;

    # Allows early (earlier) modesetting for the Raspberry Pi
    boot.initrd.availableKernelModules = [
      "vc4"
      "bcm2835_dma"
      "i2c_bcm2835"
    ];
  
    # Fix missing modules
    # https://github.com/NixOS/nixpkgs/issues/154163
    nixpkgs.overlays = [
      (final: super: {
        makeModulesClosure = x:
          super.makeModulesClosure (x // { allowMissing = true; });
      })
    ];

    # Networking
    systemd.network.enable = true;
    networking.useNetworkd = true;
    networking.hostName = "raspberry-pi";

    # SSH
    systemd.services.sshd.wantedBy = lib.mkForce [ "multi-user.target" ];
    services.openssh.enable = true;
    services.openssh.settings.PermitRootLogin = "yes";

    # Grabs your public keys from Github so you can log in.
    # Replace "IamTheCarl" with your github username, unless you want me logging into your Pi (I'd rather not).
    users.extraUsers.root.openssh.authorizedKeys.keys = getSshKeys "IamTheCarl";

    # Install system packages.
    environment.systemPackages = [
      pkgs.neovim
    ];

  };
}
