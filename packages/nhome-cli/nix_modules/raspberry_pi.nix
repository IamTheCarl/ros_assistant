# This module depends on the (nixpkgs + "/nixos/modules/installer/sd-card/sd-image-aarch64.nix") module being
# loaded into your system config before this module. 
{ config, pkgs ? import <nixpkgs> {}, lib, ... }: 
{
  # This is where rhome-cli expects to find the disk image, so we need to point that to the sdImage.
  system.build.raw = config.system.build.sdImage;
  
  # Allows early (earlier) modesetting for the Raspberry Pi
  boot.initrd.availableKernelModules = [
    "vc4"
    "bcm2835_dma"
    "i2c_bcm2835"
  ];
}
