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

    # Raspberry Pi drivers are weird. Don't try to figure it out for yourself.
    # Someone already figured it out for you. This makes the bluetooth work.
    "${builtins.fetchGit { url = "https://github.com/NixOS/nixos-hardware.git"; }}/raspberry-pi/4"
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

        firewall.enable = true;
      };

    # Bluetooth
    hardware.raspberry-pi."4".bluetooth.enable = true;
    hardware.bluetooth =
      {
        enable = true;
        powerOnBoot = true;
      };

    # SSH
    systemd.services.sshd.wantedBy = lib.mkForce [ "multi-user.target" ];
    services.openssh.enable = true;
    services.openssh.settings.PermitRootLogin = "yes";

    # Grabs your public keys from Github so you can log in.
    # Replace "IamTheCarl" with your github username, unless you want me logging into your Pi (I'd rather not).
    users.extraUsers.root.openssh.authorizedKeys.keys = getSshKeys "IamTheCarl";

    # Install system packages.
    # None of these are necessary for autonomy to run. They are just debug tools.
    environment.systemPackages = [
      pkgs.neovim
      pkgs.htop
      pkgs.bluez
      pkgs.linuxConsoleTools
      pkgs.usbutils
      (ros_pkgs.rosPackages.humble.buildEnv {
        paths = [
          ros_pkgs.rosPackages.humble.ros-core
        ];
      })
    ];

    users.groups = {
      # Let ROS access serial interfaces.
      dialout.members = [ "ros" ];
      # Let ROS access human input devices (specifically, game pads)
      input.members = [ "ros" ];
      # Access video (specifically, realsense camera)
      video.members = [ "ros" ];
    };

    services.ros2 = {
      enable = true;
      distro = "humble";
      domainId = 0;
      nodes = {
        # Bridges ROS and the iRobot Create interface.
        create_bridge = {
          env = (ros_pkgs.rosPackages.humble.buildEnv {
            paths = [
              ros_pkgs.rosPackages.humble.ros-core
              (import ../../../../create_bridge { pkgs = pkgs; })
            ];
          });
          package = "create_bridge";
          node = "create_bridge";
          args = [ ];
          rosArgs = [ ];
          params = {
            serial_device = "\"/dev/serial/by-id/usb-FTDI_FT231X_USB_UART_DA01NM8I-if00-port0\"";
          };
        };
        # Provides joystick messages from a locally connected joystick.
        joy = {
          env = (ros_pkgs.rosPackages.humble.buildEnv {
            paths = [
              ros_pkgs.rosPackages.humble.ros-core
              ros_pkgs.rosPackages.humble.joy
            ];
          });
          package = "joy";
          node = "joy_node";
          args = [ ];
          rosArgs = [ ];
          params = { };
        };
        # Converts Joystick messages into velocity commands.
        teleop-twist-joy = {
          env = (ros_pkgs.rosPackages.humble.buildEnv {
            paths = [
              ros_pkgs.rosPackages.humble.ros-core
              ros_pkgs.rosPackages.humble.teleop-twist-joy
            ];
          });
          package = "teleop_twist_joy";
          node = "teleop_node";
          args = [ ];
          rosArgs = [ ];
          params = {
            # Documentation for these parameters:
            # https://docs.ros.org/en/ros2_packages/humble/api/teleop_twist_joy/standard_docs/README.html
            enable_button = "4";
            enable_turbo_button = "5";
            "axis_linear.x" = "1";
            "axis_angular.yaw" = "0";
            "scale_angular.yaw" = "1.0";
          };
        };
        # Converts velocity commands into Create 2 movement commands.
        create_cmd_vel = {
          env = (ros_pkgs.rosPackages.humble.buildEnv {
            paths = [
              ros_pkgs.rosPackages.humble.ros-core
              (import ../../../../create_cmd_vel { pkgs = pkgs; })
            ];
          });
          package = "create_cmd_vel";
          node = "create_cmd_vel";
          args = [ ];
          rosArgs = [ ];
          params = { };
        };
        # Not enough power for the real sense camera :(
        # We're going to use a cheap webcam instaed.
        webcam = {
          env = (ros_pkgs.rosPackages.humble.buildEnv {
            paths = [
              ros_pkgs.rosPackages.humble.ros-core
              ros_pkgs.rosPackages.humble.usb-cam
            ];
          });
          package = "usb_cam";
          node = "usb_cam_node_exe";
          args = [ ];
          rosArgs = [ ];
          params = { };
        };
      };
    };

  };
}
