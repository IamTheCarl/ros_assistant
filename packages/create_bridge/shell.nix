{ pkgs ? import <nixpkgs> { } }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
in
pkgs.mkShell {
  buildInputs = import ./build_dependencies.nix { pkgs = pkgs; };

  # Set environment variables
  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.webkitgtk}/lib/pkgconfig:$PKG_CONFIG_PATH"  
    export OPENSSL_NO_VENDOR=1
    export HOME_ASSISTANT_API_URL="ws://localhost:8123/api/websocket"
    # A token only used for testing.
  '';
}
