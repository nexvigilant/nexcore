#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

use quote::quote;

pub fn dummy_logic() -> proc_macro2::TokenStream {
    quote! {
        pub fn hello_from_nexcore() {
            println!("Hello from NexCore!");
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_dummy_logic() {
        let res = dummy_logic().to_string();
        assert!(res.contains("hello_from_nexcore"));
    }
}
