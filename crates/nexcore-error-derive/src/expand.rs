//! Expansion logic for `#[derive(Error)]`.
//!
//! Generates `Display`, `std::error::Error`, and `From` implementations.

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use syn::{Data, DeriveInput, Fields, Index, Result};

use crate::attr::{ErrorAttr, has_from, has_source, parse_error_attr};

/// Main expansion entry point.
pub fn expand(input: &DeriveInput) -> Result<TokenStream> {
    match &input.data {
        Data::Enum(data) => expand_enum(input, data),
        Data::Struct(data) => expand_struct(input, data),
        Data::Union(_) => Err(syn::Error::new_spanned(
            input,
            "#[derive(Error)] does not support unions",
        )),
    }
}

/// Expand for enum types.
fn expand_enum(input: &DeriveInput, data: &syn::DataEnum) -> Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let mut display_arms = Vec::new();
    let mut source_arms = Vec::new();
    let mut from_impls = Vec::new();

    for variant in &data.variants {
        let var_ident = &variant.ident;
        let error_attr = parse_error_attr(&variant.attrs)?;

        let Some(ref attr) = error_attr else {
            return Err(syn::Error::new_spanned(
                variant,
                "missing #[error(\"...\")] attribute on variant",
            ));
        };

        match attr {
            ErrorAttr::Transparent(_) => {
                // #[error(transparent)] — forward Display and source to inner
                let (pattern, inner_expr) = match &variant.fields {
                    Fields::Unnamed(fields) if fields.unnamed.len() == 1 => {
                        (quote! { #name::#var_ident(__0) }, quote! { __0 })
                    }
                    _ => {
                        return Err(syn::Error::new_spanned(
                            variant,
                            "#[error(transparent)] requires exactly one unnamed field",
                        ));
                    }
                };

                display_arms.push(quote! {
                    #pattern => ::core::fmt::Display::fmt(#inner_expr, f),
                });
                source_arms.push(quote! {
                    #pattern => Some(#inner_expr),
                });

                // Also generate From impl for transparent
                let field = &variant.fields.iter().next();
                if let Some(field) = field {
                    if has_from(field) {
                        let ty = &field.ty;
                        from_impls.push(quote! {
                            impl #impl_generics ::core::convert::From<#ty> for #name #ty_generics #where_clause {
                                fn from(source: #ty) -> Self {
                                    #name::#var_ident(source)
                                }
                            }
                        });
                    }
                }
            }
            ErrorAttr::Fmt(fmt_str, _span) => {
                // Build pattern and format args
                let (pattern, format_call) =
                    build_display_arm(name, var_ident, &variant.fields, fmt_str)?;

                display_arms.push(quote! {
                    #pattern => #format_call,
                });

                // Source: find #[source] or #[from] field
                let source_expr = build_source_arm(name, var_ident, &variant.fields);
                source_arms.push(source_expr);

                // From impl for #[from] fields
                for (i, field) in variant.fields.iter().enumerate() {
                    if has_from(field) {
                        let ty = &field.ty;
                        let construction = match &variant.fields {
                            Fields::Unnamed(_) => {
                                let mut args: Vec<TokenStream> = Vec::new();
                                for (j, _f) in variant.fields.iter().enumerate() {
                                    if j == i {
                                        args.push(quote! { source });
                                    } else {
                                        args.push(quote! { ::core::default::Default::default() });
                                    }
                                }
                                quote! { #name::#var_ident(#(#args),*) }
                            }
                            Fields::Named(_) => {
                                let field_ident = field.ident.as_ref();
                                let mut fields_init: Vec<TokenStream> = Vec::new();
                                for f in variant.fields.iter() {
                                    let fi = f.ident.as_ref();
                                    if fi == field_ident {
                                        fields_init.push(quote! { #fi: source });
                                    } else {
                                        fields_init.push(
                                            quote! { #fi: ::core::default::Default::default() },
                                        );
                                    }
                                }
                                quote! { #name::#var_ident { #(#fields_init),* } }
                            }
                            Fields::Unit => unreachable!(),
                        };

                        from_impls.push(quote! {
                            impl #impl_generics ::core::convert::From<#ty> for #name #ty_generics #where_clause {
                                fn from(source: #ty) -> Self {
                                    #construction
                                }
                            }
                        });
                    }
                }
            }
        }
    }

    Ok(quote! {
        impl #impl_generics ::core::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                match self {
                    #(#display_arms)*
                }
            }
        }

        impl #impl_generics ::std::error::Error for #name #ty_generics #where_clause {
            fn source(&self) -> ::core::option::Option<&(dyn ::std::error::Error + 'static)> {
                #[allow(unused_variables)]
                match self {
                    #(#source_arms)*
                }
            }
        }

        #(#from_impls)*
    })
}

/// Expand for struct types.
fn expand_struct(input: &DeriveInput, data: &syn::DataStruct) -> Result<TokenStream> {
    let name = &input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();

    let error_attr = parse_error_attr(&input.attrs)?;

    let display_body = if let Some(attr) = &error_attr {
        match attr {
            ErrorAttr::Transparent(_) => {
                let field = data.fields.iter().next().ok_or_else(|| {
                    syn::Error::new_spanned(input, "#[error(transparent)] requires a field")
                })?;
                let accessor = match &field.ident {
                    Some(id) => quote! { &self.#id },
                    None => quote! { &self.0 },
                };
                quote! { ::core::fmt::Display::fmt(#accessor, f) }
            }
            ErrorAttr::Fmt(fmt_str, _) => build_struct_format_call(&data.fields, fmt_str)?,
        }
    } else {
        // No #[error] attribute — use type name
        let s = name.to_string();
        quote! { f.write_str(#s) }
    };

    // Find source field
    let source_body = build_struct_source(&data.fields);

    // From impls
    let mut from_impls = Vec::new();
    for field in data.fields.iter() {
        if has_from(field) {
            let ty = &field.ty;
            let construction = match &field.ident {
                Some(id) => quote! { #name { #id: source, ..::core::default::Default::default() } },
                None => quote! { #name(source) },
            };
            from_impls.push(quote! {
                impl #impl_generics ::core::convert::From<#ty> for #name #ty_generics #where_clause {
                    fn from(source: #ty) -> Self {
                        #construction
                    }
                }
            });
        }
    }

    Ok(quote! {
        impl #impl_generics ::core::fmt::Display for #name #ty_generics #where_clause {
            fn fmt(&self, f: &mut ::core::fmt::Formatter<'_>) -> ::core::fmt::Result {
                #display_body
            }
        }

        impl #impl_generics ::std::error::Error for #name #ty_generics #where_clause {
            fn source(&self) -> ::core::option::Option<&(dyn ::std::error::Error + 'static)> {
                #source_body
            }
        }

        #(#from_impls)*
    })
}

/// Build a Display match arm for an enum variant.
fn build_display_arm(
    enum_name: &syn::Ident,
    var_ident: &syn::Ident,
    fields: &Fields,
    fmt_str: &str,
) -> Result<(TokenStream, TokenStream)> {
    match fields {
        Fields::Unit => {
            let pattern = quote! { #enum_name::#var_ident };
            let call = quote! { f.write_str(#fmt_str) };
            Ok((pattern, call))
        }
        Fields::Unnamed(unnamed) => {
            let bindings: Vec<_> = (0..unnamed.unnamed.len())
                .map(|i| format_ident!("__field{}", i))
                .collect();
            let pattern = quote! { #enum_name::#var_ident(#(#bindings),*) };
            let call = build_write_call(fmt_str, &bindings, &[], fields);
            Ok((pattern, call))
        }
        Fields::Named(named) => {
            let field_idents: Vec<_> = named
                .named
                .iter()
                .map(|f| {
                    f.ident
                        .as_ref()
                        .map_or_else(|| format_ident!("_"), |id| id.clone())
                })
                .collect();
            let pattern = quote! { #enum_name::#var_ident { #(#field_idents),* } };
            let call = build_write_call(fmt_str, &[], &field_idents, fields);
            Ok((pattern, call))
        }
    }
}

/// Extract field names referenced in a format string.
/// Parses `{name}`, `{name:?}`, `{name:<spec>}` etc.
/// Returns the set of referenced field name strings.
fn extract_format_references(fmt_str: &str) -> std::collections::HashSet<String> {
    let mut refs = std::collections::HashSet::new();
    let bytes = fmt_str.as_bytes();
    let len = bytes.len();
    let mut i = 0;

    while i < len {
        if bytes[i] == b'{' {
            i += 1;
            // Skip escaped `{{`
            if i < len && bytes[i] == b'{' {
                i += 1;
                continue;
            }
            // Read the identifier until `:`, `}`, or end
            let start = i;
            while i < len && bytes[i] != b':' && bytes[i] != b'}' {
                i += 1;
            }
            let name = &fmt_str[start..i];
            if !name.is_empty() {
                refs.insert(name.to_string());
            }
            // Skip to closing `}`
            while i < len && bytes[i] != b'}' {
                i += 1;
            }
        }
        i += 1;
    }

    refs
}

/// Build a `write!(f, ...)` call from a format string and field bindings.
/// Only passes fields actually referenced in the format string.
fn build_write_call(
    fmt_str: &str,
    positional: &[syn::Ident],
    named: &[syn::Ident],
    _fields: &Fields,
) -> TokenStream {
    let refs = extract_format_references(fmt_str);

    if !positional.is_empty() {
        // Unnamed fields: {0}, {1}, etc. — filter to referenced indices
        let used: Vec<_> = positional
            .iter()
            .enumerate()
            .filter(|(i, _)| refs.contains(&i.to_string()))
            .map(|(_, ident)| ident)
            .collect();
        if used.is_empty() {
            quote! { f.write_str(#fmt_str) }
        } else {
            quote! { write!(f, #fmt_str, #(#used),*) }
        }
    } else if !named.is_empty() {
        // Named fields: only pass those referenced in format string
        let used: Vec<_> = named
            .iter()
            .filter(|ident| refs.contains(&ident.to_string()))
            .collect();
        if used.is_empty() {
            quote! { f.write_str(#fmt_str) }
        } else {
            quote! { write!(f, #fmt_str, #(#used = #used),*) }
        }
    } else {
        quote! { f.write_str(#fmt_str) }
    }
}

/// Build a source() match arm for an enum variant.
fn build_source_arm(
    enum_name: &syn::Ident,
    var_ident: &syn::Ident,
    fields: &Fields,
) -> TokenStream {
    // Look for #[source] or #[from] field
    for (i, field) in fields.iter().enumerate() {
        if has_source(field) || has_from(field) {
            match fields {
                Fields::Named(_) => {
                    let id = field.ident.as_ref();
                    let field_idents: Vec<_> = fields
                        .iter()
                        .map(|f| {
                            f.ident
                                .as_ref()
                                .map_or_else(|| format_ident!("_"), |i| i.clone())
                        })
                        .collect();
                    return quote! {
                        #enum_name::#var_ident { #(#field_idents),* } =>
                            ::core::option::Option::Some(#id),
                    };
                }
                Fields::Unnamed(_) => {
                    let bindings: Vec<_> = (0..fields.len())
                        .map(|j| {
                            if j == i {
                                format_ident!("__source")
                            } else {
                                format_ident!("_")
                            }
                        })
                        .collect();
                    return quote! {
                        #enum_name::#var_ident(#(#bindings),*) =>
                            ::core::option::Option::Some(__source),
                    };
                }
                Fields::Unit => {}
            }
        }
    }

    // No source
    match fields {
        Fields::Unit => {
            quote! { #enum_name::#var_ident => ::core::option::Option::None, }
        }
        Fields::Unnamed(u) => {
            let wilds: Vec<_> = (0..u.unnamed.len()).map(|_| quote! { _ }).collect();
            quote! { #enum_name::#var_ident(#(#wilds),*) => ::core::option::Option::None, }
        }
        Fields::Named(n) => {
            let wilds: Vec<_> = n.named.iter().map(|_| quote! { _ }).collect();
            let names: Vec<_> = n.named.iter().map(|f| &f.ident).collect();
            quote! { #enum_name::#var_ident { #(#names: #wilds),* } => ::core::option::Option::None, }
        }
    }
}

/// Build format call for struct Display.
/// Only passes fields actually referenced in the format string.
fn build_struct_format_call(fields: &Fields, fmt_str: &str) -> Result<TokenStream> {
    let refs = extract_format_references(fmt_str);

    match fields {
        Fields::Named(named) => {
            let used: Vec<_> = named
                .named
                .iter()
                .filter_map(|f| f.ident.as_ref())
                .filter(|ident| refs.contains(&ident.to_string()))
                .collect();
            if used.is_empty() {
                Ok(quote! { f.write_str(#fmt_str) })
            } else {
                Ok(quote! { write!(f, #fmt_str, #(#used = &self.#used),*) })
            }
        }
        Fields::Unnamed(unnamed) => {
            let used: Vec<_> = (0..unnamed.unnamed.len())
                .filter(|i| refs.contains(&i.to_string()))
                .map(Index::from)
                .collect();
            if used.is_empty() {
                Ok(quote! { f.write_str(#fmt_str) })
            } else {
                Ok(quote! { write!(f, #fmt_str, #(&self.#used),*) })
            }
        }
        Fields::Unit => Ok(quote! { f.write_str(#fmt_str) }),
    }
}

/// Build source() for struct.
fn build_struct_source(fields: &Fields) -> TokenStream {
    for field in fields.iter() {
        if has_source(field) || has_from(field) {
            let accessor = match &field.ident {
                Some(id) => quote! { &self.#id },
                None => quote! { &self.0 },
            };
            return quote! {
                ::core::option::Option::Some(#accessor)
            };
        }
    }
    quote! { ::core::option::Option::None }
}
