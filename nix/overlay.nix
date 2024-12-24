prev: final: {
  demostf-api = final.callPackage ./demostf-api.nix {};
  demostf-parser = final.callPackage ./demostf-parser.nix {};
  demostf-api-test = final.callPackage ./package.nix {};
}
