/// This diamond dependency is necessary because
/// the macro package does not allow any type exported
/// other than macros
pub extern crate lean_buffer_internal as internal;
pub extern crate lean_buffer_macros as macros;

pub mod traits;
