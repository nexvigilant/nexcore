//! AST query tools for structural Rust code search.
//!
//! Uses `academy_forge::extract::syn_parser` to parse Rust source files
//! and extract types, enums, traits, functions, and impl blocks.

use std::path::{Path, PathBuf};

use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::ast_query::{
    AstQueryFileParams, AstQueryImplementorsParams, AstQuerySearchParams,
};

/// Parse a single Rust source file and return all extracted items.
pub fn ast_query_file(params: AstQueryFileParams) -> Result<CallToolResult, McpError> {
    let path = PathBuf::from(&params.path);
    if !path.exists() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "File not found: {}",
            params.path
        ))]));
    }

    match academy_forge::extract::syn_parser::parse_file(&path) {
        Ok(parsed) => {
            let result = serde_json::json!({
                "success": true,
                "file": params.path,
                "module_doc": parsed.module_doc,
                "structs": parsed.types.iter().map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "doc": t.doc_comment,
                        "fields": t.fields.iter().map(|f| {
                            serde_json::json!({"name": f.name, "ty": f.ty})
                        }).collect::<Vec<_>>(),
                        "derives": t.derives,
                    })
                }).collect::<Vec<_>>(),
                "enums": parsed.enums.iter().map(|e| {
                    serde_json::json!({
                        "name": e.name,
                        "doc": e.doc_comment,
                        "variants": e.variants.iter().map(|v| v.name.clone()).collect::<Vec<_>>(),
                    })
                }).collect::<Vec<_>>(),
                "traits": parsed.traits.iter().map(|t| {
                    serde_json::json!({
                        "name": t.name,
                        "doc": t.doc_comment,
                        "methods": t.methods,
                    })
                }).collect::<Vec<_>>(),
                "functions": parsed.fns.iter().map(|f| {
                    serde_json::json!({
                        "name": f.name,
                        "doc": f.doc_comment,
                        "params": f.params.iter().map(|p| {
                            serde_json::json!({"name": p.name, "ty": p.ty})
                        }).collect::<Vec<_>>(),
                        "return_type": f.return_type,
                        "is_async": f.is_async,
                    })
                }).collect::<Vec<_>>(),
                "impls": parsed.impls.iter().map(|i| {
                    serde_json::json!({
                        "type_name": i.type_name,
                        "trait_name": i.trait_name,
                        "methods": i.methods,
                    })
                }).collect::<Vec<_>>(),
                "public_items": parsed.public_item_names,
            });
            Ok(CallToolResult::success(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
            )]))
        }
        Err(e) => Ok(CallToolResult::error(vec![Content::text(format!(
            "Parse error: {e}"
        ))])),
    }
}

/// Search a crate's source files for items matching a name pattern.
pub fn ast_query_search(params: AstQuerySearchParams) -> Result<CallToolResult, McpError> {
    let crate_dir = resolve_crate_dir(&params.crate_name);
    let src_dir = crate_dir.join("src");

    if !src_dir.exists() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Crate source not found: {}/src/",
            crate_dir.display()
        ))]));
    }

    let pattern_lower = params.pattern.to_lowercase();
    let item_types = params.item_types.as_ref();

    let mut matches: Vec<serde_json::Value> = Vec::new();
    let rs_files = collect_rs_files(&src_dir);

    for file_path in &rs_files {
        let parsed = match academy_forge::extract::syn_parser::parse_file(file_path) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let rel_path = file_path
            .strip_prefix(&crate_dir)
            .unwrap_or(file_path)
            .display()
            .to_string();

        if should_include("struct", item_types) {
            for t in &parsed.types {
                if t.name.to_lowercase().contains(&pattern_lower) {
                    matches.push(serde_json::json!({
                        "kind": "struct",
                        "name": t.name,
                        "file": rel_path,
                        "doc": t.doc_comment,
                    }));
                }
            }
        }

        if should_include("enum", item_types) {
            for e in &parsed.enums {
                if e.name.to_lowercase().contains(&pattern_lower) {
                    matches.push(serde_json::json!({
                        "kind": "enum",
                        "name": e.name,
                        "file": rel_path,
                        "doc": e.doc_comment,
                    }));
                }
            }
        }

        if should_include("trait", item_types) {
            for t in &parsed.traits {
                if t.name.to_lowercase().contains(&pattern_lower) {
                    matches.push(serde_json::json!({
                        "kind": "trait",
                        "name": t.name,
                        "file": rel_path,
                        "doc": t.doc_comment,
                        "methods": t.methods,
                    }));
                }
            }
        }

        if should_include("fn", item_types) {
            for f in &parsed.fns {
                if f.name.to_lowercase().contains(&pattern_lower) {
                    matches.push(serde_json::json!({
                        "kind": "fn",
                        "name": f.name,
                        "file": rel_path,
                        "doc": f.doc_comment,
                        "is_async": f.is_async,
                        "return_type": f.return_type,
                    }));
                }
            }
        }

        if should_include("impl", item_types) {
            for i in &parsed.impls {
                let type_match = i.type_name.to_lowercase().contains(&pattern_lower);
                let trait_match = i
                    .trait_name
                    .as_ref()
                    .is_some_and(|tn| tn.to_lowercase().contains(&pattern_lower));
                if type_match || trait_match {
                    matches.push(serde_json::json!({
                        "kind": "impl",
                        "type_name": i.type_name,
                        "trait_name": i.trait_name,
                        "file": rel_path,
                        "methods": i.methods,
                    }));
                }
            }
        }
    }

    let result = serde_json::json!({
        "success": true,
        "crate": params.crate_name,
        "pattern": params.pattern,
        "files_scanned": rs_files.len(),
        "matches": matches.len(),
        "items": matches,
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

/// Find all implementors of a trait within a crate.
pub fn ast_query_implementors(
    params: AstQueryImplementorsParams,
) -> Result<CallToolResult, McpError> {
    let crate_dir = resolve_crate_dir(&params.crate_name);
    let src_dir = crate_dir.join("src");

    if !src_dir.exists() {
        return Ok(CallToolResult::error(vec![Content::text(format!(
            "Crate source not found: {}/src/",
            crate_dir.display()
        ))]));
    }

    let trait_lower = params.trait_name.to_lowercase();
    let rs_files = collect_rs_files(&src_dir);
    let mut implementors: Vec<serde_json::Value> = Vec::new();

    for file_path in &rs_files {
        let parsed = match academy_forge::extract::syn_parser::parse_file(file_path) {
            Ok(p) => p,
            Err(_) => continue,
        };

        let rel_path = file_path
            .strip_prefix(&crate_dir)
            .unwrap_or(file_path)
            .display()
            .to_string();

        for imp in &parsed.impls {
            if let Some(ref tn) = imp.trait_name {
                if tn.to_lowercase().contains(&trait_lower) {
                    implementors.push(serde_json::json!({
                        "type_name": imp.type_name,
                        "trait_name": tn,
                        "methods": imp.methods,
                        "file": rel_path,
                    }));
                }
            }
        }
    }

    let result = serde_json::json!({
        "success": true,
        "crate": params.crate_name,
        "trait_name": params.trait_name,
        "files_scanned": rs_files.len(),
        "implementors_found": implementors.len(),
        "implementors": implementors,
    });
    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
    )]))
}

// ── Helpers ──────────────────────────────────────────────────────────────

fn resolve_crate_dir(crate_name: &str) -> PathBuf {
    let home = nexcore_fs::dirs::home_dir().unwrap_or_else(|| PathBuf::from("/home/matthew"));
    home.join("Projects/Active/nexcore/crates").join(crate_name)
}

fn collect_rs_files(dir: &Path) -> Vec<PathBuf> {
    let mut files = Vec::new();
    collect_rs_files_recursive(dir, &mut files);
    files
}

fn collect_rs_files_recursive(dir: &Path, files: &mut Vec<PathBuf>) {
    let entries = match std::fs::read_dir(dir) {
        Ok(e) => e,
        Err(_) => return,
    };
    for entry in entries.flatten() {
        let path = entry.path();
        if path.is_dir() {
            collect_rs_files_recursive(&path, files);
        } else if path.extension().is_some_and(|e| e == "rs") {
            files.push(path);
        }
    }
}

fn should_include(kind: &str, filter: Option<&Vec<String>>) -> bool {
    match filter {
        None => true,
        Some(types) => types.iter().any(|t| t.eq_ignore_ascii_case(kind)),
    }
}
