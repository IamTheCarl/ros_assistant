{ pkgs ? import <nixpkgs> { }, packages ? [], libraries ? []}:
# Draws a lot of insparation from: https://github.com/bouk/arduino-nix/blob/dd6c6f4de7d8d8bb460508de911c120dfc35b709/wrap-arduino-cli.nix
let
  # Borrow Bouk's work.
  # arduino-nix = pkgs.fetchFromGitHub {
  #   owner = "bouk";
  #   repo = "arduino-nix";
  #   rev = "dd6c6f4de7d8d8bb460508de911c120dfc35b709";
  #   hash = "sha256-PGZVUTg8BQKL1QgTr3Fcia8RBYYqsbHbx9lIunqdmxQ=";
  # };
  # arduino-lib = import "${arduino-nix}/lib.nix" { lib = pkgs.lib; };


  # Arduino-cli will try to download this stuff if it isn't already present.
  # These dummies prevent that from happening.
  data_directory = pkgs.symlinkJoin {
    name = "arduino-data";
    paths = packages ++ [
      (pkgs.writeTextDir "inventory.yaml" (builtins.toJSON {}))
      (pkgs.writeTextDir "package_index.json" (builtins.toJSON {packages = [];}))
      (pkgs.writeTextDir "library_index.json" (builtins.toJSON {libraries = [];}))
    ];
    postBuild = ''
      mkdir -p $out/staging
    '';
  };
  user_directory = pkgs.symlinkJoin {
    name = "arduino-libraries";
    paths = libraries;
  };
in
pkgs.runCommand "arduino-cli-wrapped" {
  buildInputs = [ pkgs.makeWrapper ];
  meta.mainProgram = "arduino-cli";
  passthru = {
    inherit data_directory user_directory;
  };
} ''
  makeWrapper ${pkgs.arduino-cli}/bin/arduino-cli $out/bin/arduino-cli \
    --set ARDUINO_UPDATER_ENABLE_NOTIFICATION false \
    --set ARDUINO_DIRECTORIES_DATA ${data_directory} \
    --set ARDUINO_DIRECTORIES_USER ${user_directory}
''
