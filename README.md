# motivation

this rust project implements a rust client to the `nix build --log-format internal-json` protocol.

it is also an attempt to make nix logging less verbose and more readable by humans by doing this:

* when building several drv(s) in parallel, it accumulate the individual logs and only outputs them when there is an error
* it prevents the print of the nix error summary where you have the error twice in the output but with the hideous indentation

# status

* [x] prototype as standalone binary works
  * [x] support for @nix
* [ ] minimalist logging
  * [x] support for custom extensions like @cargo
  * [ ] add 3 log levels: --level cargo|errors|verbose
    * [ ] "cargo" (default): ignores logs from @nix and uses @cargo logs and error detection
    * [ ] "errors" only use @nix logs, ignore @cargo messages for the failing build
    * [ ] "verbose" shows all @nix messages for each build: successful and failed ones but still in sorted blocks (no mixed logging), ignores all @cargo messages
  * [x] remove --timing / --min-time / --debug
  * [ ] print the @cargo internal logs properly & use the exit code
* [ ] make it a crate libary
* [ ] create demo with https://docs.asciinema.org/manual/server/embedding/
  * [ ] successful build
  * [ ] build with one error
  * [ ] build with two errors
  * [ ] build with many targets

a good illustration on how to @nix protocol works and can be implemented. it sometimes shows different results than `nix-output-monitor` and `nix` itself:

* https://github.com/nixos/nix/issues/13909
* https://github.com/nixos/nix/issues/13910
* https://github.com/nixos/nix/issues/13935

# examples

as an example there are outputs in the tests folder one can experiment with:

the 'normal' monitor:

    cat tests/example.stdin2 | cargo run -- --json

the 'verbose' monitor:

    cat tests/example.stdin2 | cargo run -- --json -v

# license

PUBLIC DOMAIN
