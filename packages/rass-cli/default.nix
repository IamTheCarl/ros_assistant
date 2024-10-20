{ pkgs ? import <nixpkgs> { }, ... }:
let
  rust = import ../../nix/rust.nix { pkgs = pkgs; };
  rust_platform = pkgs.makeRustPlatform {
    cargo = rust;
    rustc = rust;
  };

  cargo_toml = builtins.fromTOML (builtins.readFile ./Cargo.toml);

  ros_tunnel = (import ../dds_bridge { pkgs = pkgs; });
in
rust_platform.buildRustPackage rec {
  pname = "rass-cli";
  version = cargo_toml.package.version;

  src = ./.;

  cargoHash = "sha256-TFS1JV6HhNLE9GqOEkHvgZl2Kl9k5dffpHOSIMjyVY4=";

  checkFlags = [
    # Requires access to the nix store, which is not available during a build.
    "--skip=host_config::test::read_config"
  ];

  nativeBuildInputs = [
    pkgs.makeWrapper
  ];

  buildInputs = [
    pkgs.nix
    pkgs.nixos-rebuild
    pkgs.openssh
    ros_tunnel
  ];

  meta = with pkgs.lib; {
    description = "Command line interface for ROS Assistant";
    homepage = "https://github.com/IamTheCarl/ros_assistant";
    # license = licenses.mit; # TODO a license needs to be picked.
    maintainers = with maintainers; [ "James Carl" ];
  };

  postInstall = ''
    mv $out/bin/cli $out/bin/rass
    wrapProgram $out/bin/rass \
      --prefix PATH : ${pkgs.nix}/bin:${pkgs.nixos-rebuild}/bin:${pkgs.openssh}/bin:${ros_tunnel}/bin
  '';
}
