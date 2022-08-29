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
        rust = pkgs.rust-bin.nightly."2022-08-25".default;
      in
      with pkgs;
      {
        devShell = mkShell {
          name = "solana";
          buildInputs = [
            rust
            openssl
            pkg-config
            clang
          ];
          LIBCLANG_PATH = "${llvmPackages.libclang.lib}/lib";
        };
      }
    );
}
