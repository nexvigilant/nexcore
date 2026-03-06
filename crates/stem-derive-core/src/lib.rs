#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

use quote::quote;
use syn::DeriveInput;

pub fn impl_stem_newtype(input: &DeriveInput) -> syn::Result<proc_macro2::TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    match &input.data {
        syn::Data::Struct(s) => match &s.fields {
            syn::Fields::Unnamed(f) if f.unnamed.len() == 1 => {}
            syn::Fields::Unnamed(_) | syn::Fields::Named(_) | syn::Fields::Unit => {
                return Err(syn::Error::new_spanned(
                    name,
                    "requires tuple struct with one field",
                ))
            }
        },
        syn::Data::Enum(_) | syn::Data::Union(_) => {
            return Err(syn::Error::new_spanned(
                name,
                "can only be applied to tuple structs",
            ))
        }
    }

    let mut clamp_min: Option<f64> = None;
    let mut clamp_max: Option<f64> = None;
    let mut default_value: f64 = 0.0;

    for attr in &input.attrs {
        if !attr.path().is_ident("stem") {
            continue;
        }
        attr.parse_nested_meta(|meta| {
            if meta.path.is_ident("clamp_min") {
                clamp_min = Some(
                    meta.value()?
                        .parse::<syn::LitFloat>()?
                        .base10_parse::<f64>()?,
                );
                Ok(())
            } else if meta.path.is_ident("clamp_max") {
                clamp_max = Some(
                    meta.value()?
                        .parse::<syn::LitFloat>()?
                        .base10_parse::<f64>()?,
                );
                Ok(())
            } else if meta.path.is_ident("default_value") {
                default_value = meta
                    .value()?
                    .parse::<syn::LitFloat>()?
                    .base10_parse::<f64>()?;
                Ok(())
            } else {
                Err(meta.error("unknown stem attribute"))
            }
        })?;
    }

    let clamp_expr = match (clamp_min, clamp_max) {
        (Some(min), Some(max)) => quote! { value.clamp(#min, #max) },
        (Some(min), None) => quote! { value.max(#min) },
        (None, Some(max)) => quote! { value.min(#max) },
        (None, None) => quote! { value },
    };

    Ok(quote! {
        impl #impl_generics #name #ty_generics #where_clause {
            pub fn new(value: f64) -> Self { Self(#clamp_expr) }
            pub fn value(&self) -> f64 { self.0 }
        }
        impl #impl_generics Default for #name #ty_generics #where_clause {
            fn default() -> Self { Self(#default_value) }
        }
        impl #impl_generics From<f64> for #name #ty_generics #where_clause {
            fn from(value: f64) -> Self { Self::new(value) }
        }
        impl #impl_generics From<#name #ty_generics> for f64 #where_clause {
            fn from(val: #name #ty_generics) -> f64 { val.0 }
        }
    })
}

#[cfg(test)]
mod tests {
    use super::*;
    use syn::parse_quote;

    #[test]
    fn test_valid_struct() {
        let input: DeriveInput = parse_quote! { struct Prob(f64); };
        let res = impl_stem_newtype(&input);
        assert!(res.is_ok());
    }

    #[test]
    fn test_clamping_attributes() {
        let input: DeriveInput = parse_quote! {
            #[stem(clamp_min = 0.0, clamp_max = 1.0)]
            struct Prob(f64);
        };
        let res = impl_stem_newtype(&input).unwrap().to_string(); // INVARIANT: test
        let normalized: String = res.chars().filter(|c| !c.is_whitespace()).collect();
        assert!(normalized.contains("clamp("), "Expected clamp in: {}", res);
    }
}
