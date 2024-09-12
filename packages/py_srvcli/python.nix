{ pkgs ? import <nixpkgs> { }, extra_pkgs ? [ ] }:
pkgs.python3.withPackages (
  python-pkgs: [
  ] ++ extra_pkgs
)
