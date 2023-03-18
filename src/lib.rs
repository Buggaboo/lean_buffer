/// This diamond dependency is necessary because
/// the macro package does not allow any type exported
/// other than macros
pub extern crate internal;

pub mod traits;
pub extern crate lean_buffer_macros as macros;
