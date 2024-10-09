{ pkgs ? import <nixpkgs> { } }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
  rust = import ../../nix/rust.nix { pkgs = pkgs; };
  rust_platform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
in
pkgs.mkShell {
  buildInputs = [
    (import ./. { pkgs = pkgs; })
  ];

  # Set environment variables
  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.webkitgtk}/lib/pkgconfig:$PKG_CONFIG_PATH"  
    export OPENSSL_NO_VENDOR=1
  '';
}
