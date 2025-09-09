# motivation

this rust project implements a rust client to the `nix build --log-format internal-json` protocol.

it is also an attempt to make nix logging less verbose and more readable by humans by doing this:

* when building several drv(s) it accumulate the individual logs and only outputs them when there is an error
* it prevents the print of the nix error summary where you have the error twice in the output but with the hideous indentation

# status

* [x] prototype as standalone binary works
  * [x] support for @nix
  * [ ] support for custom extensions like @cargo
* [ ] available as crate libary

it is currently a good illustration on how to @nix protocol works and can be implemented. it sometimes has different results as nix-output-monitor and even nix as the current json format has some issues:

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

this was replit generated, use the code for whatever you need it. 

if in doubt just use the same license(s) nix is distributed under.