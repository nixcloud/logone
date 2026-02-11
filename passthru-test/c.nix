# generated from rustc-call.nix.handlebars using cargo (manual edits won't be persistent)
{ pkgs, b }:
let
  rustc_linker_arguments = rust_crate_libraries: builtins.concatStringsSep " " (map (lib: "-L ${lib}") rust_crate_libraries);

  in
  pkgs.stdenv.mkDerivation rec {
    name = "c";

    passthru.rust_crate_libraries = [b];

    src = builtins.filterSource
    (path: type:
        let base = baseNameOf path;
        in !(base == "target" || base == "result" || builtins.match "result-*" base != null)
    ) ./.;
phases = [ "buildPhase" ];
    unpackPhase = "";
    buildInputs = [ b ];

    buildPhase = ''
      echo "@cargo {\"type\":0, \"crate_name\": \"${name}\", \"crate_type\": \"(lib)\"}"

       echo ${name}
       sleep 2
       echo ${name}
       mkdir -p $out
       echo "${rustc_linker_arguments passthru.rust_crate_libraries}" > $out/c
       sleep 2
       echo ${name}
       sleep 2
       echo ${name}
       sleep 2
       echo ${name}
       sleep 2
      echo "@cargo {\"type\":3, \"crate_name\": \"${name}\", \"crate_type\": \"(lib)\", \"exit_code\": 0, \"messages\": []}"
    '';

}
