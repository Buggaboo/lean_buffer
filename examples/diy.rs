use darling::FromDeriveInput;
use lean_buffer_internal::core::InputReceiver;

fn main() {
    let input = syn::parse_str(
        r#"
            #[derive(LeanBufferInternal)]
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
        "#,
    )
    .unwrap();
    let receiver = InputReceiver::from_derive_input(&input).unwrap();
    assert!(receiver.data.is_struct());
}
