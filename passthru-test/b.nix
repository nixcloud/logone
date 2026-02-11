# generated from rustc-call.nix.handlebars using cargo (manual edits won't be persistent)
{ pkgs, a }:
  pkgs.stdenv.mkDerivation rec {
    name = "b";

    passthru.rust_crate_libraries = [ a ];

    src = builtins.filterSource
    (path: type:
        let base = baseNameOf path;
        in !(base == "target" || base == "result" || builtins.match "result-*" base != null)
    ) ./.;

    phases = [ "buildPhase" ];
    unpackPhase = "";

    buildInputs = [ a ];

    buildPhase = ''
      echo "@cargo {\"type\":0, \"crate_name\": \"${name}\", \"crate_type\": \"(lib)\"}"

       echo ${name}
       sleep 2
       echo ${name}
       sleep 2
       echo ${name}
       sleep 2
       echo ${name}
       sleep 2
       echo ${name}
       sleep 2
       mkdir -p $out
      echo "@cargo {\"type\":3, \"crate_name\": \"${name}\", \"crate_type\": \"(lib)\", \"exit_code\": 0, \"messages\": []}"
    '';

}
