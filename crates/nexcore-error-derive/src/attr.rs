//! Attribute parsing for `#[error("...")]`, `#[from]`, `#[source]`.

use proc_macro2::Span;
use syn::spanned::Spanned;
use syn::{Attribute, Expr, Field, Lit, Meta, Result};

/// Parsed `#[error("...")]` attribute.
#[derive(Clone)]
pub enum ErrorAttr {
    /// `#[error("format string")]`
    Fmt(String, Span),
    /// `#[error(transparent)]`
    Transparent(Span),
}

/// Parse `#[error(...)]` from a list of attributes.
pub fn parse_error_attr(attrs: &[Attribute]) -> Result<Option<ErrorAttr>> {
    for attr in attrs {
        if !attr.path().is_ident("error") {
            continue;
        }
        let meta = &attr.meta;
        match meta {
            Meta::List(list) => {
                let tokens = list.tokens.clone();
                let token_str = tokens.to_string();

                // #[error(transparent)]
                if token_str.trim() == "transparent" {
                    return Ok(Some(ErrorAttr::Transparent(attr.span())));
                }

                // #[error("format string")]
                let expr: Expr = syn::parse2(tokens)?;
                if let Expr::Lit(lit) = &expr {
                    if let Lit::Str(s) = &lit.lit {
                        return Ok(Some(ErrorAttr::Fmt(s.value(), s.span())));
                    }
                }

                return Err(syn::Error::new_spanned(
                    attr,
                    "expected #[error(\"...\")]  or #[error(transparent)]",
                ));
            }
            // Meta::Path and Meta::NameValue are never valid for #[error]; both
            // are caught here so the exhaustive match is explicit rather than using
            // a wildcard that would silently absorb future syn variants.
            Meta::Path(_) | Meta::NameValue(_) => {
                return Err(syn::Error::new_spanned(attr, "expected #[error(\"...\")]"));
            }
        }
    }
    Ok(None)
}

/// Check if a field has `#[from]` attribute.
pub fn has_from(field: &Field) -> bool {
    field.attrs.iter().any(|a| a.path().is_ident("from"))
}

/// Check if a field has `#[source]` attribute.
pub fn has_source(field: &Field) -> bool {
    field.attrs.iter().any(|a| a.path().is_ident("source"))
}
