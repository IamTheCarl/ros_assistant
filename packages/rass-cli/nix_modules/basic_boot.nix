{ config, pkgs ? import <nixpkgs> {}, lib, ... }: 
{
  # Our image builder calls this to build the disk image, when that needs to be done.
  # This is ignored on ssh update deployments.
  # You can find this package in the nkspkgs repository. Read up on that. There are some
  # interesting features to better support environments like LXC and QEMU, for the rare
  # case that may be useful to you.
  system.build.raw = pkgs.callPackage (pkgs.path + "/nixos/lib/make-disk-image.nix") {      
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
}
