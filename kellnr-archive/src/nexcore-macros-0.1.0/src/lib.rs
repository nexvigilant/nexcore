#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

extern crate proc_macro;
use nexcore_macros_core::dummy_logic;
use proc_macro::TokenStream;

#[proc_macro]
pub fn nexcore_dummy(input: TokenStream) -> TokenStream {
    let _ = input;
    dummy_logic().into()
}
