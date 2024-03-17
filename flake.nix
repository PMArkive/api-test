{
  inputs = {
    nixpkgs.url = "nixpkgs/release-23.11";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = {
    self,
    nixpkgs,
    flake-utils,
  }:
  flake-utils.lib.eachDefaultSystem (system: let
      overlays = [
        (import ./overlay.nix)
      ];
      pkgs = (import nixpkgs) {
        inherit system overlays;
      };
      tools = with pkgs; [
        rustc
        cargo
        bacon
        cargo-edit
        cargo-outdated
        clippy
        cargo-audit
        cargo-msrv
      ];
      dependencies = with pkgs; [
        openssl
        pkg-config
      ];
    in rec {
      packages = rec {
        inherit (pkgs) demostf-api-test;
        default = demostf-api-test;
      };
      devShells.default = pkgs.mkShell {
        nativeBuildInputs = tools ++ dependencies;
      };
    });
}
