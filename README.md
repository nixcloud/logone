# motivation

this rust project 'logone' implements a rust program and crate library to consume the nix output produced by:

    `nix build --log-format internal-json`

it is also an attempt to make nix logging less verbose / nosiy

* when building several drv(s) in parallel, it accumulate the individual logs and only outputs them when there is an error
* it prevents the print of the nix error summary where you have the error twice in the output but with the hideous indentation

# status

* [x] prototype as standalone binary works
  * [x] support for @nix
* [ ] minimalist logging
  * [x] support for custom extensions like @cargo
  * [x] add 3 log levels: --level cargo|errors|verbose
    * [x] "cargo" (default): ignores logs from @nix and uses @cargo logs and error detection
    * [x] "errors" only use @nix logs, ignore @cargo messages for the failing build
    * [x] "verbose" shows all @nix messages for each build: successful and failed ones but still in sorted blocks (no mixed logging), ignores all @cargo messages
  * [x] remove --timing / --min-time / --debug
  * [x] print the @cargo internal logs properly & use the exit code
* [x] make it a crate libary

a good illustration on how to @nix protocol works and can be implemented. it sometimes shows different results than `nix-output-monitor` and `nix` itself:

* https://github.com/nixos/nix/issues/13909
* https://github.com/nixos/nix/issues/13910
* https://github.com/nixos/nix/issues/13935

# examples

as an example there are outputs in the tests folder one can experiment with:

the 'verbose' level:

    cat tests/example.stdin2 | cargo run -- --json --level verbose

the 'cargo' level:

    cat tests/example.stdin11 | cargo run -- --json --level cargo
    cat tests/example.stdin12 | cargo run -- --json --level cargo

the 'errors' level:

    cat tests/example.stdin2 | cargo run -- --json --level errors

## license

logone is primarily distributed under the terms of both the MIT license
and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
