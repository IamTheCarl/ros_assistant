{ pkgs ? import <nixpkgs> { }, ... }:
pkgs.mkShell {
  buildInputs = [
    (import ../../cli { pkgs = pkgs; })
  ];
}
