{
  rustPlatform,
  lib,
  pkg-config,
  openssl,
}: let
  inherit (lib.sources) sourceByRegex;
  inherit (builtins) fromTOML readFile;
  src = sourceByRegex ../. ["Cargo.*" "(src|data)(/.*)?"];
  cargoPackage = (fromTOML (readFile ../Cargo.toml)).package;
in
  rustPlatform.buildRustPackage {
    pname = cargoPackage.name;
    inherit (cargoPackage) version;
    inherit src;

    buildInputs = [openssl];

    nativeBuildInputs = [pkg-config];

    doCheck = false;

    cargoLock = {
      lockFile = ../Cargo.lock;
    };

    meta.mainProgram = "api-test";
  }
