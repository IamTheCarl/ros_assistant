{ pkgs ? import <nixpkgs> { } }:
pkgs.fetchFromGitHub {
  owner = "lopsided98";
  repo = "nix-ros-overlay";
  rev = "4072d6ed51d9053d2cc85c0ec4f69884cc99f392";
  hash = "sha256-xvre4xiezhPhU794dpjcHusVLBIfA4jPg9Kq439x4yY=";
}

# fetchTarball "https://github.com/lopsided98/nix-ros-overlay/archive/20e7f58b7377ed3e932e8924d9ee39efaf888aa8.tar.gz"
# fetchTarball "https://github.com/IamTheCarl/nix-ros-overlay/archive/usb-cam-fix.tar.gz"
