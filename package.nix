{
  rustPlatform,
  lib,
  pkg-config,
  openssl,
}: let
  inherit (lib.sources) sourceByRegex;
in
  rustPlatform.buildRustPackage rec {
    pname = "demostf-api-test";
    version = "0.1.0";

    src = sourceByRegex ./. ["Cargo.*" "(src|data)(/.*)?"];

    buildInputs = [openssl];

    nativeBuildInputs = [pkg-config];

    doCheck = false;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };
  }
