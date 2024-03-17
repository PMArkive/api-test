{
  rustPlatform,
  lib,
  pkg-config,
  openssl,
}: let
  inherit (lib.sources) sourceByRegex;
  inherit (builtins) fromTOML readFile;
  version = (fromTOML (readFile ./Cargo.toml)).package.version;
  src = sourceByRegex ./. ["Cargo.*" "(src|data)(/.*)?"];
in
  rustPlatform.buildRustPackage rec {
    pname = "demostf-api-test";

    inherit src version;

    buildInputs = [openssl];

    nativeBuildInputs = [pkg-config];

    doCheck = false;

    cargoLock = {
      lockFile = ./Cargo.lock;
    };
  }
