# Build an Arduino sketch
{ pkgs ? import <nixpkgs> { } }: args@{ pname, fqbn, platforms, ... }:
let
  sketch_config = {
    profiles = {
      ${pname} = {
        inherit fqbn;
	platforms = map (platform: {
          platform = "${platform.name}:${platform.name} (${platform.version})";
	  platform_index_url = platform.url;
	}) platforms;
      };
    };
  };
  sketch_json = pkgs.writeText "sketch.json" (pkgs.lib.generators.toJSON { } sketch_config);
  filteredArgs = builtins.removeAttrs args [ "fqbn" "platforms" ];
  arduino-cli = import ./wrap-arduino-cli.nix {
    inherit pkgs;
    packages = platforms;
    libraries = [];
  };

  build_arguments = {
    nativeBuildInputs = [
      arduino-cli
      pkgs.yj
    ];
  
    postUnpack = ''
      cat ${sketch_json} | yj -jy > $sourceRoot/sketch.yaml
      cat $sourceRoot/sketch.yaml
      ls $sourceRoot
    '';
  
    buildPhase = ''
      arduino-cli compile --profile ${pname} --output-dir output
    '';
  
    installPhase = ''
      mkdir -p $out/share
      cp output/build.elf $out/share/payload.elf
    '';
  };
in
pkgs.stdenv.mkDerivation (filteredArgs // build_arguments)
