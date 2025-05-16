# Build an Arduino platform repository
{ pkgs ? import <nixpkgs> { }, index_url, index_sha256 }:
let
  # TODO we should have the user provide the file path so that they can be commited to a trackable repo.
  index_file = builtins.fromJSON (builtins.readFile (builtins.fetchurl {
    url = index_url;
    sha256 = index_sha256;
  }));

  download_package = package:
  pkgs.stdenv.mkDerivation {
    name = package.name;
    src = builtins.fetchurl {
      url = package.url;
      sha256 = pkgs.lib.replaceStrings ["MD5" "SHA-1" "SHA-256"] ["md5" "sha1" "sha256"] package.checksum;
    };

    nativeBuildInputs = [ pkgs.unzip ];

    buildPhase = ''
      mkdir -p $out
      unzip $src -d $out
    '';
  };

  generate_platform = platform:
  {
    name = platform.name;
    value = {
      "${platform.version}" = {
        package = download_package platform;
        tool_dependencies = if platform?toolDependencies then platform.toolDependencies else [];
      };
    };
  };
  generate_system_tool = system:
  {
    name = system.host;
    value = download_package system;
  };
  generate_tool = tool:
  {
    name = tool.name;
    value = {
      "${tool.version}" = {
        systems = builtins.listToAttrs (map generate_system_tool (map (system: system // { name = tool.name; }) tool.systems));
      };
    };
  };

  generate_package = package: {
    name = package.name;
    value = {
      maintainer = if package?maintainer then package.maintainer else null;
      websiteURL = if package?websiteURL then package.websiteURL else if package?url then package.url else null;
      email = if package?email then package.email else null;
      help = if package?help then package.help else null;
      platforms = builtins.listToAttrs (map generate_platform package.platforms);
      tools = builtins.listToAttrs (map generate_tool package.tools);
    };
  };
  packages = map generate_package index_file.packages;
in
builtins.listToAttrs packages
