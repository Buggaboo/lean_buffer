use std::rc::Rc;

use flatbuffers::FlatBufferBuilder;
use lean_buffer::{
    macros::LeanBufferWrite,
    traits::{AdapterExt, Factory, FactoryExt},
};

// Don't panic when you see this false positive warning:
// proc macro `LeanBufferWrite` not expanded: proc macro not found in the built dylib
// Just check if the generated file can be located.
#[derive(LeanBufferWrite)]
struct EntityVecs {
    t_vec_u64: Vec<u64>,
    t_vec_i64: Vec<i64>,
    t_vec_u32: Vec<u32>,
    t_vec_i32: Vec<i32>,
    t_vec_char: Vec<char>,
    t_vec_u16: Vec<u16>,
    t_vec_i16: Vec<i16>,
    t_vec_u8: Vec<u8>,
    t_vec_i8: Vec<i8>,
    t_vec_bool: Vec<bool>,
    t_vec_string: Vec<String>,
    t_vec_double: Vec<f64>,
    t_vec_float: Vec<f32>,
}

// Either copy this file from your project, or use the name convention
// `<struct name>_lb_gen.rs` to include the generated file.
include!(concat!(env!("OUT_DIR"), "/EntityVecs_lb_gen.rs"));

fn main() {
    let mut builder = FlatBufferBuilder::new();

    let factory = Factory::<EntityVecs> {
        phantom_data: std::marker::PhantomData,
    };
    let f = Rc::new(factory) as Rc<dyn FactoryExt<EntityVecs>>;
    let mut e1 = f.new_object();

    let v = vec![0x8, 0x3, 0x3, 0xF];
    e1.t_vec_i64 = v;

    let a1 = Box::new(e1) as Box<dyn AdapterExt>;

    // flatten
    a1.to_fb(&mut builder);
    let data = Vec::from(builder.finished_data());

    // inflate
    let data_slice = data.as_slice();
    let first_offset: usize = data_slice[0].into();

    unsafe {
        let mut table = flatbuffers::Table::new(data_slice, first_offset);
        let resurrected_e1 = f.make(&mut table);

        if resurrected_e1.t_vec_i64 == vec![0x8, 0x3, 0x3, 0xF] {
            println!("Hello world!");
        } else {
            println!("Goodbye cruel world!");
        }
    }
}
