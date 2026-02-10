# motivation

logone was created to be used as a crate (library) in cargo (libnix) on https://github.com/nixcloud/cargo

this rust project 'logone' implements a 'standalone program' and a 'crate library' to consume `nix build` output like:

    nix build --log-format internal-json

and it changes the way logging works by making it less verbose / nosiy.

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

## cargo specific message types

logone bridges two worlds by refining the wild nix logs in a way cargo legacy would display them.

cargo specific messages are embedded in @nix messages and their `id` field is inherited from the upper layer.

motivation: ideally we have 3 message types:

* session start (type 0)
* arbitrary message inside session (type 1)
* session end (type 2) with exit code and optional final messages

messages with an `id`, while there is no active session, will be discarded silently.

note: type 2 and 3 messages are currently used because i did not want to handle the json processing in bash scripts. type 1 messages are currently not used but reserved.

### type 0

motivation: sent when a crate build, usually sent from a `mkDerivation`, has started. usually emitted by types (lib), (bin) or (build.rs *) builds.

example:

    @cargo { 
        "type": 0, 
        "crate_name": "prettyplease",
        "crate_type": "(lib)",
    }

### type 1

motivation: to send arbitraty text messages (unused)

example:

    @cargo { 
        "type": 1, 
        "crate_name": "prettyplease",
        "crate_type": "(lib)",
        "messages": [ "message 1", "message 2", ... ]
    }

note: this message should probably removed in a refactor.

### type 2

motivation: as reaction to a 'type 0 message', sent from a crate build type (lib), (bin), (build.rs build). this message frees the accumulated messages stored in the buffer.

    @cargo { 
        "type": 2, 
        "crate_name": "prettyplease",
        "crate_type": "(lib)",
        "rustc_exit_code": 1, 
        "rustc_messages": [ 
            { "rendered": "cargo:rerun-if-changed=build.rs\\n 1    \\u001b[31mcargo:VERSION=0.2.37\\u001b[0m\\nError: \\\"Command: 'VERSION' on line: '4' not implemented yet!\\\"\\n\", 
              ...
            }
        ]
    }

### type 3

motivation: also as a reaction to a 'type 0 message', but sent from a (build.rs run). this type was created because `rustc` emits different messages than `build_script_build` executions.

usage: to indicate a mkDerivation has finished compiling a crate
counterpart: finishes a type 0 message 

    @cargo {
        "type": 3, 
        "crate_name": "prettyplease",
        "crate_type": "(lib)",
        "exit_code": 0, 
        "messages": [ "message 1", "message 2", ... ]
    }

note: not sure but maybe 'type 2' and 'type 3' should be refactored into one message type but this has to be done on the sender side also, which is the generated nix toolchain.

# usage

    nix build --log-format internal-json 2>&1 | logone --json --level cargo

or

    nix build --log-format internal-json 2> >(logone --json --level cargo)

# examples

as an example there are outputs in the tests folder one can experiment with:

the 'cargo' level (https://asciinema.org/a/784901):

    cat examples/example.stdin11 | cargo run -- --json --level cargo

the 'cargo' level (https://asciinema.org/a/784927):

    cat examples/example.stdin12 | cargo run -- --json --level cargo

the 'verbose' level (https://asciinema.org/a/784929):

    cat examples/example.stdin2 | cargo run -- --json --level verbose

the 'errors' level (https://asciinema.org/a/784947):

    cat examples/example.stdin2 | cargo run -- --json --level errors

## license

logone is primarily distributed under the terms of both the MIT license
and the Apache License (Version 2.0).

See [LICENSE-APACHE](LICENSE-APACHE) and [LICENSE-MIT](LICENSE-MIT) for details.

this is the same license as cargo uses.
