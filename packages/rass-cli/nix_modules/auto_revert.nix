# Provides a service that will automatically revert to a previous derivation
# if rass fails to cancel the reversion in time. This is meant to be a convenient
# recovery tool if you accidentally update the configuration in a way that locks
# you out of your robot.
{ config, pkgs ? import <nixpkgs> {}, lib, ... }:
let
  cfg = config.auto-revert;
in
{
  options.auto-revert = {
    enable = lib.mkOption {
      type = lib.types.bool;
      default = true;
      description = "Enable auto-revert";
    };
    time = lib.mkOption {
      type = lib.types.int;
      default = 10;
      description = "Number of seconds before auto-revert is performed";
    };
  };

  config = {
    systemd.services.auto-revert = {
      enable = cfg.enable;
      description = "Auto revert on update failure";
      wantedBy = [ "multi-user.target" ];
      script = ''
        #!${pkgs.bash}/bin/bash -e
        if [ -f '/run/rass/auto-revert/set' ]; then
          echo "Auto revert timer set to ${toString cfg.time} second and started."
          ${pkgs.coreutils}/bin/sleep ${toString cfg.time}
        
          if [ -f '/run/rass/auto-revert/set' ]; then
            echo "Timer has expired. Reverting to previous generation."
	    cd /tmp
	    rm -f result
            ${pkgs.nixos-rebuild}/bin/nixos-rebuild \
              -I nixpkgs=/root/.nix-defexpr/channels \
              test --rollback --verbose --fast
            echo "System generation reverted"
            rm -f '/run/rass/auto-revert/set'
          else
            echo "Auto revert aborted."
          fi
        else
          echo "Auto-revert has not been enabled for this activation."
        fi
      '';
    };
  };
}
