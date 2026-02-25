#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

extern crate proc_macro;
use proc_macro::TokenStream;
use stem_derive_core::impl_stem_newtype;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(StemNewtype, attributes(stem))]
pub fn derive_stem_newtype(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    match impl_stem_newtype(&input) {
        Ok(tokens) => tokens.into(),
        Err(e) => e.to_compile_error().into(),
    }
}
