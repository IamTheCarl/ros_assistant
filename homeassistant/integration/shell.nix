{ pkgs ? import <nixpkgs> { } }:
let
  pkgs = import ../../nix/ros.nix { pkgs = pkgs; };
  # mach-nix = import
  #   (builtins.fetchGit {
  #     url = "https://github.com/DavHau/mach-nix";
  #     ref = "refs/tags/3.5.0";
  #   })
  #   { };
  poetry2nix = import
    (pkgs.fetchFromGitHub {
      owner = "nix-community";
      repo = "poetry2nix";
      rev = "2024.9.1912527";
      sha256 = "sha256-OUpbkEbpkRh91WX8XYq5rugIHhzJUS84qwvCd+HBer8=";
    })
    {
      inherit pkgs;
    };
  homeassistant_tarball = pkgs.python3Packages.fetchPypi {
    pname = "homeassistant";
    version = "2024.8.3";
    sha256 = "sha256-iGAH6hxLgqAEovI82W0FREw7nqgX+4J+gm4cCGIS7h4=";
  };
  homeassistant_directory = pkgs.runCommand "homeassistant-src"
    {
      buildInputs = [ pkgs.poetry ];
    } ''
    mkdir -p $out
    tar -xzf ${homeassistant_tarball} -C $out --strip-components=1
    cd $out
    poetry lock
  '';
in
pkgs.mkShell {
  buildInputs = [
    pkgs.rosPackages.humble.ros-core
    (poetry2nix.mkPoetryEnv {
      projectDir = homeassistant_directory;
      python = pkgs.python311;
    })
    # (mach-nix.mkPython {
    #   python = "python3";

    #   requirements = ''
    #     websockets
    #     cbor2
    #     homeassistant==2023.1.0
    #   '';
    # })
    # (pkgs.python3.withPackages
    #   (
    #     python-pkgs: [
    #       python-pkgs.websockets
    #       python-pkgs.cbor2
    #       python-pkgs.voluptuous
    #       (# TODO attempting to include home assistant into the dev enviroment for better code recommendations.
    #         pkgs.python3.pkgs.buildPythonPackage rec {
    #           format = "pyproject";
    #           pname = "homeassistant";
    #           version = "2024.8.3";
    #           src = pkgs.python3Packages.fetchPypi {
    #             inherit pname version;
    #             sha256 = "sha256-iGAH6hxLgqAEovI82W0FREw7nqgX+4J+gm4cCGIS7h4=";
    #           };
    #           prePatch = ''
    #             # pythonRelaxDepsHook did not work
    #             substituteInPlace pyproject.toml \
    #               --replace-fail "setuptools==69.2.0" "setuptools>=69.2.0"
    #           '';

    #           # prePatch = ''
    #           #   # pythonRelaxDepsHook did not work
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "setuptools==69.2.0" "setuptools>=69.2.0"

    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "aiohttp==3.10.5" "aiohttp==3.9.5"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "aiohttp-fast-zlib==0.1.1" "aiohttp-fast-zlib==0.1.0"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "aiozoneinfo==0.2.1" "aiozoneinfo==0.2.0"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "astral==2.2" "astral==3.2"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "async-interrupt==1.2.0" "async-interrupt==1.1.1"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "awesomeversion==24.6.0" "awesomeversion==24.2.0"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "bcrypt==4.1.3" "bcrypt==4.1.2"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "fnv-hash-fast==1.0.2" "fnv-hash-fast==0.5.0"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "home-assistant-bluetooth==1.12.2" "home-assistant-bluetooth==1.12.0"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "PyJWT==2.9.0" "pyjwt==2.8.0"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "cryptography==43.0.0" "cryptography==42.0.5"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "Pillow==10.4.0" "pillow==10.3.0"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "pyOpenSSL==24.2.1" "pyopenssl==24.1.0"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "orjson==3.10.7" "orjson==3.10.3"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "PyYAML==6.0.2" "pyyaml==6.0.1"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "SQLAlchemy==2.0.31" "sqlalchemy==2.0.30"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "typing-extensions>=4.12.2,<5.0" "typing-extensions==4.11.0"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "ulid-transform==1.0.2" "ulid-transform==0.9.0"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "urllib3>=1.26.5,<2" "urllib3==2.2.1"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "voluptuous==0.15.2" "voluptuous==0.15.1"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "voluptuous-openapi==0.0.5" "voluptuous-openapi==0.0.4"
    #           #   substituteInPlace pyproject.toml \
    #           #     --replace-fail "yarl==1.9.7" "yarl==1.9.4" 
    #           # '';

    #           propagatedBuildInputs = [ python-pkgs.setuptools ];
    #           dependencies = [
    #             python-pkgs.aiodns
    #             python-pkgs.aiohttp
    #             python-pkgs.aiohttp-cors
    #             python-pkgs.aiohttp-fast-zlib
    #             python-pkgs.aiozoneinfo
    #             python-pkgs.astral
    #             python-pkgs.async-interrupt
    #             python-pkgs.attrs
    #             python-pkgs.atomicwrites-homeassistant
    #             python-pkgs.awesomeversion
    #             python-pkgs.bcrypt
    #             python-pkgs.certifi
    #             python-pkgs.ciso8601
    #             python-pkgs.fnv-hash-fast
    #             python-pkgs.hass-nabucasa
    #             python-pkgs.httpx
    #             python-pkgs.home-assistant-bluetooth
    #             python-pkgs.ifaddr
    #             python-pkgs.jinja2
    #             python-pkgs.lru-dict
    #             python-pkgs.pyjwt
    #             python-pkgs.cryptography
    #             python-pkgs.pillow
    #             python-pkgs.pyopenssl
    #             python-pkgs.orjson
    #             python-pkgs.pip
    #             python-pkgs.psutil-home-assistant
    #             python-pkgs.python-slugify
    #             python-pkgs.pyyaml
    #             python-pkgs.requests
    #             python-pkgs.sqlalchemy
    #             python-pkgs.typing-extensions
    #             python-pkgs.ulid-transform
    #             python-pkgs.urllib3
    #             python-pkgs.voluptuous
    #             python-pkgs.voluptuous-serialize
    #             python-pkgs.voluptuous-openapi
    #             python-pkgs.yarl
    #           ];
    #         }
    #       )
    #     ]
    #   ))
  ];

  shellHook = ''
    eval "$(register-python-argcomplete ros2)"
    eval "$(register-python-argcomplete colcon)"
    eval "$(register-python-argcomplete rosidl)"
  '';
}
