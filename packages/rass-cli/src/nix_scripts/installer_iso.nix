{ config, pkgs ? import <nixpkgs> {}, lib, ... }: 
let
  # Evaluate the config for the system we are to deploy.
  evaluated_config = (import (<nixpkgs/nixos/lib/eval-config.nix>) {
    system = target_arch;
    modules = [ target_config ];
  }).config;

  # Get the image we are to deploy.
  raw_image = evaluated_config.system.build.raw;

  # Make a derivation to provide that image during installation.
  raw_image_package = pkgs.stdenv.mkDerivation {
    name = "raw-image";
    src = evaluated_config.system.build.raw;

    installPhase = ''
      mkdir -p $out/share
      cp -r $src/nixos.img $out/share
    '';
  };

  # Provides an installer for the raw image.
  install_script = pkgs.writeScriptBin "install_script" ''
    #!${pkgs.bash}/bin/bash
    dd if=${raw_image_package}/share/nixos.img of=${target_device}
  '';
in
{
  imports = [
    <nixpkgs/nixos/modules/installer/cd-dvd/installation-cd-minimal.nix> 
  ];

  # Makes the console print to the main display.
  services.journald.console = "/dev/tty1";

  systemd.services.install = {
    description = "Install system";
    wantedBy = [ "multi-user.target" ];
    after = [ "network.target" "polkit.service" ];
    path = [ raw_image_package install_script ];

    script = ''
      install_script
      ${pkgs.systemd}/bin/systemctl poweroff
    '';
  };
}