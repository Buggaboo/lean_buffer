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
struct Entity {
    t_u64: u64,
    t_i64: i64,
    t_u32: u32,
    t_i32: i32,
    t_char: char,
    t_u16: u16,
    t_i16: i16,
    t_u8: u8,
    t_i8: i8,
    t_bool: bool,
    t_string: String,
    t_vec_string: Vec<String>,
    t_vec_u8: Vec<u8>,
    t_double: f64,
    t_float: f32,
}

/*
// TODO
#[derive(LeanBufferWrite)]
enum NestedStructs {
    Yes { t_bool: bool },
    No { t_u64: u64 },
}
*/

// Either copy this file from your project, or use the name convention
// `<struct name>_lb_gen.rs` to include the generated file.
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
    let data = Vec::from(builder.finished_data());

    // inflate
    let data_slice = data.as_slice();
    let first_offset: usize = data_slice[0].into();

    unsafe {
        let mut table = flatbuffers::Table::new(data_slice, first_offset);
        let resurrected_e1 = f.inflate(&mut table);

        if resurrected_e1.t_i64 == e1_t_i64 {
            println!("Hello world! {}", resurrected_e1.t_i64);
        } else {
            println!("Goodbye cruel world! {}", resurrected_e1.t_i64);
        }
    }
}
