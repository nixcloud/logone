{ system ? builtins.currentSystem }:
let
  nixpkgsSrc = builtins.fetchTarball {
    url = "https://github.com/NixOS/nixpkgs/archive/25.05.tar.gz";
    sha256 = "sha256:1915r28xc4znrh2vf4rrjnxldw2imysz819gzhk9qlrkqanmfsxd";
  };
  pkgs = import (nixpkgsSrc + "/pkgs/top-level/default.nix") {
    localSystem = { inherit system; };
  };
  lib = pkgs.lib;
  callPackage' = lib.callPackageWith (pkgs // lib // self );
  self = {
    a=callPackage' ./a.nix {};
    b=callPackage' ./b.nix {};
    c=callPackage' ./c.nix {};
    d=callPackage' ./d.nix {};
    e=callPackage' ./e.nix {};
    f=callPackage' ./f.nix {};
  };
in
  self
