{ pkgs ? import <nixpkgs> { }, extra_pkgs ? [ ] }:
pkgs.python3.withPackages (
  python-pkgs: [
    python-pkgs.websockets
    python-pkgs.cbor2
    python-pkgs.pyparsing
    python-pkgs.voluptuous
  ] ++ extra_pkgs
)
