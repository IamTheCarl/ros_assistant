{ pkgs ? import <nixpkgs> { } }:
let
  packages = pkgs.callPackage ../../../../arduino/packages.nix {
    platform_indexes = [
      (builtins.fetchurl {
        url = "https://downloads.arduino.cc/packages/package_index.json";
        sha256 = "1wnc9vla47kpsqn14mm086m32avmrz9a6bnca6ms24xq4v96padz";
      })
      (builtins.fetchurl {
        url = "https://arduino.esp8266.com/stable/package_esp8266com_index.json";
        sha256 = "1svdbgj99xqwav9zn9d7yk2h8v49g7farznj23ad93vbciq438q8";
      })
    ];
  };

  build_arduino = import ../../../../arduino/build.nix {
    inherit pkgs;
  };
in
build_arduino {
  pname = "blink";
  version = "1.0.0";
  fqbn = "esp8266:esp8266:nodemcu";
  inherit packages;
  platforms = [
    packages.platforms.esp8266.esp8266."3.1.2"
  ];

  src = ./.;
}
# packages
