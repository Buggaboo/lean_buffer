#![allow(dead_code)]

extern crate lean_buffer_internal as internal;

use std::str::FromStr;

use darling::FromDeriveInput;
use proc_macro::TokenStream;
use syn::DeriveInput;

use internal::core::InputReceiver;

#[proc_macro_derive(LeanBufferWrite)]
pub fn derive_fb_code_then_write(input: TokenStream) -> TokenStream {
    let mut out = TokenStream::new();
    // yes, nasty hack, to wrap code generation
    out.extend(TokenStream::from_str("#[derive(LeanBufferInternal)]"));
    out.extend(input.clone());
    let parsed = syn::parse::<DeriveInput>(out).expect("crash");
    let mut receiver = InputReceiver::from_derive_input(&parsed).expect("crash");
    receiver.write_to_out_dir(None, None, None);
    TokenStream::new()
}

/// Not-prettified output, will be collated later, then hopefully prettified
#[proc_macro_derive(LeanBufferRaw)]
pub fn derive_fb_code_then_write_raw(input: TokenStream) -> TokenStream {
    let mut out = TokenStream::new();
    // yes, nasty hack, to wrap code generation
    out.extend(TokenStream::from_str("#[derive(LeanBufferInternal)]"));
    out.extend(input.clone());
    let parsed = syn::parse::<DeriveInput>(out).expect("crash");
    let mut receiver = InputReceiver::from_derive_input(&parsed).expect("crash");
    receiver.write_raw_to_out_dir(None, None, None);
    TokenStream::new()
}
