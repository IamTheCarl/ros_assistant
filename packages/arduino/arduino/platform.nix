# Build an Arduino platform repository
{ pkgs ? import <nixpkgs> { }, index_url, index_sha256 }:
let
  index_file = builtins.fromJSON (builtins.readFile (builtins.fetchurl {
    url = index_url;
    sha256 = index_sha256;
  }));
  generate_platform = platform:
    let
      package_zip = builtins.fetchurl
        {
          url = platform.url;
          sha256 = platform.checksum;
        };
    in
    {
      name = platform.name;
      value = {
        platform.version = {
          meta = { };
          deriviation = pkgs.stdenv.mkDerivation
            {
              pname = platform.name + "(" + platform.version + ")";
              version = platform.version;

              buildPhase = ''
                mkdir -p $out/share
                cp ${package_zip} $out/share
              '';
            };
        };
      };
    };

  generate_package = package: {
    name = package.name;
    value = {
      mantainer = package.mantainer;
      websiteURL = package.websiteURL;
      email = package.email;
      help = package.help;
      platforms = map generate_platform package.platforms;
      # TODO tools
    };
  };
  packages = map generate_package index_file.packages;
in
builtins.listToAttrs packages
