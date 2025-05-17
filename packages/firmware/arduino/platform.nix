# Build an Arduino platform repository
{ pkgs ? import <nixpkgs> { }, lib, stdenv, platform_indexes }:
let
  merge_sets = set_list: lib.fold (set: accumulator: lib.attrsets.recursiveUpdate accumulator set) {} set_list;
  index_files = map (index: builtins.fromJSON (builtins.readFile index)) platform_indexes;
  download_package = package: destination:
  pkgs.stdenv.mkDerivation {
    name = package.name;
    src = builtins.fetchurl {
      url = package.url;
      sha256 = pkgs.lib.replaceStrings ["MD5" "SHA-1" "SHA-256"] ["md5" "sha1" "sha256"] package.checksum;
    };

    nativeBuildInputs = [ pkgs.unzip ];

    buildPhase = ''
      if [[ "$src" == *.zip ]]; then
        unzip $src
      else
        tar xf $src
      fi
    '';

    installPhase = ''
      mkdir -p $out/${destination}
      cp -r * -d $out/${destination}
    '';
  };

  tools = merge_sets (
    map (
      index_file:
        merge_sets (
          map (
	    package: {
	      "${package.name}" = merge_sets (
                map (tool: generate_tool tool package.name) package.tools
	      );
	    }
	  )
	  index_file.packages
        )
    )
    index_files
  );
  generate_system_tool = system: tool: platform_name:
    download_package (system // { name = tool.name; }) "packages/${platform_name}/tools/${tool.name}/${tool.version}";
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
    pattern_set = regex_patterns."${stdenv.hostPlatform.system}";
    find_first_match = pattern: (lib.findFirst ({host, ...}: (builtins.match pattern host) != null) null systems);
    find_first_multi_match = patterns:
      if patterns == [] then
        throw "Could not find host for this tool"
      else
        let
          match = find_first_match (builtins.head patterns);
	in
        if match != null then
	  match
        else
	  find_first_multi_match (lib.lists.drop 1 patterns);
  in
    find_first_multi_match pattern_set;
  generate_tool = tool: platform_name:
  {
    "${tool.name}" = {
      "${tool.version}" = generate_system_tool (select_system tool.systems) tool platform_name;
    };
  };

  platforms = merge_sets (
    map (
      index_file:
        merge_sets (
          map (
	    package: {
	      packages."${package.name}" = merge_sets (
                map generate_platform package.platforms
	      );
	    }
	  )
	  index_file.packages
        )
    )
    index_files
  );
  generate_platform = platform:
  {
    "${platform.name}" = {
      "${platform.version}" = platform // {
        derivation = download_package platform "packages/${platform.name}/hardware/${platform.architecture}/${platform.version}";
	toolsDependencies = map ({packager, name, version}: tools.${packager}.${name}.${version}) platform.toolsDependencies;
      };
    };
  };
in
platforms
# tools
