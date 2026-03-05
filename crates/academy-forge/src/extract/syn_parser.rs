//! `syn`-based Rust source parser.
//!
//! Extracts public types, enums, traits, constants, and doc comments
//! from Rust source files using AST parsing.

use std::path::Path;

use crate::error::ForgeError;
use crate::extract::doc_comments::{extract_doc_comment, extract_doc_summary};
use crate::ir::{
    ConstantInfo, EnumInfo, FieldInfo, FunctionInfo, FunctionParam, ImplInfo, TraitInfo, TypeInfo,
    VariantInfo,
};

/// Parsed items from a single Rust source file.
#[non_exhaustive]
#[derive(Debug, Default)]
pub struct ParsedFile {
    /// Module-level doc comment.
    pub module_doc: Option<String>,
    /// Public struct types.
    pub types: Vec<TypeInfo>,
    /// Public enum types.
    pub enums: Vec<EnumInfo>,
    /// Public constants.
    pub constants: Vec<ConstantInfo>,
    /// Public traits.
    pub traits: Vec<TraitInfo>,
    /// Public functions with full signatures.
    pub fns: Vec<FunctionInfo>,
    /// Impl blocks (both inherent and trait).
    pub impls: Vec<ImplInfo>,
    /// Names of all public items.
    pub public_item_names: Vec<String>,
}

/// Parse a Rust source file and extract public API items.
pub fn parse_file(path: &Path) -> Result<ParsedFile, ForgeError> {
    let content = std::fs::read_to_string(path).map_err(|e| ForgeError::IoError {
        path: path.to_path_buf(),
        source: e,
    })?;

    parse_source(&content, &path.display().to_string())
}

/// Parse Rust source code string and extract public API items.
pub fn parse_source(source: &str, file_name: &str) -> Result<ParsedFile, ForgeError> {
    let syntax: syn::File = syn::parse_file(source).map_err(|e| ForgeError::ParseError {
        file: file_name.to_string(),
        message: e.to_string(),
    })?;

    let mut result = ParsedFile {
        module_doc: extract_doc_comment(&syntax.attrs),
        ..Default::default()
    };

    for item in &syntax.items {
        extract_item(item, &mut result);
    }

    Ok(result)
}

fn extract_item(item: &syn::Item, result: &mut ParsedFile) {
    match item {
        syn::Item::Struct(s) if is_public(&s.vis) => {
            result.public_item_names.push(s.ident.to_string());
            result.types.push(extract_struct(s));
        }
        syn::Item::Enum(e) if is_public(&e.vis) => {
            result.public_item_names.push(e.ident.to_string());
            result.enums.push(extract_enum(e));
        }
        syn::Item::Const(c) if is_public(&c.vis) => {
            result.public_item_names.push(c.ident.to_string());
            result.constants.push(extract_const(c));
        }
        syn::Item::Trait(t) if is_public(&t.vis) => {
            result.public_item_names.push(t.ident.to_string());
            result.traits.push(extract_trait(t));
        }
        syn::Item::Fn(f) if is_public(&f.vis) => {
            result.public_item_names.push(f.sig.ident.to_string());
            result.fns.push(extract_fn(&f.sig, &f.attrs));
        }
        syn::Item::Impl(imp) => {
            result.impls.push(extract_impl(imp));
        }
        syn::Item::Mod(m) if is_public(&m.vis) => {
            result.public_item_names.push(m.ident.to_string());
            // Recurse into inline modules
            if let Some((_, items)) = &m.content {
                for sub_item in items {
                    extract_item(sub_item, result);
                }
            }
        }
        syn::Item::Const(_)
        | syn::Item::Enum(_)
        | syn::Item::ExternCrate(_)
        | syn::Item::Fn(_)
        | syn::Item::ForeignMod(_)
        | syn::Item::Macro(_)
        | syn::Item::Mod(_)
        | syn::Item::Static(_)
        | syn::Item::Struct(_)
        | syn::Item::Trait(_)
        | syn::Item::TraitAlias(_)
        | syn::Item::Type(_)
        | syn::Item::Union(_)
        | syn::Item::Use(_)
        | syn::Item::Verbatim(_)
        | _ => {}
    }
}

fn is_public(vis: &syn::Visibility) -> bool {
    matches!(vis, syn::Visibility::Public(_))
}

fn extract_struct(s: &syn::ItemStruct) -> TypeInfo {
    let fields = match &s.fields {
        syn::Fields::Named(named) => named
            .named
            .iter()
            .filter(|f| is_public(&f.vis))
            .map(|f| FieldInfo {
                name: f.ident.as_ref().map(|id| id.to_string()),
                ty: quote::ToTokens::to_token_stream(&f.ty).to_string(),
                doc_comment: extract_doc_summary(&f.attrs),
            })
            .collect(),
        syn::Fields::Unnamed(unnamed) => unnamed
            .unnamed
            .iter()
            .enumerate()
            .map(|(i, f)| FieldInfo {
                name: Some(format!("{i}")),
                ty: quote::ToTokens::to_token_stream(&f.ty).to_string(),
                doc_comment: extract_doc_summary(&f.attrs),
            })
            .collect(),
        syn::Fields::Unit => Vec::new(),
    };

    let derives = extract_derives(&s.attrs);

    TypeInfo {
        name: s.ident.to_string(),
        doc_comment: extract_doc_comment(&s.attrs),
        fields,
        derives,
    }
}

fn extract_enum(e: &syn::ItemEnum) -> EnumInfo {
    let variants = e
        .variants
        .iter()
        .map(|v| {
            let fields = match &v.fields {
                syn::Fields::Named(named) => named
                    .named
                    .iter()
                    .map(|f| FieldInfo {
                        name: f.ident.as_ref().map(|id| id.to_string()),
                        ty: quote::ToTokens::to_token_stream(&f.ty).to_string(),
                        doc_comment: extract_doc_summary(&f.attrs),
                    })
                    .collect(),
                syn::Fields::Unnamed(unnamed) => unnamed
                    .unnamed
                    .iter()
                    .enumerate()
                    .map(|(i, f)| FieldInfo {
                        name: Some(format!("{i}")),
                        ty: quote::ToTokens::to_token_stream(&f.ty).to_string(),
                        doc_comment: extract_doc_summary(&f.attrs),
                    })
                    .collect(),
                syn::Fields::Unit => Vec::new(),
            };

            VariantInfo {
                name: v.ident.to_string(),
                doc_comment: extract_doc_comment(&v.attrs),
                fields,
            }
        })
        .collect();

    EnumInfo {
        name: e.ident.to_string(),
        doc_comment: extract_doc_comment(&e.attrs),
        variants,
    }
}

fn extract_const(c: &syn::ItemConst) -> ConstantInfo {
    let value = Some(quote::ToTokens::to_token_stream(&c.expr).to_string());
    ConstantInfo {
        name: c.ident.to_string(),
        ty: quote::ToTokens::to_token_stream(&c.ty).to_string(),
        value,
        doc_comment: extract_doc_comment(&c.attrs),
    }
}

fn extract_trait(t: &syn::ItemTrait) -> TraitInfo {
    let methods = t
        .items
        .iter()
        .filter_map(|item| {
            if let syn::TraitItem::Fn(method) = item {
                Some(method.sig.ident.to_string())
            } else {
                None
            }
        })
        .collect();

    TraitInfo {
        name: t.ident.to_string(),
        doc_comment: extract_doc_comment(&t.attrs),
        methods,
    }
}

fn extract_fn(sig: &syn::Signature, attrs: &[syn::Attribute]) -> FunctionInfo {
    let params = sig
        .inputs
        .iter()
        .filter_map(|arg| {
            if let syn::FnArg::Typed(pat_type) = arg {
                let name = quote::ToTokens::to_token_stream(&pat_type.pat).to_string();
                let ty = quote::ToTokens::to_token_stream(&pat_type.ty).to_string();
                Some(FunctionParam { name, ty })
            } else {
                None // skip `self` parameters
            }
        })
        .collect();

    let return_type = match &sig.output {
        syn::ReturnType::Default => None,
        syn::ReturnType::Type(_, ty) => Some(quote::ToTokens::to_token_stream(ty).to_string()),
    };

    FunctionInfo {
        name: sig.ident.to_string(),
        doc_comment: extract_doc_comment(attrs),
        params,
        return_type,
        is_async: sig.asyncness.is_some(),
    }
}

fn extract_impl(imp: &syn::ItemImpl) -> ImplInfo {
    let type_name = quote::ToTokens::to_token_stream(&imp.self_ty).to_string();

    let trait_name = imp
        .trait_
        .as_ref()
        .map(|(_, path, _)| quote::ToTokens::to_token_stream(path).to_string());

    let methods = imp
        .items
        .iter()
        .filter_map(|item| {
            if let syn::ImplItem::Fn(method) = item {
                Some(method.sig.ident.to_string())
            } else {
                None
            }
        })
        .collect();

    ImplInfo {
        type_name,
        trait_name,
        methods,
    }
}

fn extract_derives(attrs: &[syn::Attribute]) -> Vec<String> {
    let mut derives = Vec::new();
    for attr in attrs {
        if attr.path().is_ident("derive") {
            if let Ok(nested) = attr.parse_args_with(
                syn::punctuated::Punctuated::<syn::Path, syn::Token![,]>::parse_terminated,
            ) {
                for path in nested {
                    if let Some(ident) = path.get_ident() {
                        derives.push(ident.to_string());
                    } else {
                        derives.push(
                            path.segments
                                .last()
                                .map(|s| s.ident.to_string())
                                .unwrap_or_default(),
                        );
                    }
                }
            }
        }
    }
    derives
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_simple_struct() {
        let source = r#"
            /// A test struct.
            #[derive(Debug, Clone)]
            pub struct Foo {
                /// The name.
                pub name: String,
                /// The value.
                pub value: u32,
            }
        "#;
        let parsed = parse_source(source, "test.rs").unwrap();
        assert_eq!(parsed.types.len(), 1);

        let ty = &parsed.types[0];
        assert_eq!(ty.name, "Foo");
        assert!(ty.doc_comment.as_deref().unwrap().contains("test struct"));
        assert_eq!(ty.fields.len(), 2);
        assert!(ty.derives.contains(&"Debug".to_string()));
        assert!(ty.derives.contains(&"Clone".to_string()));
    }

    #[test]
    fn test_parse_enum() {
        let source = r#"
            /// An enum.
            pub enum Color {
                /// Red.
                Red,
                /// Green.
                Green,
                /// Blue.
                Blue,
            }
        "#;
        let parsed = parse_source(source, "test.rs").unwrap();
        assert_eq!(parsed.enums.len(), 1);

        let en = &parsed.enums[0];
        assert_eq!(en.name, "Color");
        assert_eq!(en.variants.len(), 3);
        assert_eq!(en.variants[0].name, "Red");
    }

    #[test]
    fn test_parse_trait() {
        let source = r#"
            /// A trait.
            pub trait Greet {
                fn hello(&self) -> String;
                fn goodbye(&self);
            }
        "#;
        let parsed = parse_source(source, "test.rs").unwrap();
        assert_eq!(parsed.traits.len(), 1);
        assert_eq!(parsed.traits[0].methods.len(), 2);
    }

    #[test]
    fn test_parse_function() {
        let source = r#"
            /// Compute the score.
            pub async fn compute_score(data: &[f64], threshold: f64) -> Option<f64> {
                None
            }
        "#;
        let parsed = parse_source(source, "test.rs").unwrap();
        assert_eq!(parsed.fns.len(), 1);

        let f = &parsed.fns[0];
        assert_eq!(f.name, "compute_score");
        assert!(f.is_async);
        assert_eq!(f.params.len(), 2);
        assert_eq!(f.params[0].name, "data");
        assert!(f.params[1].name.contains("threshold"));
        assert!(f.return_type.as_deref().unwrap().contains("Option"));
        assert!(
            f.doc_comment
                .as_deref()
                .unwrap()
                .contains("Compute the score")
        );
    }

    #[test]
    fn test_parse_impl_block() {
        let source = r#"
            pub struct Foo;
            impl Foo {
                pub fn new() -> Self { Foo }
                fn private_method(&self) {}
            }
            impl std::fmt::Display for Foo {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    Ok(())
                }
            }
        "#;
        let parsed = parse_source(source, "test.rs").unwrap();
        assert_eq!(parsed.impls.len(), 2);

        let inherent = &parsed.impls[0];
        assert_eq!(inherent.type_name, "Foo");
        assert!(inherent.trait_name.is_none());
        assert_eq!(inherent.methods.len(), 2); // both new and private_method
        assert!(inherent.methods.contains(&"new".to_string()));

        let trait_impl = &parsed.impls[1];
        assert_eq!(trait_impl.type_name, "Foo");
        assert!(
            trait_impl
                .trait_name
                .as_deref()
                .unwrap()
                .contains("Display")
        );
        assert_eq!(trait_impl.methods.len(), 1);
    }

    #[test]
    fn test_skips_private_items() {
        let source = r#"
            struct Private;
            pub struct Public;
            fn private_fn() {}
            pub fn public_fn() {}
        "#;
        let parsed = parse_source(source, "test.rs").unwrap();
        assert_eq!(parsed.types.len(), 1);
        assert_eq!(parsed.types[0].name, "Public");
        assert_eq!(parsed.public_item_names.len(), 2); // Public + public_fn
    }
}
