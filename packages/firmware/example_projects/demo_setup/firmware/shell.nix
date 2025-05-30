{ pkgs ? import <nixpkgs> { } }:
let
  rust-overlay = import (builtins.fetchTarball "https://github.com/oxalica/rust-overlay/archive/stable.tar.gz");
  rust = pkgs.callPackage ./nix-libs/rust.nix {};
  rust_platform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };
  probe-rs = rust_platform.buildRustPackage rec {
    pname = "probe-rs-tools";
    version = "0.29.0";

    nativeBuildInputs = [ pkgs.pkg-config ];
    buildInputs = [ pkgs.udev ];

    src = pkgs.fetchCrate {
      inherit pname version;
      sha256 = "sha256-w/4D9HkEvM6oEd5hv2Vul3grVbx0hMxo7VIhtzxARU0=";
    };
    cargoHash = "sha256-/EgT3ZzPupCSPJ37R2TJ5RwHsPipWTat9FiQPeyuEzk=";

    # We have to skip the tests. They almost all pass but the doctest fails because
    # the README.md isn't included in the crate. There's no way to disable it at runtime.
    doCheck = false;
    checkFlags = [
      "--skip=config::registry::tests::add_targets_with_and_without_scanchain"
    ];
  };
  common_dependencies = pkgs.callPackage ./nix-libs/build_deps.nix {};
in
pkgs.mkShell {
  buildInputs = [
    rust
    pkgs.gdb
    pkgs.crate2nix
    probe-rs
  ] ++ common_dependencies;

  # Set environment variables
  shellHook = ''
    export OPENSSL_NO_VENDOR=1
  '';
}
