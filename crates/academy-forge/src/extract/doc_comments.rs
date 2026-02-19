//! Extract doc comments from `syn` AST nodes.

/// Extract doc comment text from a slice of `syn::Attribute`.
///
/// Handles both `/// comment` (outer) and `#[doc = "comment"]` forms.
pub fn extract_doc_comment(attrs: &[syn::Attribute]) -> Option<String> {
    let mut lines = Vec::new();

    for attr in attrs {
        // Handle #[doc = "..."] attributes (which is what /// desugars to)
        if attr.path().is_ident("doc") {
            if let syn::Meta::NameValue(meta) = &attr.meta {
                if let syn::Expr::Lit(expr_lit) = &meta.value {
                    if let syn::Lit::Str(lit_str) = &expr_lit.lit {
                        lines.push(lit_str.value());
                    }
                }
            }
        }
    }

    if lines.is_empty() {
        return None;
    }

    // Clean up: trim leading space from each line (/// adds a space)
    let cleaned: Vec<String> = lines
        .iter()
        .map(|line| {
            if let Some(stripped) = line.strip_prefix(' ') {
                stripped.to_string()
            } else {
                line.clone()
            }
        })
        .collect();

    Some(cleaned.join("\n").trim().to_string())
}

/// Extract the first line of a doc comment (summary).
pub fn extract_doc_summary(attrs: &[syn::Attribute]) -> Option<String> {
    extract_doc_comment(attrs).map(|doc| {
        doc.lines()
            .next()
            .unwrap_or("")
            .trim_start_matches("# ")
            .trim()
            .to_string()
    })
}

#[cfg(test)]
#[allow(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
mod tests {
    use super::*;

    #[test]
    fn test_extract_doc_comment() {
        let code = r#"
            /// First line.
            /// Second line.
            pub struct Foo;
        "#;
        let file: syn::File = syn::parse_str(code).unwrap();
        if let syn::Item::Struct(s) = &file.items[0] {
            let doc = extract_doc_comment(&s.attrs);
            assert!(doc.is_some());
            let text = doc.unwrap();
            assert!(text.contains("First line."));
            assert!(text.contains("Second line."));
        }
    }

    #[test]
    fn test_no_doc_comment() {
        let code = r#"pub struct Foo;"#;
        let file: syn::File = syn::parse_str(code).unwrap();
        if let syn::Item::Struct(s) = &file.items[0] {
            assert!(extract_doc_comment(&s.attrs).is_none());
        }
    }
}
