{ config, pkgs ? import <nixpkgs> {}, lib, ... }:
let
  evalConfig = import (pkgs.path + "/nixos/lib/eval-config.nix");
  installer_base = pkgs.callPackage ./installer_base.nix {};
in
{

  options.installer_isoboot = installer_base.options;

  config = {
    system.build.installer_iso = (evalConfig {
        system = pkgs.system;
        modules = [
          (import "${pkgs.path}/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix")
	  (installer_base.installer_system_module {
	    cfg = config.installer_isoboot;
	    payload_config = config;
	  })
        ];
      }).config.system.build.isoImage;
  };
}
