{ pkgs ? import <nixpkgs> { }, ... }:
pkgs.mkShell {
  buildInputs = [
    (import ../../../rass-cli { pkgs = pkgs; })
  ];
}
