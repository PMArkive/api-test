prev: final: {
  demostf-api-test = final.callPackage ./package.nix {};
  demostf-api-test-docker = final.callPackage ./docker.nix {};
}
