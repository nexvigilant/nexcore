#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use quote::quote;
use syn::DeriveInput;

pub fn impl_skill_derive(input: &DeriveInput) -> proc_macro2::TokenStream {
    let name = &input.ident;
    let name_str = name.to_string().to_lowercase().replace('_', "-");

    quote! {
        impl #name {
            /// Get skill name
            pub fn skill_name() -> &'static str {
                #name_str
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_skill_name_generation() {
        let input: DeriveInput = parse_quote! { struct My_Skill; };
        let res = impl_skill_derive(&input).to_string();
        assert!(res.contains("\"my-skill\""));
    }
}
