# Lean buffer

Rust macro for structs to leverage flatbuffers without *.fbs files and without bloated generated code.

## Longer description
This is a macro library, for Rust, that generates extension traits that 
leverage [flatbuffers](https://google.github.io/flatbuffers/flatbuffers_guide_use_rust.html)
to flatten struct objects in `Vec<u8>`,
which can be used later to reinflate them again as struct objects.

All without the aid of `*.fbs` files.

This can be, in its turn, leveraged to facilitate inter-process / thread / channel communication.

## Requirements
To use this library, a `build.rs`, with an (empty) `fn main` is required,
in your crate project.

## Usage
See the `examples/usage.rs` for instructions.

## Interesting avenues of research
* Transpile to WASM and pass messages between browsers, and [light-weight compute servers with Lunatic](https://github.com/lunatic-solutions/lunatic).
* [Pass messages between Erlang-like actors with ractor](https://github.com/slawlor/ractor)
* Translate eBPF bytecode instructions to rust struct objects, combined with pattern matching, and share with userspace.