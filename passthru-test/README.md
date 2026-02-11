This is a small test-lab for logone and parallel evaluation:

    nix build --file default.nix  -L --json --log-format internal-json 2>&1 | ~/logone/target/debug/logone --json --level cargo

