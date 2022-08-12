{

  inputs = {
    nixpkgs.url = "github:nixos/nixpkgs/nixos-unstable";
    flake-utils.url = "github:numtide/flake-utils";
    dependency-refresh.url = "github:yanganto/dependency-refresh";
    near.url = "path:near";
    solana.url = "path:solana";
  };

  outputs = { self, nixpkgs, flake-utils, dependency-refresh, near, solana }:
    flake-utils.lib.eachDefaultSystem (system:
      let
        pkgs = import nixpkgs {
          inherit system;
        };
        dr = dependency-refresh.defaultPackage.${system};
        updateDependencyScript = pkgs.writeShellScriptBin "update-dependency" ''
          dr ./$1/Cargo.toml
          if [ -f "Cargo.toml.old" ]; then
            rm Cargo.toml.old
            exit 1
          fi
        '';
      in
      with pkgs;
      {
        packages.${system} = {
          inherit solana near;
        };
        devShell = mkShell {
          name = "ci";
          buildInputs = [
            dr
            updateDependencyScript
          ];
        };
      }
    );
}
