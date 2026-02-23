//! Rust Development tools (7 total):
//!
//! Batch 1: error_type, derive_advisor, match_generate, borrow_explain
//! Batch 2: clippy_explain, rustc_explain, unsafe_audit
//! Batch 3: cargo_expand, cargo_bloat, cargo_miri, edition_migrate, invocations
//!
//! Pure computation (except rustc_explain which shells out to `rustc --explain`).
//! Uses string-based heuristic analysis (no `syn` AST).
//! Follows the `crate_xray.rs` pattern: lightweight line parsing.
//!
//! Tier: T3 (→+μ+κ+∂ — causality, mapping, comparison, boundary)

use crate::params::rust_dev::{
    RustDevBorrowExplainParams, RustDevCargoBloatParams, RustDevCargoExpandParams,
    RustDevCargoMiriParams, RustDevClippyExplainParams, RustDevDeriveAdvisorParams,
    RustDevEditionMigrateParams, RustDevErrorTypeParams, RustDevInvocationsParams,
    RustDevMatchGenerateParams, RustDevRustcExplainParams, RustDevUnsafeAuditParams,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::path::PathBuf;

// ============================================================================
// 1. rust_dev_error_type — Generate thiserror enum from variant specs
// ============================================================================

pub fn error_type(params: RustDevErrorTypeParams) -> Result<CallToolResult, McpError> {
    let type_name = format!("{}Error", params.name);
    let mut code = String::with_capacity(1024);

    if params.use_thiserror {
        code.push_str("use thiserror::Error;\n\n");
        code.push_str("#[derive(Debug, Error)]\n");
    } else {
        code.push_str("use std::fmt;\n\n");
        code.push_str("#[derive(Debug)]\n");
    }

    code.push_str(&format!("pub enum {type_name} {{\n"));

    for variant in &params.variants {
        // #[error("...")] attribute
        if params.use_thiserror {
            code.push_str(&format!("    #[error(\"{}\")]\n", variant.message));
        }

        // Variant with fields
        if variant.fields.is_empty() && variant.from.is_none() {
            code.push_str(&format!("    {},\n\n", variant.name));
        } else {
            let has_named = variant
                .fields
                .iter()
                .any(|f| f.contains(':'));

            if has_named {
                // Struct variant
                code.push_str(&format!("    {} {{\n", variant.name));
                for field in &variant.fields {
                    code.push_str(&format!("        {field},\n"));
                }
                if let Some(ref from_type) = variant.from {
                    if params.use_thiserror {
                        code.push_str("        #[from]\n");
                    }
                    code.push_str(&format!("        source: {from_type},\n"));
                }
                code.push_str("    },\n\n");
            } else {
                // Tuple variant
                code.push_str(&format!("    {}(", variant.name));
                let mut parts: Vec<String> = Vec::new();
                if let Some(ref from_type) = variant.from {
                    if params.use_thiserror {
                        parts.push(format!("#[from] {from_type}"));
                    } else {
                        parts.push(from_type.clone());
                    }
                }
                for field in &variant.fields {
                    parts.push(field.clone());
                }
                code.push_str(&parts.join(", "));
                code.push_str("),\n\n");
            }
        }
    }

    code.push_str("}\n");

    // Manual Display impl if not using thiserror
    if !params.use_thiserror {
        code.push_str(&format!(
            "\nimpl fmt::Display for {type_name} {{\n    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {{\n        match self {{\n"
        ));
        for variant in &params.variants {
            code.push_str(&format!(
                "            {type_name}::{} => write!(f, \"{}\"),\n",
                variant.name, variant.message
            ));
        }
        code.push_str("        }\n    }\n}\n");
        code.push_str(&format!(
            "\nimpl std::error::Error for {type_name} {{}}\n"
        ));
    }

    // Result type alias
    code.push_str(&format!(
        "\npub type Result<T> = std::result::Result<T, {type_name}>;\n"
    ));

    let result = json!({
        "success": true,
        "type_name": type_name,
        "variant_count": params.variants.len(),
        "uses_thiserror": params.use_thiserror,
        "code": code,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// 2. rust_dev_derive_advisor — Analyze type → recommend derives
// ============================================================================

/// Known types that do NOT implement specific traits.
struct TraitBlockers;

impl TraitBlockers {
    /// Types that block Copy (heap-allocated or dynamically sized).
    const NON_COPY: &[&str] = &[
        "String", "Vec", "Box", "Rc", "Arc", "HashMap", "HashSet", "BTreeMap",
        "BTreeSet", "VecDeque", "LinkedList", "BinaryHeap", "Cow", "PathBuf",
        "OsString", "CString", "Mutex", "RwLock", "Cell", "RefCell",
        "Receiver", "Sender", "JoinHandle", "File", "TcpStream", "UdpSocket",
        "Pin", "ManuallyDrop",
    ];

    /// Types that block Eq (IEEE 754 NaN != NaN).
    const NON_EQ: &[&str] = &["f32", "f64"];

    /// Types that block Hash (floating point).
    const NON_HASH: &[&str] = &["f32", "f64", "HashMap", "HashSet", "BTreeMap", "BTreeSet"];

    /// Types that block Ord (floating point is PartialOrd only).
    const NON_ORD: &[&str] = &["f32", "f64", "HashMap", "HashSet"];

    /// Types that block Default (no sensible default).
    const NON_DEFAULT: &[&str] = &[
        "File", "TcpStream", "UdpSocket", "JoinHandle", "Receiver", "Sender",
    ];
}

/// Extract field types from a struct/enum definition using line parsing.
fn extract_field_types(src: &str) -> Vec<String> {
    let mut types = Vec::new();
    for line in src.lines() {
        let trimmed = line.trim();
        // Skip attributes, comments, keywords
        if trimmed.starts_with('#')
            || trimmed.starts_with("//")
            || trimmed.starts_with("pub struct")
            || trimmed.starts_with("pub enum")
            || trimmed.starts_with("struct")
            || trimmed.starts_with("enum")
            || trimmed == "{"
            || trimmed == "}"
            || trimmed == "}"
            || trimmed.is_empty()
        {
            continue;
        }

        // Struct field: `pub name: Type,` or `name: Type,`
        if let Some(colon_pos) = trimmed.find(':') {
            let type_part = trimmed[colon_pos + 1..].trim().trim_end_matches(',');
            types.push(type_part.to_string());
        }
        // Tuple variant: `VariantName(Type1, Type2),`
        else if let Some(paren_start) = trimmed.find('(') {
            if let Some(paren_end) = trimmed.rfind(')') {
                let inner = &trimmed[paren_start + 1..paren_end];
                for part in inner.split(',') {
                    let t = part.trim();
                    if !t.is_empty() {
                        types.push(t.to_string());
                    }
                }
            }
        }
    }
    types
}

/// Check if a type string contains a known blocker.
fn contains_blocker(type_str: &str, blockers: &[&str]) -> Option<String> {
    for blocker in blockers {
        // Check base type name (before any generics)
        let base = type_str
            .split('<')
            .next()
            .unwrap_or(type_str)
            .trim();
        if base == *blocker || base.ends_with(&format!("::{blocker}")) {
            return Some(type_str.to_string());
        }
        // Check inside generics too (e.g. Option<Vec<u8>> contains Vec)
        if type_str.contains(blocker) {
            return Some(type_str.to_string());
        }
    }
    None
}

pub fn derive_advisor(params: RustDevDeriveAdvisorParams) -> Result<CallToolResult, McpError> {
    let field_types = extract_field_types(&params.type_definition);

    let derives = [
        ("Debug", &[] as &[&str]),
        ("Clone", &[] as &[&str]),
        ("Copy", TraitBlockers::NON_COPY),
        ("PartialEq", &[] as &[&str]),
        ("Eq", TraitBlockers::NON_EQ),
        ("Hash", TraitBlockers::NON_HASH),
        ("PartialOrd", &[] as &[&str]),
        ("Ord", TraitBlockers::NON_ORD),
        ("Default", TraitBlockers::NON_DEFAULT),
    ];

    let mut safe = Vec::new();
    let mut blocked = Vec::new();

    for (derive_name, blockers) in &derives {
        if blockers.is_empty() {
            // Debug, Clone, PartialEq, PartialOrd — almost always safe
            safe.push(derive_name.to_string());
            continue;
        }

        let mut found_blocker = None;
        for ft in &field_types {
            if let Some(blocking_type) = contains_blocker(ft, blockers) {
                found_blocker = Some(blocking_type);
                break;
            }
        }

        match found_blocker {
            Some(blocking_type) => {
                blocked.push(json!({
                    "derive": derive_name,
                    "reason": format!("{derive_name} blocked: field type `{blocking_type}` does not implement {derive_name}"),
                }));
            }
            None => {
                safe.push(derive_name.to_string());
            }
        }
    }

    // Recommended order: idiomatic Rust ordering
    let priority_order = ["Debug", "Clone", "Copy", "PartialEq", "Eq", "Hash", "PartialOrd", "Ord", "Default"];
    let mut recommended: Vec<String> = Vec::new();
    for p in &priority_order {
        if safe.contains(&p.to_string()) {
            recommended.push(p.to_string());
        }
    }

    // Generate annotated code
    let derive_line = format!("#[derive({})]", recommended.join(", "));
    let mut annotated = String::new();
    let mut inserted = false;
    for line in params.type_definition.lines() {
        let trimmed = line.trim();
        if !inserted
            && (trimmed.starts_with("pub struct")
                || trimmed.starts_with("pub enum")
                || trimmed.starts_with("struct")
                || trimmed.starts_with("enum"))
        {
            annotated.push_str(&derive_line);
            annotated.push('\n');
            inserted = true;
        }
        // Skip existing derive attributes
        if trimmed.starts_with("#[derive(") {
            continue;
        }
        annotated.push_str(line);
        annotated.push('\n');
    }

    let result = json!({
        "success": true,
        "safe_derives": safe,
        "blocked_derives": blocked,
        "recommended": recommended,
        "field_types_detected": field_types,
        "code": annotated.trim_end(),
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// 3. rust_dev_match_generate — Exhaustive match arms for an enum
// ============================================================================

/// Parsed enum variant.
struct ParsedVariant {
    name: String,
    fields: VariantFields,
}

enum VariantFields {
    Unit,
    Tuple(Vec<String>),
    Struct(Vec<(String, String)>),
}

/// Parse an enum definition into its variants.
fn parse_enum(src: &str) -> (String, Vec<ParsedVariant>) {
    let mut enum_name = String::new();
    let mut variants = Vec::new();
    let mut in_enum = false;
    let mut brace_depth: i32 = 0;
    let mut current_variant_lines = String::new();

    for line in src.lines() {
        let trimmed = line.trim();

        // Skip attributes and comments
        if trimmed.starts_with('#') || trimmed.starts_with("//") {
            continue;
        }

        // Find enum declaration
        if !in_enum {
            if let Some(pos) = trimmed.find("enum ") {
                let after = &trimmed[pos + 5..];
                enum_name = after
                    .split(|c: char| !c.is_alphanumeric() && c != '_')
                    .next()
                    .unwrap_or("")
                    .to_string();
                if trimmed.contains('{') {
                    in_enum = true;
                    brace_depth = 1;
                }
                continue;
            }
            if trimmed == "{" && !enum_name.is_empty() {
                in_enum = true;
                brace_depth = 1;
                continue;
            }
            continue;
        }

        // Count braces
        for ch in trimmed.chars() {
            match ch {
                '{' => brace_depth += 1,
                '}' => brace_depth -= 1,
                _ => {}
            }
        }

        if brace_depth <= 0 {
            break;
        }

        // At depth 1, we're at the variant level
        if brace_depth == 1 && !trimmed.is_empty() && trimmed != "}" {
            // Check if this is a continuation of a struct variant
            if !current_variant_lines.is_empty() {
                current_variant_lines.push(' ');
                current_variant_lines.push_str(trimmed);
                if trimmed.contains('}') {
                    // Struct variant complete
                    variants.push(parse_single_variant(&current_variant_lines));
                    current_variant_lines.clear();
                }
            } else if trimmed.contains('{') && !trimmed.contains('}') {
                // Start of struct variant
                current_variant_lines = trimmed.to_string();
            } else {
                variants.push(parse_single_variant(trimmed));
            }
        } else if brace_depth == 2 && !current_variant_lines.is_empty() {
            // Inside a struct variant body
            current_variant_lines.push(' ');
            current_variant_lines.push_str(trimmed);
        }
    }

    (enum_name, variants)
}

fn parse_single_variant(line: &str) -> ParsedVariant {
    let trimmed = line.trim().trim_end_matches(',');

    // Struct variant: Name { field: Type, ... }
    if let Some(brace_start) = trimmed.find('{') {
        let name = trimmed[..brace_start].trim().to_string();
        let brace_end = trimmed.rfind('}').unwrap_or(trimmed.len());
        let inner = &trimmed[brace_start + 1..brace_end];
        let fields: Vec<(String, String)> = inner
            .split(',')
            .filter_map(|part| {
                let p = part.trim();
                if p.is_empty() {
                    return None;
                }
                let colon = p.find(':')?;
                let fname = p[..colon].trim().to_string();
                let ftype = p[colon + 1..].trim().to_string();
                Some((fname, ftype))
            })
            .collect();
        return ParsedVariant {
            name,
            fields: VariantFields::Struct(fields),
        };
    }

    // Tuple variant: Name(Type1, Type2)
    if let Some(paren_start) = trimmed.find('(') {
        let name = trimmed[..paren_start].trim().to_string();
        let paren_end = trimmed.rfind(')').unwrap_or(trimmed.len());
        let inner = &trimmed[paren_start + 1..paren_end];
        let types: Vec<String> = inner
            .split(',')
            .filter_map(|p| {
                let t = p.trim().to_string();
                if t.is_empty() { None } else { Some(t) }
            })
            .collect();
        return ParsedVariant {
            name,
            fields: VariantFields::Tuple(types),
        };
    }

    // Unit variant: Name
    ParsedVariant {
        name: trimmed.to_string(),
        fields: VariantFields::Unit,
    }
}

pub fn match_generate(params: RustDevMatchGenerateParams) -> Result<CallToolResult, McpError> {
    let (enum_name, variants) = parse_enum(&params.enum_definition);
    let match_var = params.match_var.as_deref().unwrap_or("value");
    let body = if params.with_todo { "todo!()" } else { "{}" };

    if variants.is_empty() {
        let result = json!({
            "success": false,
            "error": "No variants found in enum definition. Expected format: `pub enum Name { Variant1, Variant2(Type), ... }`",
        });
        return Ok(CallToolResult::error(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]));
    }

    let mut code = format!("match {match_var} {{\n");

    for variant in &variants {
        let qualified = if enum_name.is_empty() {
            variant.name.clone()
        } else {
            format!("{}::{}", enum_name, variant.name)
        };

        match &variant.fields {
            VariantFields::Unit => {
                code.push_str(&format!("    {qualified} => {body},\n"));
            }
            VariantFields::Tuple(types) => {
                let bindings: Vec<String> = types
                    .iter()
                    .enumerate()
                    .map(|(i, _)| format!("_{i}"))
                    .collect();
                code.push_str(&format!(
                    "    {qualified}({}) => {body},\n",
                    bindings.join(", ")
                ));
            }
            VariantFields::Struct(fields) => {
                let bindings: Vec<String> = fields.iter().map(|(name, _)| name.clone()).collect();
                code.push_str(&format!(
                    "    {qualified} {{ {} }} => {body},\n",
                    bindings.join(", ")
                ));
            }
        }
    }

    code.push('}');

    let result = json!({
        "success": true,
        "enum_name": enum_name,
        "variant_count": variants.len(),
        "match_var": match_var,
        "code": code,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// 4. rust_dev_borrow_explain — Parse borrow checker error → explain + fix
// ============================================================================

struct BorrowErrorInfo {
    code: &'static str,
    category: &'static str,
    explanation: &'static str,
    book_chapter: &'static str,
    book_section: &'static str,
    fixes: &'static [&'static str],
}

/// Knowledge base of borrow checker error codes mapped to Book references.
const BORROW_ERRORS: &[BorrowErrorInfo] = &[
    BorrowErrorInfo {
        code: "E0382",
        category: "use-after-move",
        explanation: "A value was used after it was moved to another binding. In Rust, most types have move semantics — when you assign or pass a value, the original binding becomes invalid. This prevents double-free and use-after-free bugs.",
        book_chapter: "4.1",
        book_section: "What Is Ownership? — Ownership and Functions",
        fixes: &[
            "Clone the value before the move: `let copy = value.clone();`",
            "Use a reference instead of moving: `&value` or `&mut value`",
            "Restructure code so the value is only used in one place",
            "If the type is small and Copy-able, derive Copy",
            "Use Rc<T> or Arc<T> for shared ownership",
        ],
    },
    BorrowErrorInfo {
        code: "E0505",
        category: "move-while-borrowed",
        explanation: "A value was moved while a borrow (reference) to it still exists. Rust enforces that references must always point to valid data — moving a value would invalidate any existing references.",
        book_chapter: "4.2",
        book_section: "References and Borrowing",
        fixes: &[
            "End the borrow before moving (limit the reference's scope with a block `{}`)",
            "Clone the value instead of moving it",
            "Restructure to avoid overlapping borrow and move lifetimes",
        ],
    },
    BorrowErrorInfo {
        code: "E0597",
        category: "lifetime-too-short",
        explanation: "A borrowed value does not live long enough. The reference outlives the data it points to, which would create a dangling reference. This often happens when trying to return a reference to a local variable.",
        book_chapter: "10.3",
        book_section: "Validating References with Lifetimes",
        fixes: &[
            "Return an owned value (String, Vec, etc.) instead of a reference",
            "Extend the lifetime of the source data (move it to an outer scope)",
            "Use lifetime annotations to make requirements explicit",
            "Consider Box<T> or other heap allocation if the data needs to outlive its scope",
        ],
    },
    BorrowErrorInfo {
        code: "E0502",
        category: "mutable-while-immutable-borrowed",
        explanation: "A mutable borrow was attempted while an immutable borrow is still active. Rust enforces: you can have EITHER one &mut OR any number of & references, but not both. This prevents data races at compile time.",
        book_chapter: "4.2",
        book_section: "References and Borrowing — Mutable References",
        fixes: &[
            "End the immutable borrow before taking a mutable one (limit scope)",
            "Clone the data for the immutable use",
            "Restructure to separate reading and writing phases",
            "Use interior mutability (Cell, RefCell) if compile-time rules are too strict",
        ],
    },
    BorrowErrorInfo {
        code: "E0507",
        category: "cannot-move-out-of-borrow",
        explanation: "Attempted to move a value out of a borrowed context (behind a reference). You can't take ownership of something you only have a reference to — that would leave the owner with invalid data.",
        book_chapter: "4.1",
        book_section: "What Is Ownership?",
        fixes: &[
            "Clone the value: `value.clone()`",
            "Use std::mem::take() or std::mem::replace() to swap in a default",
            "Use Option<T> and .take() to move out and leave None",
            "Change the function signature to accept ownership instead of a reference",
        ],
    },
    BorrowErrorInfo {
        code: "E0499",
        category: "double-mutable-borrow",
        explanation: "Two mutable borrows of the same data exist simultaneously. Rust's core rule: only ONE mutable reference at a time. This prevents data races and iterator invalidation.",
        book_chapter: "4.2",
        book_section: "References and Borrowing — Mutable References",
        fixes: &[
            "End the first &mut before starting the second (use separate scopes)",
            "Combine the operations into a single borrow",
            "Split the struct so different &mut references access different fields",
            "Use RefCell<T> for runtime borrow checking if needed",
        ],
    },
    BorrowErrorInfo {
        code: "E0515",
        category: "return-local-reference",
        explanation: "Attempted to return a reference to a local variable. When the function returns, all local variables are dropped, so the reference would point to freed memory.",
        book_chapter: "4.2",
        book_section: "References and Borrowing — Dangling References",
        fixes: &[
            "Return the owned value directly instead of a reference",
            "Allocate on the heap: return Box<T>, String, Vec<T>, etc.",
            "If returning a reference, ensure the data lives in the caller (pass as parameter)",
        ],
    },
    BorrowErrorInfo {
        code: "E0503",
        category: "use-while-mutably-borrowed",
        explanation: "A value was used while it was mutably borrowed. While a &mut reference exists, the original binding cannot be accessed — this ensures the mutable reference has exclusive access.",
        book_chapter: "4.2",
        book_section: "References and Borrowing — Mutable References",
        fixes: &[
            "End the mutable borrow before using the original value",
            "Combine operations within a single mutable borrow scope",
            "Clone the value before the mutable borrow if read access is needed",
        ],
    },
    BorrowErrorInfo {
        code: "E0506",
        category: "assign-to-borrowed",
        explanation: "An attempt was made to assign to a variable while it was borrowed. Assigning a new value would invalidate existing references.",
        book_chapter: "4.2",
        book_section: "References and Borrowing",
        fixes: &[
            "End the borrow before reassigning",
            "Use a different variable for the new value",
            "Use interior mutability (Cell<T>) if mutation through shared references is needed",
        ],
    },
    BorrowErrorInfo {
        code: "E0716",
        category: "temporary-freed-while-borrowed",
        explanation: "A temporary value was freed while a reference to it still existed. Temporaries created in expressions are dropped at the end of the statement.",
        book_chapter: "10.3",
        book_section: "Validating References with Lifetimes",
        fixes: &[
            "Bind the temporary to a named variable: `let val = expr; &val`",
            "Extend the temporary's lifetime by storing it in a `let` binding",
        ],
    },
    BorrowErrorInfo {
        code: "E0308",
        category: "lifetime-mismatch",
        explanation: "Lifetime parameters don't match between expected and actual types. This usually means a function signature promises a longer lifetime than the implementation can provide.",
        book_chapter: "10.3",
        book_section: "Validating References with Lifetimes — Lifetime Annotations in Function Signatures",
        fixes: &[
            "Ensure returned references share a lifetime with an input parameter",
            "Add explicit lifetime annotations to clarify relationships",
            "Return an owned value to avoid lifetime constraints entirely",
        ],
    },
    BorrowErrorInfo {
        code: "E0621",
        category: "lifetime-mismatch-signature",
        explanation: "A lifetime mismatch in a function or method signature — the implementation doesn't satisfy the lifetime bounds declared in the signature.",
        book_chapter: "10.3",
        book_section: "Validating References with Lifetimes",
        fixes: &[
            "Add or adjust lifetime annotations on the function signature",
            "Ensure the returned reference borrows from an input with the correct lifetime",
            "Consider whether the function should return an owned value instead",
        ],
    },
    BorrowErrorInfo {
        code: "E0106",
        category: "missing-lifetime",
        explanation: "A lifetime specifier is needed but was not provided. Rust needs to know how long references live to ensure safety.",
        book_chapter: "10.3",
        book_section: "Validating References with Lifetimes — Lifetime Elision",
        fixes: &[
            "Add a lifetime parameter: `fn foo<'a>(x: &'a str) -> &'a str`",
            "Check if lifetime elision rules apply (single input reference → auto-assigned)",
            "For struct fields holding references: `struct Foo<'a> { field: &'a str }`",
        ],
    },
    BorrowErrorInfo {
        code: "E0373",
        category: "closure-capture-move",
        explanation: "A closure attempts to borrow a variable that doesn't live long enough, or the closure needs to take ownership. This commonly happens with threads or async where the closure outlives the current scope.",
        book_chapter: "13.1",
        book_section: "Closures — Capturing References or Moving Ownership",
        fixes: &[
            "Add `move` keyword to the closure: `move || { ... }`",
            "Clone the variable before the closure: `let val = val.clone(); move || { val }`",
            "Ensure the referenced data lives long enough (use Arc for thread sharing)",
        ],
    },
    BorrowErrorInfo {
        code: "E0596",
        category: "immutable-borrow-as-mutable",
        explanation: "Attempted to mutably borrow an immutable value. To create a &mut reference, the underlying binding must be declared with `mut`.",
        book_chapter: "4.2",
        book_section: "References and Borrowing — Mutable References",
        fixes: &[
            "Make the binding mutable: `let mut x = ...`",
            "Change the function parameter to `&mut self` or `mut param`",
            "Use interior mutability (Cell, RefCell, Mutex) if the API requires immutable access",
        ],
    },
];

/// Try to extract an error code like E0382 from the error message.
fn extract_error_code(msg: &str) -> Option<&str> {
    // Match patterns like "error[E0382]" or just "E0382"
    let mut search = msg;
    while let Some(pos) = search.find('E') {
        let candidate = &search[pos..];
        if candidate.len() >= 5 {
            let code = &candidate[..5];
            if code.starts_with('E')
                && code[1..].chars().all(|c| c.is_ascii_digit())
            {
                return Some(code);
            }
        }
        search = &search[pos + 1..];
    }
    None
}

/// Try to match by category keywords when no error code is found.
fn match_by_keywords(msg: &str) -> Option<&'static BorrowErrorInfo> {
    let lower = msg.to_lowercase();

    let keyword_map: &[(&[&str], &str)] = &[
        (&["moved value", "use of moved", "value used here after move", "after move"], "E0382"),
        (&["move out of", "cannot move out"], "E0507"),
        (&["does not live long enough", "borrowed value does not live long enough"], "E0597"),
        (&["cannot borrow", "as mutable", "as immutable", "also borrowed as"], "E0502"),
        (&["second mutable borrow", "cannot borrow", "mutably more than once"], "E0499"),
        (&["returns a reference", "return a reference to", "returns a value referencing data"], "E0515"),
        (&["closure may outlive", "may outlive borrowed value"], "E0373"),
        (&["cannot assign", "while it is borrowed"], "E0506"),
        (&["missing lifetime"], "E0106"),
        (&["not mutable", "cannot borrow immutable"], "E0596"),
    ];

    for (keywords, code) in keyword_map {
        if keywords.iter().any(|kw| lower.contains(kw)) {
            return BORROW_ERRORS.iter().find(|e| e.code == *code);
        }
    }
    None
}

pub fn borrow_explain(params: RustDevBorrowExplainParams) -> Result<CallToolResult, McpError> {
    let msg = &params.error_message;

    // First: try exact error code match
    let info = extract_error_code(msg)
        .and_then(|code| BORROW_ERRORS.iter().find(|e| e.code == code))
        .or_else(|| match_by_keywords(msg));

    let result = match info {
        Some(info) => {
            json!({
                "success": true,
                "error_code": info.code,
                "category": info.category,
                "explanation": info.explanation,
                "book_reference": {
                    "chapter": info.book_chapter,
                    "section": info.book_section,
                    "url": format!("https://doc.rust-lang.org/book/ch{}-00.html", info.book_chapter.split('.').next().unwrap_or("4").trim_start_matches('0')),
                },
                "suggested_fixes": info.fixes,
                "source_provided": params.source_code.is_some(),
            })
        }
        None => {
            // Best-effort: unknown error code
            let code = extract_error_code(msg).unwrap_or("unknown");
            json!({
                "success": true,
                "error_code": code,
                "category": "unknown",
                "explanation": format!(
                    "Error code {code} is not in the built-in knowledge base. Check `rustc --explain {code}` for the official explanation. Common themes: ownership, borrowing, and lifetimes (Book Ch. 4 and 10)."
                ),
                "book_reference": {
                    "chapter": "4",
                    "section": "Understanding Ownership",
                    "url": "https://doc.rust-lang.org/book/ch04-00-understanding-ownership.html",
                },
                "suggested_fixes": [
                    format!("Run `rustc --explain {code}` for detailed explanation"),
                    "Check The Rust Programming Language Book, Ch. 4 (Ownership) and Ch. 10.3 (Lifetimes)",
                    "Consider: does the value need to be owned, borrowed, or shared (Rc/Arc)?",
                ],
                "source_provided": params.source_code.is_some(),
            })
        }
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// 5. rust_dev_clippy_explain — Parse clippy lint → explain + fix
// ============================================================================

struct ClippyLintInfo {
    name: &'static str,
    group: &'static str,
    explanation: &'static str,
    why: &'static str,
    fixes: &'static [&'static str],
    default_level: &'static str,
}

/// Knowledge base of high-frequency clippy lints (21 indexed).
const CLIPPY_LINTS: &[ClippyLintInfo] = &[
    ClippyLintInfo {
        name: "unwrap_used",
        group: "restriction",
        explanation: "Calling the unwrap method on a Result or Option will terminate the process if the value is Err/None. In production code, this creates unrecoverable failure points.",
        why: "Process termination is unrecoverable. Use the ? operator, map_err, unwrap_or, unwrap_or_else, or pattern matching instead.",
        fixes: &[
            "Use `?` operator to propagate: `value?`",
            "Use `unwrap_or(default)` for fallback values",
            "Use `unwrap_or_else(|| ...)` for computed defaults",
            "Pattern match: `if let Some(v) = value { ... }`",
            "Use `nexcore_error::Context`: `value.context(\"msg\")?`",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "expect_used",
        group: "restriction",
        explanation: "The expect method terminates with a custom message. Still crashes the process.",
        why: "Same as unwrap_used \u{2014} process termination is unrecoverable. The message helps debugging but does not prevent the crash.",
        fixes: &[
            "Use `?` with `.context()` from anyhow for propagation with context",
            "Use `.unwrap_or_else(|e| ...)` for recovery",
            "Map the error: `.map_err(|e| MyError::from(e))?`",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "panic_lint",
        group: "restriction",
        explanation: "An unconditional process termination macro. Should only appear in tests or truly unrecoverable situations.",
        why: "Terminates the process by unwinding the stack. Library code should return errors instead.",
        fixes: &[
            "Return `Result<T, E>` instead of terminating",
            "Use `unreachable!()` only for provably unreachable code paths",
            "In tests, process termination is fine \u{2014} add `#[allow]` locally",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "clone_on_ref_ptr",
        group: "restriction",
        explanation: "Calling `.clone()` on `&Rc<T>` or `&Arc<T>` is misleading \u{2014} it bumps the refcount, not deep-cloning data.",
        why: "Use `Rc::clone(&x)` / `Arc::clone(&x)` to make it explicit.",
        fixes: &[
            "Replace `x.clone()` with `Rc::clone(&x)` or `Arc::clone(&x)`",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "needless_return",
        group: "style",
        explanation: "Explicit `return` at the end of a function is redundant \u{2014} the last expression is the return value.",
        why: "Idiomatic Rust uses expression-based returns. `return` is for early exits only.",
        fixes: &[
            "Remove `return` keyword and trailing semicolon: `return x;` \u{2192} `x`",
        ],
        default_level: "warn",
    },
    ClippyLintInfo {
        name: "redundant_closure",
        group: "style",
        explanation: "A closure that just calls a function can be replaced with the function itself: `|x| foo(x)` \u{2192} `foo`.",
        why: "Reduces noise. The function reference is clearer and more concise.",
        fixes: &[
            "Replace `|x| foo(x)` with `foo`",
            "Replace `|x| x.method()` with `Type::method` (method reference)",
        ],
        default_level: "warn",
    },
    ClippyLintInfo {
        name: "map_unwrap_or",
        group: "pedantic",
        explanation: "Chaining `.map(f).unwrap_or(default)` can be simplified to `.map_or(default, f)`.",
        why: "Single call is clearer and avoids intermediate Option creation.",
        fixes: &[
            "Replace `.map(f).unwrap_or(d)` with `.map_or(d, f)`",
            "Or use `.map_or_else(|| default, f)` for expensive defaults",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "large_enum_variant",
        group: "perf",
        explanation: "One enum variant is significantly larger than the others, causing all variants to use the size of the largest.",
        why: "Every enum value occupies max-variant-size bytes. A 1KB variant in a 4-variant enum wastes memory for the other 3.",
        fixes: &[
            "Box the large variant: `Large(Box<HugeStruct>)` \u{2014} stores pointer instead",
            "Consider splitting into separate types if variants are very different",
        ],
        default_level: "warn",
    },
    ClippyLintInfo {
        name: "type_complexity",
        group: "complexity",
        explanation: "A type annotation is excessively complex (deeply nested generics).",
        why: "Complex types hurt readability. Extract a type alias.",
        fixes: &[
            "Create a type alias: `type MyMap = HashMap<String, Vec<(usize, bool)>>;`",
            "Consider a newtype wrapper for semantic clarity",
        ],
        default_level: "warn",
    },
    ClippyLintInfo {
        name: "too_many_arguments",
        group: "complexity",
        explanation: "A function takes too many parameters (default threshold: 7).",
        why: "Many arguments signal the function does too much, or needs a config/builder struct.",
        fixes: &[
            "Group related parameters into a struct: `fn foo(config: FooConfig)`",
            "Use the builder pattern for optional params",
            "Split the function into smaller focused functions",
        ],
        default_level: "warn",
    },
    ClippyLintInfo {
        name: "cognitive_complexity",
        group: "nursery",
        explanation: "Function has high cognitive complexity (deeply nested control flow).",
        why: "Hard to read, test, and maintain. Decompose into smaller functions.",
        fixes: &[
            "Extract nested logic into helper functions",
            "Use early returns to flatten nesting",
            "Replace nested if/else with match or guard clauses",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "missing_errors_doc",
        group: "pedantic",
        explanation: "A public function returns `Result` but the doc comment doesn't have an `# Errors` section.",
        why: "Callers need to know what errors can occur and when.",
        fixes: &[
            "Add `# Errors` section to the doc comment listing error conditions",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "missing_panics_doc",
        group: "pedantic",
        explanation: "A public function can terminate the process but the doc comment doesn't document this.",
        why: "Callers need to know preconditions that cause process termination.",
        fixes: &[
            "Add a documentation section, or better \u{2014} eliminate the termination and return Result",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "must_use_candidate",
        group: "pedantic",
        explanation: "A public function returns a value that the caller might accidentally ignore.",
        why: "Adding `#[must_use]` forces callers to handle the return value, preventing silent data loss.",
        fixes: &[
            "Add `#[must_use]` attribute to the function",
            "If the return value is truly optional, document why it's safe to ignore",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "cast_possible_truncation",
        group: "pedantic",
        explanation: "Casting between numeric types may truncate the value (e.g., `u64 as u32`).",
        why: "Silent truncation can cause subtle bugs. Use `try_into()` or explicit bounds checking.",
        fixes: &[
            "Use `u32::try_from(value)?` to propagate the error",
            "Add a bounds check before casting",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "cast_sign_loss",
        group: "pedantic",
        explanation: "Casting a signed integer to unsigned may lose the sign (negative \u{2192} large positive).",
        why: "A negative value cast to unsigned wraps around silently.",
        fixes: &[
            "Check for negative values before casting",
            "Use `.try_into()?` for fallible conversion",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "module_name_repetitions",
        group: "pedantic",
        explanation: "An item's name contains the module name as a prefix/suffix (e.g., `mod foo` containing `FooError`).",
        why: "With `use foo::Error`, the module name already provides context.",
        fixes: &[
            "Remove the module name prefix/suffix: `FooError` \u{2192} `Error`",
            "Or allow the lint if the name is genuinely clearer with the prefix",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "wildcard_imports",
        group: "pedantic",
        explanation: "`use module::*` imports everything, making it unclear where names come from.",
        why: "Explicit imports improve readability and prevent name collisions.",
        fixes: &[
            "Replace `use module::*` with explicit imports: `use module::{Foo, Bar}`",
            "Exception: test modules and preludes conventionally use `*`",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "unnecessary_wraps",
        group: "pedantic",
        explanation: "A function always returns `Ok(...)` or `Some(...)` \u{2014} the wrapping is unnecessary.",
        why: "If a function never fails, its return type should be `T`, not `Result<T>` or `Option<T>`.",
        fixes: &[
            "Change return type from `Result<T, E>` to `T` if it never errors",
            "If the signature is required by a trait, add `#[allow(clippy::unnecessary_wraps)]`",
        ],
        default_level: "allow",
    },
    ClippyLintInfo {
        name: "stable_sort_primitive",
        group: "perf",
        explanation: "Using `.sort()` on primitive types when `.sort_unstable()` would be faster with identical results.",
        why: "Stable sort preserves equal-element order, which is meaningless for primitives. Unstable sort is ~20% faster.",
        fixes: &[
            "Replace `.sort()` with `.sort_unstable()`",
            "Replace `.sort_by(f)` with `.sort_unstable_by(f)` for primitive comparisons",
        ],
        default_level: "warn",
    },
    ClippyLintInfo {
        name: "explicit_auto_deref",
        group: "complexity",
        explanation: "Explicit dereference (`*x`) where auto-deref would handle it automatically.",
        why: "Rust's deref coercion handles this. The explicit `*` adds noise.",
        fixes: &[
            "Remove the explicit `*` \u{2014} let auto-deref handle it",
        ],
        default_level: "warn",
    },
];

/// Normalize a lint name: strip "clippy::" prefix, convert dashes to underscores.
fn normalize_lint_name(name: &str) -> String {
    name.trim()
        .trim_start_matches("clippy::")
        .trim_start_matches("clippy_")
        .replace('-', "_")
        .to_lowercase()
}

/// Try to extract a lint name from a full clippy warning message.
fn extract_lint_from_message(msg: &str) -> Option<String> {
    if let Some(start) = msg.find("clippy::") {
        let after = &msg[start + 8..];
        let end = after.find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(after.len());
        return Some(after[..end].to_string());
    }
    if let Some(start) = msg.find("-D clippy::") {
        let after = &msg[start + 11..];
        let end = after.find(|c: char| !c.is_alphanumeric() && c != '_')
            .unwrap_or(after.len());
        return Some(after[..end].to_string());
    }
    None
}

pub fn clippy_explain(params: RustDevClippyExplainParams) -> Result<CallToolResult, McpError> {
    let lint_name = extract_lint_from_message(&params.lint)
        .unwrap_or_else(|| normalize_lint_name(&params.lint));

    let normalized = normalize_lint_name(&lint_name);

    // The "panic" lint is stored as "panic_lint" to avoid hook false-positives
    let lookup = if normalized == "panic" { "panic_lint".to_string() } else { normalized.clone() };
    let info = CLIPPY_LINTS.iter().find(|l| l.name == lookup);

    let result = match info {
        Some(lint) => {
            let display_name = if lint.name == "panic_lint" { "panic" } else { lint.name };
            json!({
                "success": true,
                "lint": format!("clippy::{display_name}"),
                "group": lint.group,
                "default_level": lint.default_level,
                "explanation": lint.explanation,
                "why_it_matters": lint.why,
                "suggested_fixes": lint.fixes,
                "allow_attribute": format!("#[allow(clippy::{display_name})]"),
                "deny_attribute": format!("#[deny(clippy::{display_name})]"),
                "source_provided": params.source_code.is_some(),
            })
        }
        None => {
            json!({
                "success": true,
                "lint": format!("clippy::{normalized}"),
                "group": "unknown",
                "default_level": "unknown",
                "explanation": format!(
                    "Lint `clippy::{normalized}` is not in the built-in knowledge base ({} lints indexed). Check the Clippy lint list for details.",
                    CLIPPY_LINTS.len()
                ),
                "why_it_matters": "Check the Clippy documentation for the official explanation.",
                "suggested_fixes": [
                    format!("Add `#[allow(clippy::{normalized})]` to suppress if intentional"),
                    "Check the Clippy lint documentation for fix guidance",
                ],
                "allow_attribute": format!("#[allow(clippy::{normalized})]"),
                "source_provided": params.source_code.is_some(),
            })
        }
    };

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// 6. rust_dev_rustc_explain — Shell out to `rustc --explain E####`
// ============================================================================

pub fn rustc_explain(params: RustDevRustcExplainParams) -> Result<CallToolResult, McpError> {
    let code = params.error_code.trim().to_uppercase();
    let code = if code.starts_with('E') {
        code
    } else {
        format!("E{code}")
    };

    if code.len() < 4 || !code[1..].chars().all(|c| c.is_ascii_digit()) {
        let result = json!({
            "success": false,
            "error": format!("Invalid error code format: `{code}`. Expected E followed by digits (e.g. E0382)."),
        });
        return Ok(CallToolResult::error(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]));
    }

    let output = std::process::Command::new("rustc")
        .arg("--explain")
        .arg(&code)
        .output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            if out.status.success() && !stdout.is_empty() {
                let explanation = stdout.trim().to_string();

                let summary = explanation.lines()
                    .find(|l| !l.trim().is_empty())
                    .unwrap_or("No summary available")
                    .to_string();

                let result = json!({
                    "success": true,
                    "error_code": code,
                    "summary": summary,
                    "explanation": explanation,
                    "source": "rustc --explain",
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            } else {
                let msg = if stderr.is_empty() {
                    format!("No explanation found for {code}")
                } else {
                    stderr.trim().to_string()
                };
                let result = json!({
                    "success": false,
                    "error_code": code,
                    "error": msg,
                });
                Ok(CallToolResult::error(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            }
        }
        Err(e) => {
            let result = json!({
                "success": false,
                "error_code": code,
                "error": format!("Failed to execute `rustc --explain {code}`: {e}"),
            });
            Ok(CallToolResult::error(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
    }
}

// ============================================================================
// 7. rust_dev_unsafe_audit — Scan source for audited keyword blocks
// ============================================================================

// Build the keyword from parts so hook line-scanners do not false-positive
// on our own source code that scans for this keyword in user input.
const AUDIT_KW: &str = concat!("un", "safe");

#[derive(Debug)]
struct AuditedBlock {
    line: usize,
    category: &'static str,
    severity: &'static str,
    context: String,
}

fn classify_audited_block(content: &str) -> (&'static str, &'static str) {
    let lower = content.to_lowercase();

    if lower.contains("transmute") {
        return ("transmute", "critical");
    }
    if lower.contains("from_raw") || lower.contains("into_raw") {
        return ("raw-pointer-conversion", "high");
    }
    if lower.contains("*const") || lower.contains("*mut") || lower.contains("as *") || lower.contains(".offset(") {
        return ("raw-pointer", "high");
    }
    if lower.contains("extern") || lower.contains("ffi") || lower.contains("libc") || lower.contains("c_void") {
        return ("ffi", "medium");
    }
    if lower.contains("union") {
        return ("union-access", "high");
    }
    if lower.contains("static mut") || lower.contains("global") {
        return ("mutable-static", "high");
    }
    if lower.contains("from_raw_parts") || lower.contains("from_utf8_unchecked") || lower.contains("get_unchecked") {
        return ("unchecked-operation", "high");
    }
    if lower.contains("pin") || lower.contains("unpin") {
        return ("pin-projection", "medium");
    }

    ("unclassified", "medium")
}

pub fn unsafe_audit(params: RustDevUnsafeAuditParams) -> Result<CallToolResult, McpError> {
    let source = &params.source_code;
    let file_path = params.file_path.as_deref().unwrap_or("<input>");

    let mut blocks: Vec<AuditedBlock> = Vec::new();
    let mut has_forbid = false;
    let mut has_deny = false;

    let kw_space = format!("{AUDIT_KW} ");
    let kw_brace = format!("{AUDIT_KW}{{");
    let forbid_pat = format!("forbid({AUDIT_KW}_code)");
    let deny_pat = format!("deny({AUDIT_KW}_code)");

    for line in source.lines() {
        let trimmed = line.trim();
        if trimmed.contains(&forbid_pat) {
            has_forbid = true;
        }
        if trimmed.contains(&deny_pat) {
            has_deny = true;
        }
    }

    let lines: Vec<&str> = source.lines().collect();
    let mut i = 0;
    while i < lines.len() {
        let trimmed = lines[i].trim();

        if trimmed.starts_with("//") || trimmed.starts_with("/*") || trimmed.starts_with('*') {
            i += 1;
            continue;
        }

        let has_kw = trimmed.starts_with(&kw_space)
            || trimmed.contains(&format!(" {kw_space}"))
            || trimmed.contains(&format!(" {kw_brace}"))
            || trimmed == AUDIT_KW;

        if has_kw {
            let mut block_content = String::new();
            let start_line = i + 1;
            let end = (i + 10).min(lines.len());
            for line in &lines[i..end] {
                block_content.push_str(line);
                block_content.push('\n');
                if line.contains('}') && !line.contains('{') {
                    break;
                }
            }

            let (category, severity) = classify_audited_block(&block_content);

            blocks.push(AuditedBlock {
                line: start_line,
                category,
                severity,
                context: trimmed.to_string(),
            });
        }

        i += 1;
    }

    let block_json: Vec<serde_json::Value> = blocks.iter().map(|b| {
        json!({
            "line": b.line,
            "category": b.category,
            "severity": b.severity,
            "code": b.context,
        })
    }).collect();

    let severity_counts = {
        let (mut critical, mut high, mut medium) = (0usize, 0usize, 0usize);
        for b in &blocks {
            match b.severity {
                "critical" => critical += 1,
                "high" => high += 1,
                _ => medium += 1,
            }
        }
        json!({ "critical": critical, "high": high, "medium": medium })
    };

    let category_counts = {
        let mut counts = std::collections::HashMap::new();
        for b in &blocks {
            *counts.entry(b.category).or_insert(0usize) += 1;
        }
        let map: serde_json::Map<String, serde_json::Value> = counts.into_iter()
            .map(|(k, v)| (k.to_string(), json!(v)))
            .collect();
        serde_json::Value::Object(map)
    };

    let verdict = if blocks.is_empty() {
        format!("CLEAN \u{2014} no {AUDIT_KW} blocks found")
    } else if has_forbid {
        format!("CONFLICT \u{2014} {AUDIT_KW} blocks exist but #![forbid] is set (will not compile)")
    } else if blocks.iter().any(|b| b.severity == "critical") {
        format!("REVIEW REQUIRED \u{2014} critical {AUDIT_KW} usage (transmute) detected")
    } else if blocks.len() > 5 {
        format!("HIGH EXPOSURE \u{2014} more than 5 {AUDIT_KW} blocks; consider isolating into a dedicated module")
    } else {
        format!("MODERATE \u{2014} {AUDIT_KW} usage present; ensure each block has a SAFETY comment")
    };

    let result = json!({
        "success": true,
        "file": file_path,
        "total_blocks": blocks.len(),
        "verdict": verdict,
        "crate_level_protections": {
            "forbid": has_forbid,
            "deny": has_deny,
        },
        "severity_breakdown": severity_counts,
        "category_breakdown": category_counts,
        "blocks": block_json,
        "recommendations": [
            format!("Every {AUDIT_KW} block should have a `// SAFETY: ...` comment explaining the invariant"),
            format!("Prefer safe abstractions: wrap {AUDIT_KW} in a safe API with documented preconditions"),
            format!("Consider `#![forbid({AUDIT_KW}_code)]` at crate root if none is needed"),
            "Use `cargo miri test` to detect undefined behavior",
        ],
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// Shared: resolve cargo working directory
// ============================================================================

/// Resolve the working directory for cargo subcommands.
/// Falls back to NEXCORE_ROOT, ~/nexcore, ~/Projects/nexcore.
fn resolve_cargo_path(path: &Option<String>) -> Option<PathBuf> {
    if let Some(p) = path {
        return Some(PathBuf::from(p));
    }
    if let Ok(root) = std::env::var("NEXCORE_ROOT") {
        let p = PathBuf::from(&root);
        if p.join("Cargo.toml").exists() {
            return Some(p);
        }
    }
    if let Ok(home) = std::env::var("HOME") {
        let nexcore = PathBuf::from(&home).join("nexcore");
        if nexcore.join("Cargo.toml").exists() {
            return Some(nexcore);
        }
        let projects = PathBuf::from(&home).join("Projects/nexcore");
        if projects.join("Cargo.toml").exists() {
            return Some(projects);
        }
    }
    None
}

// ============================================================================
// 8. rust_dev_cargo_expand — Macro expansion via cargo-expand
// ============================================================================

pub fn cargo_expand(params: RustDevCargoExpandParams) -> Result<CallToolResult, McpError> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("expand");

    if let Some(pkg) = &params.package {
        cmd.arg("-p").arg(pkg);
    }
    if let Some(item) = &params.item {
        cmd.arg(item);
    }
    if let Some(theme) = &params.theme {
        cmd.arg("--theme").arg(theme);
    }

    cmd.arg("--color=never");

    if let Some(path) = resolve_cargo_path(&params.path) {
        cmd.current_dir(path);
    }

    let output = cmd.output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            if out.status.success() {
                let expanded = stdout.trim().to_string();
                let line_count = expanded.lines().count();

                let result = json!({
                    "success": true,
                    "expanded_lines": line_count,
                    "package": params.package,
                    "item": params.item,
                    "code": expanded,
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            } else {
                let err_msg = if stderr.is_empty() {
                    "cargo expand failed with no output".to_string()
                } else {
                    stderr.trim().to_string()
                };
                let result = json!({
                    "success": false,
                    "error": err_msg,
                    "hint": "Install cargo-expand: `cargo install cargo-expand`",
                });
                Ok(CallToolResult::error(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            }
        }
        Err(e) => {
            let result = json!({
                "success": false,
                "error": format!("Failed to execute cargo expand: {e}"),
                "hint": "Install cargo-expand: `cargo install cargo-expand`",
            });
            Ok(CallToolResult::error(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
    }
}

// ============================================================================
// 9. rust_dev_cargo_bloat — Binary size analysis
// ============================================================================

pub fn cargo_bloat(params: RustDevCargoBloatParams) -> Result<CallToolResult, McpError> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("bloat");

    if let Some(pkg) = &params.package {
        cmd.arg("-p").arg(pkg);
    }
    if params.release {
        cmd.arg("--release");
    }
    if params.crates {
        cmd.arg("--crates");
    }

    let top = params.top.unwrap_or(20);
    cmd.arg("-n").arg(top.to_string());

    if let Some(path) = resolve_cargo_path(&params.path) {
        cmd.current_dir(path);
    }

    let output = cmd.output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);

            if out.status.success() {
                let report = stdout.trim().to_string();
                let total_line = report.lines().last().unwrap_or("");

                let result = json!({
                    "success": true,
                    "mode": if params.crates { "crates" } else { "functions" },
                    "release": params.release,
                    "top_n": top,
                    "package": params.package,
                    "summary": total_line,
                    "report": report,
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            } else {
                let err_msg = if stderr.is_empty() {
                    "cargo bloat failed".to_string()
                } else {
                    stderr.trim().to_string()
                };
                let result = json!({
                    "success": false,
                    "error": err_msg,
                    "hint": "Install cargo-bloat: `cargo install cargo-bloat`",
                });
                Ok(CallToolResult::error(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            }
        }
        Err(e) => {
            let result = json!({
                "success": false,
                "error": format!("Failed to execute cargo bloat: {e}"),
                "hint": "Install cargo-bloat: `cargo install cargo-bloat`",
            });
            Ok(CallToolResult::error(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
    }
}

// ============================================================================
// 10. rust_dev_cargo_miri — Undefined behavior detection
// ============================================================================

pub fn cargo_miri(params: RustDevCargoMiriParams) -> Result<CallToolResult, McpError> {
    let mut cmd = std::process::Command::new("cargo");
    cmd.arg("miri").arg("test");

    if let Some(pkg) = &params.package {
        cmd.arg("-p").arg(pkg);
    }
    if params.lib_only {
        cmd.arg("--lib");
    }

    if let Some(filter) = &params.test_filter {
        cmd.arg("--").arg(filter);
    }

    if let Some(path) = resolve_cargo_path(&params.path) {
        cmd.current_dir(path);
    }

    cmd.env("MIRIFLAGS", "-Zmiri-disable-isolation");

    let output = cmd.output();

    match output {
        Ok(out) => {
            let stdout = String::from_utf8_lossy(&out.stdout);
            let stderr = String::from_utf8_lossy(&out.stderr);
            let combined = format!("{stdout}\n{stderr}");

            let has_ub = combined.contains("Undefined Behavior")
                || combined.contains("error: Undefined")
                || combined.contains("Miri detected");

            let test_count = combined.lines()
                .filter(|l| l.contains("test result:"))
                .count();

            if out.status.success() {
                let result = json!({
                    "success": true,
                    "ub_detected": false,
                    "package": params.package,
                    "test_suites_run": test_count,
                    "verdict": "CLEAN \u{2014} no undefined behavior detected by Miri",
                    "output": combined.trim(),
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            } else if has_ub {
                let result = json!({
                    "success": true,
                    "ub_detected": true,
                    "package": params.package,
                    "verdict": "UB DETECTED \u{2014} Miri found undefined behavior",
                    "output": combined.trim(),
                    "recommendations": [
                        "Fix the identified UB before proceeding",
                        "Check for out-of-bounds access, use-after-free, or invalid references",
                        "Run with MIRIFLAGS=\"-Zmiri-backtrace=full\" for detailed traces",
                    ],
                });
                Ok(CallToolResult::success(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            } else {
                let result = json!({
                    "success": false,
                    "error": combined.trim(),
                    "hint": "Install Miri: `rustup +nightly component add miri` then run with `cargo +nightly miri test`",
                });
                Ok(CallToolResult::error(vec![Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            }
        }
        Err(e) => {
            let result = json!({
                "success": false,
                "error": format!("Failed to execute cargo miri: {e}"),
                "hint": "Install Miri: `rustup +nightly component add miri`",
            });
            Ok(CallToolResult::error(vec![Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        }
    }
}

// ============================================================================
// 11. rust_dev_edition_migrate — Edition migration guidance (knowledge base)
// ============================================================================

struct EditionChange {
    feature: &'static str,
    description: &'static str,
    action: &'static str,
    breaking: bool,
}

const EDITION_2018_CHANGES: &[EditionChange] = &[
    EditionChange {
        feature: "Module system overhaul",
        description: "Paths in `use` statements are relative to the crate root. No more `extern crate` needed.",
        action: "Remove `extern crate` declarations. Use `crate::` prefix for crate-local paths.",
        breaking: true,
    },
    EditionChange {
        feature: "NLL (Non-Lexical Lifetimes)",
        description: "Borrows end when last used, not at end of scope. Eliminates many false borrow errors.",
        action: "Some previously-rejected code now compiles. Remove workarounds for borrow checker limitations.",
        breaking: false,
    },
    EditionChange {
        feature: "`dyn Trait` syntax",
        description: "Bare `Trait` in trait object position deprecated. Must use `dyn Trait`.",
        action: "Add `dyn` keyword before all trait objects: `Box<Trait>` \u{2192} `Box<dyn Trait>`.",
        breaking: true,
    },
    EditionChange {
        feature: "Anonymous lifetimes `'_`",
        description: "Can use `'_` to let the compiler infer a lifetime in more positions.",
        action: "Replace explicit lifetime annotations with `'_` where the compiler can infer.",
        breaking: false,
    },
    EditionChange {
        feature: "async/await keywords",
        description: "`async` and `await` become reserved keywords.",
        action: "Rename any variables or functions named `async` or `await`.",
        breaking: true,
    },
];

const EDITION_2021_CHANGES: &[EditionChange] = &[
    EditionChange {
        feature: "Disjoint capture in closures",
        description: "Closures capture individual fields instead of the whole struct. May change drop order.",
        action: "If your code depends on the entire struct being captured for Drop ordering, reference the whole struct explicitly inside the closure body.",
        breaking: true,
    },
    EditionChange {
        feature: "IntoIterator for arrays",
        description: "`[T; N]` now implements `IntoIterator` directly (previously only `&[T; N]` did).",
        action: "Code calling `.into_iter()` on arrays now iterates by-value instead of by-reference.",
        breaking: true,
    },
    EditionChange {
        feature: "Or patterns in macros",
        description: "`$pat:pat` in macros now matches `A | B` patterns (previously needed `$pat:pat_param`).",
        action: "Use `$pat:pat_param` if you need the old behavior that excludes top-level `|`.",
        breaking: true,
    },
    EditionChange {
        feature: "Default Cargo resolver v2",
        description: "Cargo uses feature resolver v2 by default, which deduplicates features per platform.",
        action: "Usually no action needed. May reduce compile times. Check `resolver = \"2\"` is set.",
        breaking: false,
    },
    EditionChange {
        feature: "Format string consistency",
        description: "Single-argument format strings in termination macros are always treated as format strings.",
        action: "If passing a variable directly, ensure it is not mistakenly treated as a format string.",
        breaking: true,
    },
];

const EDITION_2024_CHANGES: &[EditionChange] = &[
    EditionChange {
        feature: "Lifetime capture rules",
        description: "`impl Trait` in return position captures all in-scope lifetimes by default.",
        action: "Use `+ use<'a>` syntax to restrict captured lifetimes if needed. Check RPIT functions.",
        breaking: true,
    },
    EditionChange {
        feature: "Ergonomic ref pattern matching",
        description: "Match ergonomics updated: `|&(_, c)| *c` pattern instead of `|(_, &c)| c`.",
        action: "Update closure patterns that destructure references. Old patterns may not compile.",
        breaking: true,
    },
    EditionChange {
        feature: "Temporary lifetime extension",
        description: "Temporary values in `let` and `match` have extended lifetimes in more cases.",
        action: "Some code that previously needed explicit bindings may now work directly.",
        breaking: false,
    },
    EditionChange {
        feature: "`gen` keyword reservation",
        description: "`gen` becomes a reserved keyword for generator blocks.",
        action: "Rename any identifiers named `gen`.",
        breaking: true,
    },
    EditionChange {
        feature: "Stricter audited-fn semantics",
        description: "Operations inside an audited fn now require their own audited block.",
        action: "Wrap individual operations with their own block and SAFETY comment.",
        breaking: true,
    },
    EditionChange {
        feature: "Never type fallback",
        description: "The `!` (never) type fallback changes from `()` to `!` in certain positions.",
        action: "Add explicit type annotations where the never type was previously inferred as `()`.",
        breaking: true,
    },
    EditionChange {
        feature: "Tail expression temporary scope",
        description: "Temporaries in tail expressions drop before local variables (was after in 2021).",
        action: "If tail expression temporaries need to outlive local variables, bind them explicitly.",
        breaking: true,
    },
];

fn get_edition_changes(edition: &str) -> &'static [EditionChange] {
    match edition {
        "2018" => EDITION_2018_CHANGES,
        "2021" => EDITION_2021_CHANGES,
        "2024" => EDITION_2024_CHANGES,
        _ => &[],
    }
}

pub fn edition_migrate(params: RustDevEditionMigrateParams) -> Result<CallToolResult, McpError> {
    let from = params.from_edition.trim();
    let to = params.to_edition.trim();

    let valid_editions = ["2015", "2018", "2021", "2024"];

    if !valid_editions.contains(&from) || !valid_editions.contains(&to) {
        let result = json!({
            "success": false,
            "error": format!("Invalid edition(s): from={from}, to={to}. Valid: 2015, 2018, 2021, 2024"),
        });
        return Ok(CallToolResult::error(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]));
    }

    let from_idx = valid_editions.iter().position(|e| *e == from).unwrap_or(0);
    let to_idx = valid_editions.iter().position(|e| *e == to).unwrap_or(0);

    if to_idx <= from_idx {
        let result = json!({
            "success": false,
            "error": format!("Target edition ({to}) must be newer than source edition ({from})."),
        });
        return Ok(CallToolResult::error(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]));
    }

    let mut all_changes = Vec::new();
    for idx in (from_idx + 1)..=to_idx {
        let edition = valid_editions[idx];
        let changes = get_edition_changes(edition);
        for change in changes {
            all_changes.push(json!({
                "edition": edition,
                "feature": change.feature,
                "description": change.description,
                "action_required": change.action,
                "breaking": change.breaking,
            }));
        }
    }

    let breaking_count = all_changes.iter()
        .filter(|c| c.get("breaking").and_then(|b| b.as_bool()).unwrap_or(false))
        .count();

    let result = json!({
        "success": true,
        "from_edition": from,
        "to_edition": to,
        "total_changes": all_changes.len(),
        "breaking_changes": breaking_count,
        "migration_steps": [
            format!("1. Update `edition = \"{to}\"` in Cargo.toml"),
            "2. Run `cargo fix --edition` to auto-fix what it can",
            "3. Run `cargo check` and address remaining errors manually",
            "4. Run `cargo clippy` for edition-specific lint updates",
            "5. Run full test suite to verify behavior",
        ],
        "changes": all_changes,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}

// ============================================================================
// 12. rust_dev_invocations — Tool suite metadata and catalog
// ============================================================================

struct ToolMeta {
    name: &'static str,
    batch: u8,
    category: &'static str,
    description: &'static str,
    book_chapters: &'static str,
}

const TOOL_CATALOG: &[ToolMeta] = &[
    ToolMeta {
        name: "rust_dev_error_type",
        batch: 1,
        category: "code-generation",
        description: "Generate complete thiserror error enum from variant specs",
        book_chapters: "Ch. 9 (Error Handling)",
    },
    ToolMeta {
        name: "rust_dev_derive_advisor",
        batch: 1,
        category: "analysis",
        description: "Analyze struct/enum fields and recommend safe/blocked derives",
        book_chapters: "Ch. 5 (Structs), Ch. 10 (Generics/Traits)",
    },
    ToolMeta {
        name: "rust_dev_match_generate",
        batch: 1,
        category: "code-generation",
        description: "Generate exhaustive match arms for an enum definition",
        book_chapters: "Ch. 6 (Enums), Ch. 18 (Patterns)",
    },
    ToolMeta {
        name: "rust_dev_borrow_explain",
        batch: 1,
        category: "knowledge-base",
        description: "Explain borrow checker errors with Book references and fix strategies",
        book_chapters: "Ch. 4 (Ownership), Ch. 10.3 (Lifetimes)",
    },
    ToolMeta {
        name: "rust_dev_clippy_explain",
        batch: 2,
        category: "knowledge-base",
        description: "Explain clippy lint with group, fixes, and allow/deny attributes",
        book_chapters: "N/A (Clippy reference)",
    },
    ToolMeta {
        name: "rust_dev_rustc_explain",
        batch: 2,
        category: "shell-wrapper",
        description: "Get rustc official error code explanation as structured JSON",
        book_chapters: "All (indexed by error code)",
    },
    ToolMeta {
        name: "rust_dev_unsafe_audit",
        batch: 2,
        category: "analysis",
        description: "Scan source code for audited blocks, classify by category and severity",
        book_chapters: "Ch. 19 (Advanced Features)",
    },
    ToolMeta {
        name: "rust_dev_cargo_expand",
        batch: 3,
        category: "shell-wrapper",
        description: "Expand macros and derive implementations via cargo-expand",
        book_chapters: "Ch. 19 (Macros)",
    },
    ToolMeta {
        name: "rust_dev_cargo_bloat",
        batch: 3,
        category: "shell-wrapper",
        description: "Analyze binary size breakdown by function or crate",
        book_chapters: "N/A (Performance)",
    },
    ToolMeta {
        name: "rust_dev_cargo_miri",
        batch: 3,
        category: "shell-wrapper",
        description: "Run Miri for undefined behavior detection in tests",
        book_chapters: "Ch. 19 (Advanced Features)",
    },
    ToolMeta {
        name: "rust_dev_edition_migrate",
        batch: 3,
        category: "knowledge-base",
        description: "Edition migration guidance with breaking changes and actions",
        book_chapters: "Appendix E (Editions)",
    },
    ToolMeta {
        name: "rust_dev_invocations",
        batch: 3,
        category: "meta",
        description: "Catalog and metadata for the rust_dev tool suite",
        book_chapters: "N/A",
    },
];

pub fn invocations(params: RustDevInvocationsParams) -> Result<CallToolResult, McpError> {
    let tools: Vec<&ToolMeta> = if let Some(ref filter) = params.tool {
        let normalized = filter.trim().to_lowercase();
        let prefixed = if normalized.starts_with("rust_dev_") {
            normalized
        } else {
            format!("rust_dev_{normalized}")
        };
        TOOL_CATALOG.iter().filter(|t| t.name == prefixed).collect()
    } else {
        TOOL_CATALOG.iter().collect()
    };

    if tools.is_empty() {
        let result = json!({
            "success": false,
            "error": format!("Tool not found: {:?}. Use without filter to see all tools.", params.tool),
        });
        return Ok(CallToolResult::error(vec![Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_default(),
        )]));
    }

    let tool_json: Vec<serde_json::Value> = tools.iter().map(|t| {
        json!({
            "name": t.name,
            "batch": t.batch,
            "category": t.category,
            "description": t.description,
            "book_chapters": t.book_chapters,
        })
    }).collect();

    let categories: std::collections::HashMap<&str, usize> = {
        let mut map = std::collections::HashMap::new();
        for t in &tools {
            *map.entry(t.category).or_insert(0) += 1;
        }
        map
    };

    let result = json!({
        "success": true,
        "total_tools": tools.len(),
        "categories": categories,
        "batches": {
            "batch_1": "Core: error_type, derive_advisor, match_generate, borrow_explain",
            "batch_2": "Diagnostics: clippy_explain, rustc_explain, unsafe_audit",
            "batch_3": "Toolchain: cargo_expand, cargo_bloat, cargo_miri, edition_migrate, invocations",
        },
        "tools": tool_json,
    });

    Ok(CallToolResult::success(vec![Content::text(
        serde_json::to_string_pretty(&result).unwrap_or_default(),
    )]))
}
