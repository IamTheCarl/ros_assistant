# Build an Arduino platform repository
{ pkgs ? import <nixpkgs> { }, index_url, index_sha256 }:
let
  # TODO we should have the user provide the file path so that they can be commited to a trackable repo.
  index_file = builtins.fromJSON (builtins.readFile (builtins.fetchurl {
    url = index_url;
    sha256 = index_sha256;
  }));

  download_package = package: destination:
  pkgs.stdenv.mkDerivation {
    name = package.name;
    src = builtins.fetchurl {
      url = package.url;
      sha256 = pkgs.lib.replaceStrings ["MD5" "SHA-1" "SHA-256"] ["md5" "sha1" "sha256"] package.checksum;
    };

    nativeBuildInputs = [ pkgs.unzip ];

    installPhase = ''
      mkdir -p $out/${destination}
      unzip $src -d $out/${destination}
    '';
  };

  generate_platform = platform:
  {
    name = platform.name;
    value = {
      "${platform.version}" = platform // {
        derivation = download_package platform "packages/${platform.name}/hardware/${platform.architecture}/${platform.version}";
      };
    };
  };
  generate_system_tool = system: tool: platform_name:
  {
    name = system.host;
    value = download_package (system // { name = tool.name; }) "packages/${platform_name}/tools/${tool.name}/${tool.version}";
  };
  generate_tool = tool: platform_name:
  {
    name = tool.name;
    value = {
      "${tool.version}" = {
        systems = builtins.listToAttrs (map (system: generate_system_tool system tool platform_name)  tool.systems);
      };
    };
  };

  generate_package = package: {
    name = package.name;
    value = package // {
      platforms = builtins.listToAttrs (map generate_platform package.platforms);
      tools = builtins.listToAttrs (map (tool: generate_tool tool package.name) package.tools);
    };
  };
  packages = map generate_package index_file.packages;
in
builtins.listToAttrs packages
