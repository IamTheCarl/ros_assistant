{ pkgs ? import <nixpkgs> { } }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
  rust = import ../../nix/rust.nix { pkgs = pkgs; };
in
pkgs.mkShell {
  buildInputs = [
    rust

    # ROS related stuff.
    pkgs.rosPackages.humble.ros-core
  ];

  # Set environment variables
  shellHook = ''
    export PKG_CONFIG_PATH="${pkgs.webkitgtk}/lib/pkgconfig:$PKG_CONFIG_PATH"  
    export OPENSSL_NO_VENDOR=1
    export HOME_ASSISTANT_API_URL="ws://localhost:8123/api/websocket"
    # A token only used for testing.
  '';
}
