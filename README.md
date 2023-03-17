# Lean buffer

Rust macro for structs to leverage flatbuffers serialization without *.fbs files and without bloated generated code.

There is no support for enums and complicated nesting. Keep it simple silly.

## Show me the code
```rust
use std::rc::Rc;

use flatbuffers::FlatBufferBuilder;
use lean_buffer::{
    macros::LeanBufferWrite,
    traits::{AdapterExt, Factory, FactoryExt},
};

#[derive(LeanBufferWrite)]
struct Entity {
    t_i64: i64,
}

enum EnumSupport {
  NO,
  YES { e: Entity }
}

type TupleSupport = (Entity, Entity);

include!(concat!(env!("OUT_DIR"), "/Entity_lb_gen.rs"));

fn main() {
    let mut builder = FlatBufferBuilder::new();

    let factory = Factory::<Entity> {
        phantom_data: std::marker::PhantomData,
    };
    let f = Rc::new(factory) as Rc<dyn FactoryExt<Entity>>;
    let mut e1 = f.new_object();

    e1.t_i64 = 0x1337833F;
    let e1_t_i64 = e1.t_i64;

    let a1 = Box::new(e1) as Box<dyn AdapterExt>;

    // flatten
    a1.flatten(&mut builder);
    let data = builder.finished_data();

    // inflate
    let first_offset: usize = data[0].into();

    unsafe {
        let mut table = flatbuffers::Table::new(data, first_offset);
        let resurrected_e1 = f.inflate(&mut table);

        if resurrected_e1.t_i64 == e1_t_i64 {
            println!("Hello world! {}", resurrected_e1.t_i64);
        } else {
            println!("Goodbye cruel world! {}", resurrected_e1.t_i64);
        }
    }
}
```

## Usage
Please see the [struct with scalar values](examples/usage.rs) and [struct with vector values](examples/usage_vecs.rs).

Make sure that your cargo project, contains a [`build.rs`](build.rs) file,
albeit an empty one.

## Longer description
This is a macro library, for Rust, that generates extension traits that 
leverage [flatbuffers](https://google.github.io/flatbuffers/flatbuffers_guide_use_rust.html)
to flatten struct objects in `Vec<u8>`,
which can be used later to reinflate them again as struct objects.

The required code is generated in one step.

All without the aid of `*.fbs` files and flatc.

This can be, in its turn, leveraged to facilitate inter-process / thread / channel communication.

## Requirements
To use this library, a `build.rs`, with an (empty) `fn main` is required,
in your crate project.

## PRs are welcome!
...
