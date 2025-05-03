{
  php,
  fetchFromGitHub,
}: let
  phpWithExtensions = php.withExtensions ({
    enabled,
    all,
  }:
    enabled ++ (with all; [pdo apcu]));
in
  phpWithExtensions.buildComposerProject (finalAttrs: {
    pname = "demostf-api";
    version = "0.1.0";

    src = fetchFromGitHub {
      owner = "demostf";
      repo = "api";
      rev = "52b9bc09fd187bf3bbc11dbe4443af0e07a716a4";
      hash = "sha256-iMU7C2g76w5d5CtxRW5H50FxY+UlOc5gSpjxxbVyqUw=";
    };

    vendorHash = "sha256-EYWCR2aJAoyWvEX+SML4Fb3F3KGcUtwCgqhAGT6ZjZ4=";

    composerStrictValidation = false;

    doCheck = false;
  })
