# Lean buffer

Rust macro for structs to leverage flatbuffers without *.fbs files and without bloated generated code.

## Longer description
This is a macro library, for Rust, that generates extension traits that 
leverage [flatbuffers](https://google.github.io/flatbuffers/flatbuffers_guide_use_rust.html)
to flatten struct objects in `Vec<u8>`,
which can be used later to reinflate them again as struct objects.

All without the aid of `*.fbs` files and flatc.

This can be, in its turn, leveraged to facilitate inter-process / thread / channel communication.

## Requirements
To use this library, a `build.rs`, with an (empty) `fn main` is required,
in your crate project.

## Usage
See the `examples/usage.rs` for instructions.

## PRs are welcome
* Support for enum struct variants
* Support for enums in general
* Support for mixed primitive and composite struct values
* Support for merged, simple build.rs to remove extraneous 'use', then concat all the '*_gen.rs' files into one 'lean_buffer_gen.rs'; prolly 'sed' and 'cat' would suffice

## Interesting avenues of research
* [Transpile to WASM](https://github.com/google/flatbuffers/issues/4332) and pass messages between browsers, and [light-weight compute servers with Lunatic](https://github.com/lunatic-solutions/lunatic). [Mind the scalar constraint: i32, i64, f32, f64](https://webassembly.github.io/spec/core/syntax/types.html#number-types).
* [Pass messages between Erlang-like actors with ractor](https://github.com/slawlor/ractor).
* Translate eBPF bytecode instructions to rust struct objects, combined with pattern matching, and share with userspace.
* Somehow, by magic, merge this project with its [ObjectBox for rust progenitor](https://github.com/Buggaboo/objectbox-rust).
