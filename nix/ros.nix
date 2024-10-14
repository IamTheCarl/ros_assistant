{ pkgs ? import <nixpkgs> { } }:
import (import ./ros_tarball.nix { }) { pkgs = pkgs; }
# import ../../nix-ros-overlay { pkgs = pkgs; }
