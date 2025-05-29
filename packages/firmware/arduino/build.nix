# Build an Arduino sketch
{ pkgs ? import <nixpkgs> { } }: args@{ pname, fqbn, packages, platforms, ... }:
let
  select_latest_version = attrs: (
    builtins.head (
      builtins.sort (
        a: b: (builtins.compareVersions a.version b.version) == 1
      )
      (
        builtins.attrValues (
	  builtins.mapAttrs (
	    version: value: {
	      inherit version value;
	    })
	  attrs
	)
      )
    )
  ).value;

  # Platforms don't specify dependencies on the builtins, so we just have to include those
  # ourselves. We will use the latest version available.
  arduino_builtins = map select_latest_version (builtins.attrValues packages.tools.builtin);

  filteredArgs = builtins.removeAttrs args [ "fqbn" "packages" "platforms" ];
  arduino-cli = import ./wrap-arduino-cli.nix {
    inherit pkgs;
    packages = arduino_builtins ++ platforms;
    libraries = [];
  };

  build_arguments = {
    nativeBuildInputs = [
      arduino-cli
      pkgs.python314
    ];
  
    buildPhase = ''
      arduino-cli compile --fqbn "${fqbn}" --output-dir output
    '';
  
    installPhase = ''
      mkdir -p $out/share
      cp output/build.elf $out/share/payload.elf
    '';
  };
in
pkgs.stdenv.mkDerivation (filteredArgs // build_arguments)
# { inherit arduino_builtins; }
