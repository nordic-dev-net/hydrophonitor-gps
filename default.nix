{
  lib,
  rustPlatform,
  pkg-config,
  pkgs,
}:
rustPlatform.buildRustPackage {
  pname = "gps-recorder";
  version = "0.1.0";

  src = ./.;

  cargoLock = {
    lockFile = ./Cargo.lock;
    # outputHashes = {
    #     "gpsd_proto-0.7.0" = "9e263a841c06ccfb1a7ed8497d209a8b4a43df6753cd201a017680986721b859";
    # };
    allowBuiltinFetchGit = true;
  };
}