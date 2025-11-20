{
  description = "Rosetta the robot's configuration";

  # Build inputs are like PPAs for Apt.
  # They provide packages and configuration options you can use in this file, or your modules.
  inputs = {
    nix-ros-overlay.url = "github:lopsided98/nix-ros-overlay/master";
    nixpkgs.follows = "nix-ros-overlay/nixpkgs"; # We use the nixpkgs from the nix-ros-overlay.

    nixos-hardware.url = "github:NixOS/nixos-hardware/master";

    # Used to target all major platforms when creating development shells
    flake-utils.url  = "github:numtide/flake-utils";

    # Provides Create interface for Roomba.
    create-bridge .url = "../../../create_bridge";
    create-bridge.inputs.nix-ros-overlay.follows = "nix-ros-overlay";
    create-bridge.inputs.nixpkgs.follows = "nixpkgs";

    # Provides Create interface with cmd-vel topic. 
    create-cmd-vel.url = "../../../create_cmd_vel";
    create-cmd-vel.inputs.nix-ros-overlay.follows = "nix-ros-overlay";
    create-cmd-vel.inputs.nixpkgs.follows = "nixpkgs";

    ros_assistant.url = "../..";
    ros_assistant.inputs.nixpkgs.follows = "nixpkgs";
  };
  
  # Save yourself a lot of build time and use the ROS nix cache.
  # If you get warnings about not being a trusted user on NixOS, you need to add your account
  # name to `nix.settings.trusted-users`. This is not mandatory, it just grants you access
  # to a build cache for ROS related packages.
  nixConfig = {
    extra-substituters = [ "https://ros.cachix.org" ];
    extra-trusted-public-keys = [ "ros.cachix.org-1:dSyZxI8geDCJrwgvCOHDoAfOm5sV1wCPjBkKL+38Rvo=" ];
  };

  # Outputs are the products of our flake. We will produce a boot image for the Rosetta robot
  # and a development shell so that you can interact with the robot more easily.
  # Notice that each of our inputs are available as an argument to this. We can grab modules,
  # packages, and functions from them. We do all of these things below.
  outputs = {
    self,
    nix-ros-overlay,
    nixpkgs,
    nixos-hardware,
    flake-utils,
    create-bridge,
    create-cmd-vel,
    ros_assistant,
    ...
  }: {
    # You can have a configuration for each computer within the robot.
    nixosConfigurations.rosetta = nixpkgs.lib.nixosSystem
    {
      # Specify the target archetecture
      system = "aarch64-linux";

      # Modules are convinent libraries you can mix and match to add reusable features
      # to multiple computers, or even share functionality between multiple robots in a way
      # that composes well.
      modules = [
        (nixpkgs + "/nixos/modules/installer/sd-card/sd-image-aarch64.nix")
	../../nix_modules/raspberry_pi.nix
	nixos-hardware.nixosModules.raspberry-pi-4
	(nix-ros-overlay + "/modules")
        ({ pkgs, lib, config, ...}:
	{
	  nixpkgs.overlays = [ nix-ros-overlay.overlays.default ];
          system.stateVersion = "25.05";

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
    
          # Work around missing modules
          # https://github.com/NixOS/nixpkgs/issues/154163#issuecomment-2868994145
          hardware.enableAllHardware = lib.mkForce false;

          # SSH
          systemd.services.sshd.wantedBy = lib.mkForce [ "multi-user.target" ];
          services.openssh.enable = true;
          services.openssh.settings.PermitRootLogin = "yes";
          users.extraUsers.root.openssh.authorizedKeys.keys = lib.splitString "\n" (builtins.readFile ./public-keys);

          # Install system packages.
          # None of these are necessary for autonomy to run. They are just debug tools.
          environment.systemPackages = [
            pkgs.neovim
            pkgs.htop
            pkgs.bluez
            pkgs.linuxConsoleTools
            pkgs.usbutils
            (pkgs.rosPackages.humble.buildEnv {
              paths = [
                pkgs.rosPackages.humble.ros-core
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

          services.ros2 =
	  let
	    autonomy_env = (pkgs.rosPackages.humble.buildEnv {
              paths = [
                pkgs.rosPackages.humble.ros-core
                pkgs.rosPackages.humble.joy
                pkgs.rosPackages.humble.teleop-twist-joy
                create-bridge.packages.aarch64-linux.default
	        create-cmd-vel.packages.aarch64-linux.default
              ];
            });
	  in
	  {
            enable = true;
            distro = "humble";
            domainId = 0;
            nodes = {
              # Bridges ROS and the iRobot Create interface.
              create_bridge = {
                env = autonomy_env;
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
                env = autonomy_env;
                package = "joy";
                node = "joy_node";
                args = [ ];
                rosArgs = [ ];
                params = { };
              };
              # Converts Joystick messages into velocity commands.
              teleop-twist-joy = {
                env = autonomy_env;
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
                env = autonomy_env;
                package = "create_cmd_vel";
                node = "create_cmd_vel";
                args = [ ];
                rosArgs = [ ];
                params = { };
              };
            };
          };
	})
      ];
    };
  } // (flake-utils.lib.eachDefaultSystem (system:
    let
      pkgs = import nixpkgs {
        inherit system;
	overlays = [ nix-ros-overlay.overlays.default ];
      };
    in
    {
      # Provides development shell with tools needed to boostrap the devkit.
      devShells.default = pkgs.mkShell {
        buildInputs = [
          pkgs.rosPackages.humble.ros-core
          pkgs.rosPackages.humble.joy
          pkgs.rosPackages.humble.teleop-twist-joy
          pkgs.rosPackages.humble.rviz2
          ros_assistant.packages.${system}.default
          (pkgs.callPackage ../../../create_bridge_interface { })
	];
      };
    }
  ));
} 
