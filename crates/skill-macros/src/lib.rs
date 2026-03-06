#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

extern crate proc_macro;
use proc_macro::TokenStream;
use skill_macros_core::impl_skill_derive;
use syn::{DeriveInput, parse_macro_input};

/// Derive Skill trait with default implementations
#[proc_macro_derive(Skill)]
pub fn derive_skill(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    impl_skill_derive(&input).into()
}
