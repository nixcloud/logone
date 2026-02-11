# generated from rustc-call.nix.handlebars using cargo (manual edits won't be persistent)
{ pkgs }:
  pkgs.stdenv.mkDerivation rec {
    name = "f";

    passthru.rust_crate_libraries = [];

    src = builtins.filterSource
    (path: type:
        let base = baseNameOf path;
        in !(base == "target" || base == "result" || builtins.match "result-*" base != null)
    ) ./.;
    
    phases = [ "buildPhase" ];
    unpackPhase = "";

    buildPhase = ''
      echo "@cargo {\"type\":0, \"crate_name\": \"${name}\", \"crate_type\": \"(lib)\"}"

      echo ${name}
      sleep 2
      echo ${name}
      sleep 2
      echo ${name}
      echo "hi"
      sleep 2
      echo ${name}
      sleep 2
      echo ${name}
      sleep 2
      echo "@cargo {\"type\":3, \"crate_name\": \"${name}\", \"crate_type\": \"(lib)\", \"exit_code\": 0, \"messages\": []}"
      mkdir -p $out
    '';

}

#@nix {"action":"result","fields":["@cargo {\"type\":0, \"crate_name\": \"f\", \"crate_type\": \"(lib)\"}"],"id":2216701340942343,"type":101}
#@nix {"action":"result","fields":["@cargo {\"type\":3, \"crate_name\": \"f\", \"crate_type\": \"(lib)\", \"exit_code\": 0, \"messages\": []}"],"id":2216701340942343,"type":101}
