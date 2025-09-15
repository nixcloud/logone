# motivation

this rust project 'logone' implements a rust program and crate library to consume the nix output produced by:

    `nix build --log-format internal-json`

and attempts to make nix logging less verbose / nosiy to be used in the nix backend of cargo.

# level

how it works, depends on the selected log level using `--level`.

## verbose

* when building several drv(s) in parallel, it accumulate the individual logs and outputs them in complete sequences

## errors

* it only prints the outputs of builds which have failed
* it prevents the print of the nix error summary (DRY)

## cargo

this mode imitates the `cargo build` style of logging i.e.:

* only log start of crate builds
* only log warnings / errors
* be very concise / precise, don't add the compiler call

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
