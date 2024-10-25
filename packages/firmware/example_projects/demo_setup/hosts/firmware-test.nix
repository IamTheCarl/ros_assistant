{ config, lib, pkgs ? <nixpkgs> { }, modulesPath ? <nikpkgs/modules> { }, ... }:
let
  getSshKeys = username:
    lib.splitString "\n"
      (builtins.readFile
        (builtins.fetchurl
          "https://github.com/${username}.keys"));
in
{
  options = {
    # ROS Assistant configuration
    ros_assistant = {
      # The archtecture to compile the OS for.
      arch = "x86_64";

      # A lambda that will produce the OS disk image.
      image_output = "config.system.build.raw";

      # The device the unattended installation ISO will install the OS to.
      # You can leave this unspecified if you do not intend to do unattended installations.
      target_device = "/dev/sda";
    };
  };

  config = {
    system.stateVersion = "24.04";

    # Our image builder calls this to build the disk image, when that needs to be done.
    # This is ignored on ssh update deployments.
    # You can find this package in the nkspkgs repository. Read up on that. There are some
    # interesting features to better support environments like LXC and QEMU, for the rare
    # case that may be useful to you.
    system.build.raw = import "${toString modulesPath}/../lib/make-disk-image.nix" {
      inherit pkgs lib config;

      partitionTableType = "hybrid"; # Hybrid means we support both MBR and UEFI boot.
      label = "nixos";
      diskSize = "auto";
      format = "raw";
    };

    # This doesn't create filesystems, this just tells systemd to mount them.
    fileSystems = {
      "/" = {
        device = "/dev/disk/by-label/nixos";
        autoResize = true;
        fsType = "ext4";
      };
      "/boot" = {
        device = "/dev/disk/by-label/ESP";
        fsType = "vfat";
      };
    };

    boot = {
      # To make flashing go faster, we made the smallest image we possibly could.
      # This will expand it to use the whole drive on the first boot.
      growPartition = true;
      loader.grub = {
        device = "/dev/vda"; # Install the MBR to this drive.
        efiSupport = true; # We also suport UEFI.
        efiInstallAsRemovable = true;
      };
    };

    # Networking
    systemd.network.enable = true;
    networking.useNetworkd = true;
    networking.hostName = "firmware-test";

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
