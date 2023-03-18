use std::rc::Rc;

use flatbuffers::FlatBufferBuilder;
use lean_buffer::traits::{AdapterExt, Factory, FactoryExt};


struct EntityMixed {
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
  t_double: f64,
  t_float: f32,
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
  t_double: f64,
  t_float: f32
}

// See `build.rs`, might require multiple `cargo build` invocations
// also, to generate `*_lb_gen.rs`, each program in examples must be run at least once
include!(concat!(env!("OUT_DIR"), "/merged_gen.lb.rs"));

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
