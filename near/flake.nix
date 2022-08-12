{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    rust-overlay.url = "github:oxalica/rust-overlay";
    flake-utils.url = "github:numtide/flake-utils";
  };

  outputs = { self, rust-overlay, nixpkgs, flake-utils }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        overlays = [ (import rust-overlay) ];
        pkgs = import nixpkgs {
          inherit system overlays;
        };
        rust = pkgs.rust-bin.stable."1.62.1".default.override {
          targets = [ "wasm32-unknown-unknown" ];
        };
      in
      with pkgs;
      {
        devShell = mkShell {
          name = "near";
          buildInputs = [
            rust
            openssl
            pkg-config
          ];
          RUST_BACKTRACE = 1;
        };
      }
    );
}
