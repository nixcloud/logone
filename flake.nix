{
  description = "a flake to build logone for libnix cargo";
  inputs = {
    nixpkgs.url = "github:NixOS/nixpkgs/nixos-25.11";
    fenix.url   = "github:nix-community/fenix";
    cargo-libnix.url = "github:nixcloud/cargo";
  };
  outputs =
  { self, nixpkgs, flake-utils, fenix, cargo-libnix } @ inputs:
    flake-utils.lib.eachDefaultSystem
      (system:
        let
          project_root = ./.;
          pkgs = import nixpkgs {
            inherit system;
            overlays = [
              fenix.overlay
            ];
          };
          lib = pkgs.lib;
        in
        with pkgs;
        rec {
          packages = { inherit cargo-libnix; };
          devShells = {
            default = mkShell {
              buildInputs = [
                # to build cargo with 'cargo build'
                openssl
                pkg-config
                # git helper
                tig
                # the toolchain used
                fenix.packages.${system}.stable.rustc
                #fenix.packages.${system}.stable.cargo
                cargo-libnix.packages.${system}.cargo-libnix
                fenix.packages.${system}.stable.rust-src
                fenix.packages.${system}.stable.rustfmt
                fenix.packages.${system}.stable.clippy
              ];
              shellHook = ''
                export CARGO_BACKEND=nix
              '';
            };  
          };
        }
      );
}
