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

  # From: https://github.com/arduino/arduino-cli/blob/dd621eecd5bc9d4a52edb8c06eea2550d3f12e29/internal/arduino/cores/tools.go#L129
  # Insparation borrowed from: https://github.com/bouk/arduino-nix/blob/dd6c6f4de7d8d8bb460508de911c120dfc35b709/lib.nix#L22
  select_system = systems: 
  let
    regex_patterns = {
      "aarch64-darwin" = [ "arm64-apple-darwin.*" "x86_64-apple-darwin.*" "i[3456]86-apple-darwin.*" ];
      "x86_64-darwin" = [ "x86_64-apple-darwin.*" "i[3456]86-apple-darwin.*" ];
      "i686-darwin" = [ "i[3456]86-apple-darwin.*" ];
      "aarch64-linux"  = [ "(aarch64|arm64)-linux-gnu.*" ];
      "x86_64-linux" = [ "x86_64-.*linux-gnu.*" ];
    };
    pattern_set = regex_patterns."${pkgs.stdenv.hostPlatform.system}";
    find_first_match = pattern: pkgs.lib.findFirst ({host, ...}: (builtins.match pattern host) != null) null systems;
    find_first_multi_match = patterns:
      if patterns == [] then
        throw "Could not find host for this tool"
      else
        let
          match = find_first_match (builtins.head patterns) systems;
	in
        if match != null then
	  match
        else
	  find_first_multi_match (builtins.tail patterns) systems;
  in
    find_first_match pattern_set;


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
