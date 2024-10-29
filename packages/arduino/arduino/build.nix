# Build an Arduino sketch
{ pkgs ? import <nixpkgs> { } }: args@{ pname, fqbn, platforms, ... }:
let
  sketch_config = {
    profiles = {
      ${pname} = {
        inherit fqbn;
        platforms = map (p: p.meta) platforms;
      };
    };
  };
  sketch_json = pkgs.writeText "sketch.json" (pkgs.lib.generators.toJSON { } sketch_config);
in
pkgs.stdenv.mkDerivation (args // {
  inherit pname;

  nativeBuildInputs = [
    pkgs.arduino-cli
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
})
