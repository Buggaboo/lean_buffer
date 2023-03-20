use std::rc::Rc;

use flatbuffers::FlatBufferBuilder;
use lean_buffer::{
    macros::LeanBufferWrite,
    macros::LeanBufferRaw,
    traits::{AdapterExt, Factory, FactoryExt},
};

// Don't panic when you see this false positive warning:
// proc macro `LeanBufferWrite` not expanded: proc macro not found in the built dylib
// Just check if the generated file can be located.
#[derive(LeanBufferWrite)]
struct EntityOptions {
    t_opt_u64: Option<u64>,
    t_opt_i64: Option<i64>,
    t_opt_u32: Option<u32>,
    t_opt_i32: Option<i32>,
    t_opt_char: Option<char>,
    t_opt_u16: Option<u16>,
    t_opt_i16: Option<i16>,
    t_opt_u8: Option<u8>,
    t_opt_i8: Option<i8>,
    t_opt_bool: Option<bool>,
    t_opt_string: Option<String>,
    t_opt_double: Option<f64>,
    t_opt_float: Option<f32>,
}

#[derive(LeanBufferRaw)]
struct EntityOptionsRaw {
    t_opt_u64: Option<u64>,
}

// Either copy this file from your project, or use the name convention
// `<struct name>_lb_gen.rs` to include the generated file.
include!(concat!(env!("OUT_DIR"), "/EntityOptions_lb_gen.rs"));

fn main() {
    let mut builder = FlatBufferBuilder::new();

    let factory = Factory::<EntityOptions> {
        phantom_data: std::marker::PhantomData,
    };
    let f = Rc::new(factory) as Rc<dyn FactoryExt<EntityOptions>>;
    let mut e1 = f.new_object();

    let v = Some(64);
    e1.t_opt_i64 = v;

    let a1 = Box::new(e1) as Box<dyn AdapterExt>;

    // flatten
    a1.flatten(&mut builder);
    let data = builder.finished_data();

    // inflate
    let first_offset: usize = data[0].into();

    unsafe {
        let mut table = flatbuffers::Table::new(data, first_offset);
        let resurrected_e1 = f.inflate(&mut table);

        if resurrected_e1.t_opt_i64 == Some(64) {
            println!("Hello world!");
        } else {
            println!("Goodbye cruel world!");
        }
    }
}
