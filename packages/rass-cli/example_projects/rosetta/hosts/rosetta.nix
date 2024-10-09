{ config, lib, pkgs, ... }:
let
  ros_tarball = import ../../../../../nix/ros_tarball.nix { };
  ros_pkgs = import ../../../../../nix/ros.nix { pkgs = pkgs; };
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
    ((ros_tarball) + "/modules")
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
    systemd.network = {
      enable = true;
      # Switching to network manager will cause this to block until it eventually fails.
      wait-online.enable = false;
    };
    networking =
      {
        # Disable networkd and use networkmanager instead.
        # It's better suited for on-the-fly wifi configuration, which I don't want to bake into this
        # configuration file.
        useNetworkd = false;
        networkmanager.enable = true;
        hostName = "rosetta";
      };

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
      (ros_pkgs.rosPackages.humble.buildEnv {
        paths = [
          ros_pkgs.rosPackages.humble.ros-core
          (import ../../../../create_bridge { pkgs = pkgs; })
        ];
      })

    ];

    services.ros2 = {
      enable = true;
      distro = "humble";
      domainId = 0;
      nodes = {
        create_bridge = {
          package = "create_bridge";
          env = (import ../../../../create_bridge { pkgs = pkgs; });
          node = "create_bridge";
          args = [ ];
          rosArgs = [ ];
          params = {
            serial_device = "/dev/serial/by-id/usb-FTDI_FT231X_USB_UART_DA01NM8I-if00-port0";
            baud_rate = "115200";
          };
        };
      };
    };

  };
}
