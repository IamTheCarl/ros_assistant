{ config, pkgs ? import <nixpkgs> {}, lib, ... }:
let
  evalConfig = import (pkgs.path + "/nixos/lib/eval-config.nix");
  installer_base = pkgs.callPackage ./installer_base.nix {};
in
{
  options.installer_netboot = installer_base.options;

  config = {
    system.build.installer_netboot = let
      build = (evalConfig {
        system = pkgs.system;
        modules = [
          (import "${pkgs.path}/nixos/modules/installer/netboot/netboot-minimal.nix")
          ({ pkgs, lib, ...}: {
            netboot.squashfsCompression = "zstd -Xcompression-level 6";
          })
	  (installer_base.installer_system_module {
	    cfg = config.installer_netboot;
	    payload_config = config;
	  })
        ];
      }).config.system.build;
    in pkgs.runCommand "netboot" {} ''
        mkdir -p $out
        ln -s ${build.kernel} $out/kernel
        ln -s ${build.netbootRamdisk} $out/netbootRamdisk
        ln -s ${build.toplevel} $out/toplevel
    '';
  };
}
