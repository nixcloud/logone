# motivation

this rust project 'logone' implements a rust program and crate library to consume the nix output produced by:

    nix build --log-format internal-json

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

## message types

these messages are embedded in @nix messages and they have an id field already

### type 0

usage: to indicate that a crate build inside a mkDerivation has started of type `rustc` or `build-script-build`

    @cargo { 
        "type": 0, 
        "crate_name": "prettyplease",
    }

### type 1

usage: to send arbitraty text messages (unused)

@cargo { 
    "type": 1, 
    "crate_name": "prettyplease",
    "messages": [ "message 1", "message 2", ... ]
}

### type 2

usage: to indicate that a mkDerivation has finished compiling a crate of type `rustc` call
counterpart: finishes a type 0 message 

    @cargo { 
        "type": 2, 
        "crate_name": "prettyplease",
        "rustc_exit_code": 1, 
        "rustc_messages": [ 
            { "rendered": "cargo:rerun-if-changed=build.rs\\n 1    \\u001b[31mcargo:VERSION=0.2.37\\u001b[0m\\nError: \\\"Command: 'VERSION' on line: '4' not implemented yet!\\\"\\n\", 
              ...
            }
        ]
    }

### type 3

usage: to indicate a mkDerivation has finished compiling a crate
counterpart: finishes a type 0 message 

    @cargo {
        "type": 3, 
        "crate_name": "prettyplease",
        "exit_code": 0, 
        "messages": [ "message 1", "message 2", ... ]
    }


# usage

    nix build --log-format internal-json 2>&1 | logone --json --level cargo

or

    nix build --log-format internal-json 2> >(logone --json --level cargo)


# examples

as an example there are outputs in the tests folder one can experiment with:

the 'verbose' level:

    cat example/example.stdin2 | cargo run -- --json --level verbose

the 'cargo' level:

    cat example/example.stdin11 | cargo run -- --json --level cargo
    cat example/example.stdin12 | cargo run -- --json --level cargo

the 'errors' level:

    cat example/example.stdin2 | cargo run -- --json --level errors

## license

logone is primarily distributed under the terms of both the MIT license
and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.
