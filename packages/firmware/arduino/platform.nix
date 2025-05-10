# Build an Arduino platform repository
{ pkgs ? import <nixpkgs> { }, index_url, index_sha256 }:
let
  index_file = builtins.fromJSON (builtins.readFile (builtins.fetchurl {
    url = index_url;
    sha256 = index_sha256;
  }));
  generate_platform = platform:
    let
      package_zip = builtins.fetchurl {
        url = platform.url;
        sha256 = pkgs.lib.replaceStrings ["MD5" "SHA-1" "SHA-256"] ["md5" "sha1" "sha256"] platform.checksum;
      };
      platform_index_content = {
        # We only need the one package in our platform index.
        packages = [(platform // {
          # Override the package URL with one that lives in the Nix store.
          url = "file://${package_zip}";
        })];
      };
      platform_index_json = pkgs.writeText "platform_index.json" (pkgs.lib.generators.toJSON { } platform_index_content);
      # platform_index_file = pkgs.stdenv.mkDerivation ({
      #   pname = "platform_index_file";
      #   version = "1.0.0";

      #   unpackPhase = "true";
      # 
      #   nativeBuildInputs = [
      #     pkgs.arduino-cli
      #     pkgs.yj
      #   ];
      # 
      #   installPhase = ''
      #     mkdir -p $out
      #     cat ${platform_index_json} | yj -jy > $out/platform_index.yaml
      #     cat $out/platform_index.yaml
      #   '';
      # });

    in
    {
      name = platform.name;
      value = {
        "${platform.version}" = {
          meta = {
            platform = "${platform.name}:${platform.name} (${platform.version})";
	    platform_index_url = "file://${platform_index_json}";
	  };
          deriviation = pkgs.stdenv.mkDerivation {
            pname = "${platform.name} (${platform.version})";
            version = platform.version;
            unpackPhase = "true";

            installPhase = ''
              mkdir -p $out/share
              cp ${package_zip} $out/share
	      ls $out/share 
            '';
          };
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
      # TODO tools
    };
  };
  packages = map generate_package index_file.packages;
in
builtins.listToAttrs packages
