{ pkgs ? import <nixpkgs> { } }:
let
  rust = pkgs.callPackage ./nix-libs/rust.nix {};
  rust_platform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };

  common_dependencies = pkgs.callPackage ./nix-libs/build_deps.nix {};
in
rust_platform.buildRustPackage rec {
  pname = "embedded_firmware_demo";
  version = "0.0.1";
 
  nativeBuildInputs = common_dependencies;

  src = ./.;
  cargoHash = "sha256-W02262jLC9KIUUyHKMu2LZE4DMawEoO4gUBYKUJjatE=";

  buildPhase = ''
    # This hack is needed to work around the fact that Nix is *very incistent* on using
    # Cargo-Auditable. It's incompatible with the rust-lld linker we need forthe ARM target.
    # We also have to pass the thumbv7em target ourselves because I haven't been able to
    # successfully bring up a proper Nix cross compiler platform for that target and we
    # really don't need the whole cross compiler target for this paticular use case.
    ${rust}/bin/cargo build --target thumbv7em-none-eabihf --offline --release
  '';

  installPhase = ''
    mkdir -p $out/bin
    cp target/thumbv7em-none-eabihf/release/dfu_demo $out/bin/
  '';
}
