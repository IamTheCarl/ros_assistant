{ config, pkgs ? import <nixpkgs> {}, lib, ... }:
let
  cfg = config.installer;
  evalConfig = import (pkgs.path + "/nixos/lib/eval-config.nix");
in
{
  options.installer = {
    device = lib.mkOption {
      type = lib.types.str;
      default = "/dev/sda";
      description = "Path to device to install OS image to";
    };
    console = lib.mkOption {
      type = lib.types.str;
      default = "/dev/tty1";
      description = "Terminal to output text to";
    };
  };

  config = {
    system.build.installer = (evalConfig {
      system = pkgs.system;
      modules = [
        (import "${pkgs.path}/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix")
        ({ pkgs, lib, ...}: {
          # Makes the console print to the main display.
          services.journald.console = cfg.console;

          systemd.services.install = {
            description = "Install system";
            wantedBy = [ "multi-user.target" ];
            after = [ "network.target" "polkit.service" ];
            path = [
              # Provides an installer for the raw image.
              (pkgs.writeScriptBin "install_script" ''
                #!${pkgs.bash}/bin/bash
                echo "Installing OS"
                dd if=${config.system.build.raw}/nixos.img of=${cfg.device}
              '')
            ];

            script = ''
              install_script
              ${pkgs.systemd}/bin/systemctl poweroff
            '';
          };
        })
      ];
    }).config.system.build.isoImage;
  };
}
