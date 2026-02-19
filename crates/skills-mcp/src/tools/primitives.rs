use rmcp::ErrorData as McpError;
use rmcp::model::CallToolResult;
use serde_json::json;

use super::common::*;
use crate::types::{PrimitiveComposeParams, PrimitiveDecomposeParams};

/// The 16 T1 Lex Primitiva with their metadata.
struct T1Primitive {
    symbol: &'static str,
    name: &'static str,
    meaning: &'static str,
    rust_pattern: &'static str,
    keywords: &'static [&'static str],
}

const PRIMITIVES: &[T1Primitive] = &[
    T1Primitive {
        symbol: "→",
        name: "Causality",
        meaning: "Cause produces effect",
        rust_pattern: "fn, callbacks, event triggers, signals",
        keywords: &[
            "cause",
            "effect",
            "trigger",
            "event",
            "callback",
            "handler",
            "emit",
            "fire",
            "dispatch",
            "notify",
            "invoke",
            "call",
            "execute",
            "run",
            "produce",
            "result",
            "consequence",
            "reaction",
            "response",
            "action",
        ],
    },
    T1Primitive {
        symbol: "N",
        name: "Quantity",
        meaning: "Numerical magnitude",
        rust_pattern: "u32, f64, usize, numeric types",
        keywords: &[
            "count",
            "number",
            "amount",
            "size",
            "length",
            "total",
            "sum",
            "average",
            "rate",
            "ratio",
            "percentage",
            "score",
            "metric",
            "measure",
            "calculate",
            "compute",
            "numeric",
            "integer",
            "float",
            "magnitude",
        ],
    },
    T1Primitive {
        symbol: "∃",
        name: "Existence",
        meaning: "Instantiation of being",
        rust_pattern: "new(), constructors, Some, create",
        keywords: &[
            "create",
            "new",
            "instantiate",
            "construct",
            "initialize",
            "spawn",
            "allocate",
            "register",
            "add",
            "insert",
            "generate",
            "build",
            "make",
            "exist",
            "presence",
            "available",
            "found",
            "discover",
        ],
    },
    T1Primitive {
        symbol: "κ",
        name: "Comparison",
        meaning: "Predicate matching",
        rust_pattern: "==, Ord, match, if let, filter",
        keywords: &[
            "compare",
            "equal",
            "match",
            "check",
            "test",
            "verify",
            "validate",
            "assert",
            "filter",
            "select",
            "predicate",
            "condition",
            "criteria",
            "threshold",
            "greater",
            "less",
            "sort",
            "order",
            "rank",
            "classify",
        ],
    },
    T1Primitive {
        symbol: "ς",
        name: "State",
        meaning: "Encapsulated context at a point",
        rust_pattern: "struct, Mutex, Cell, typestates",
        keywords: &[
            "state",
            "status",
            "context",
            "config",
            "setting",
            "property",
            "field",
            "attribute",
            "variable",
            "mutable",
            "update",
            "modify",
            "change",
            "transition",
            "toggle",
            "flag",
            "mode",
            "phase",
            "session",
            "snapshot",
        ],
    },
    T1Primitive {
        symbol: "μ",
        name: "Mapping",
        meaning: "Transformation A→B",
        rust_pattern: "From/Into, map(), and_then()",
        keywords: &[
            "map",
            "transform",
            "convert",
            "translate",
            "parse",
            "serialize",
            "deserialize",
            "encode",
            "decode",
            "format",
            "adapt",
            "bridge",
            "projection",
            "derive",
            "extract",
            "lookup",
            "index",
            "dictionary",
            "association",
            "key-value",
        ],
    },
    T1Primitive {
        symbol: "σ",
        name: "Sequence",
        meaning: "Ordered succession",
        rust_pattern: "Iterator, Vec, method chains, pipeline",
        keywords: &[
            "sequence",
            "list",
            "array",
            "vector",
            "iterate",
            "loop",
            "stream",
            "pipeline",
            "chain",
            "flow",
            "step",
            "order",
            "queue",
            "stack",
            "batch",
            "series",
            "progression",
            "next",
            "previous",
            "sequential",
        ],
    },
    T1Primitive {
        symbol: "ρ",
        name: "Recursion",
        meaning: "Self-reference via indirection",
        rust_pattern: "Box<Self>, recursive enums, tree traversal",
        keywords: &[
            "recursive",
            "tree",
            "nested",
            "hierarchical",
            "self-referential",
            "depth",
            "traverse",
            "walk",
            "visit",
            "fold",
            "reduce",
            "ast",
            "graph",
            "parent",
            "child",
            "ancestor",
            "descendant",
            "fractal",
        ],
    },
    T1Primitive {
        symbol: "∅",
        name: "Void",
        meaning: "Meaningful absence",
        rust_pattern: "Option::None, (), !, PhantomData",
        keywords: &[
            "none",
            "null",
            "empty",
            "absent",
            "missing",
            "void",
            "nothing",
            "optional",
            "default",
            "fallback",
            "placeholder",
            "phantom",
            "skip",
            "ignore",
            "omit",
            "blank",
            "zero-sized",
            "unit",
        ],
    },
    T1Primitive {
        symbol: "∂",
        name: "Boundary",
        meaning: "Limits and transitions",
        rust_pattern: "Result, guards, max_iterations, timeout",
        keywords: &[
            "boundary",
            "limit",
            "constraint",
            "guard",
            "error",
            "result",
            "timeout",
            "deadline",
            "maximum",
            "minimum",
            "cap",
            "ceiling",
            "floor",
            "range",
            "bound",
            "edge",
            "permission",
            "access",
            "rule",
            "policy",
            "restrict",
            "prevent",
        ],
    },
    T1Primitive {
        symbol: "ν",
        name: "Frequency",
        meaning: "Rate of occurrence",
        rust_pattern: "counters, rate limiters, polling, intervals",
        keywords: &[
            "frequency",
            "rate",
            "interval",
            "periodic",
            "poll",
            "tick",
            "heartbeat",
            "sampling",
            "throttle",
            "debounce",
            "recurring",
            "schedule",
            "cron",
            "timer",
            "cooldown",
            "burst",
            "window",
        ],
    },
    T1Primitive {
        symbol: "λ",
        name: "Location",
        meaning: "Positional context",
        rust_pattern: "Path, pointers, indices, URLs, addresses",
        keywords: &[
            "path",
            "location",
            "address",
            "position",
            "index",
            "offset",
            "pointer",
            "reference",
            "url",
            "uri",
            "route",
            "endpoint",
            "coordinate",
            "cursor",
            "anchor",
            "target",
            "destination",
        ],
    },
    T1Primitive {
        symbol: "π",
        name: "Persistence",
        meaning: "Continuity through time",
        rust_pattern: "DB, files, static, logs, cache",
        keywords: &[
            "persist",
            "store",
            "save",
            "write",
            "database",
            "file",
            "disk",
            "cache",
            "log",
            "record",
            "archive",
            "backup",
            "snapshot",
            "durable",
            "permanent",
            "retain",
            "remember",
            "history",
        ],
    },
    T1Primitive {
        symbol: "∝",
        name: "Irreversibility",
        meaning: "One-way state transition",
        rust_pattern: "Drop, consuming methods, hashes, delete",
        keywords: &[
            "irreversible",
            "consume",
            "drop",
            "destroy",
            "delete",
            "remove",
            "hash",
            "digest",
            "one-way",
            "entropy",
            "finalize",
            "commit",
            "seal",
            "burn",
            "spend",
            "deplete",
            "exhaust",
            "terminal",
        ],
    },
    T1Primitive {
        symbol: "Σ",
        name: "Sum",
        meaning: "Exclusive disjunction (one of N)",
        rust_pattern: "enum, match, Either, coproduct",
        keywords: &[
            "enum",
            "variant",
            "union",
            "either",
            "choice",
            "alternative",
            "discriminant",
            "tag",
            "kind",
            "type",
            "category",
            "branch",
            "switch",
            "dispatch",
            "polymorphic",
            "disjunction",
        ],
    },
    T1Primitive {
        symbol: "×",
        name: "Product",
        meaning: "Conjunctive combination",
        rust_pattern: "struct, tuples, zip(), records",
        keywords: &[
            "struct",
            "tuple",
            "record",
            "pair",
            "combine",
            "zip",
            "join",
            "merge",
            "bundle",
            "aggregate",
            "compose",
            "conjunction",
            "together",
            "both",
            "all",
            "group",
            "collection",
            "compound",
        ],
    },
];

fn find_by_name_or_symbol(input: &str) -> Option<&'static T1Primitive> {
    let lower = input.to_lowercase();
    PRIMITIVES
        .iter()
        .find(|p| p.name.to_lowercase() == lower || p.symbol == input)
}

/// Decompose a concept into its T1 primitive components.
pub fn decompose(params: PrimitiveDecomposeParams) -> Result<CallToolResult, McpError> {
    let concept = params.concept.to_lowercase();
    let words: Vec<&str> = concept.split_whitespace().collect();

    let mut matches: Vec<(&T1Primitive, f64)> = Vec::new();

    for prim in PRIMITIVES {
        let mut score = 0.0_f64;
        let mut matched_keywords = Vec::new();

        for keyword in prim.keywords {
            // Check if any word in the concept matches or contains the keyword
            for word in &words {
                let word_clean = word.trim_matches(|c: char| !c.is_alphanumeric());
                if word_clean == *keyword
                    || word_clean.contains(keyword)
                    || keyword.contains(word_clean) && word_clean.len() >= 3
                {
                    score += 1.0;
                    matched_keywords.push(*keyword);
                    break;
                }
            }
        }

        // Also check the full concept string for keyword substrings
        for keyword in prim.keywords {
            if concept.contains(keyword) && !matched_keywords.contains(keyword) {
                score += 0.5;
                matched_keywords.push(*keyword);
            }
        }

        if score > 0.0 {
            matches.push((prim, score));
        }
    }

    // Sort by score descending
    matches.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(std::cmp::Ordering::Equal));

    // Take top matches (all with score > 0)
    let dominant = matches.first().map(|(p, _)| p.name);

    let primitives: Vec<serde_json::Value> = matches
        .iter()
        .map(|(p, score)| {
            json!({
                "symbol": p.symbol,
                "name": p.name,
                "meaning": p.meaning,
                "relevance": (*score * 100.0).round() / 100.0,
                "rust_pattern": p.rust_pattern,
            })
        })
        .collect();

    // Determine tier
    let tier = match primitives.len() {
        0 => "T3-DomainSpecific (no T1 primitives detected — likely domain-specific)",
        1 => "T1-Universal (pure primitive)",
        2..=3 => "T2-P (cross-domain primitive composite)",
        _ => "T2-C (cross-domain composite)",
    };

    // Generate composition suggestion
    let composition = matches
        .iter()
        .take(5)
        .map(|(p, _)| format!("{}({})", p.symbol, p.name))
        .collect::<Vec<_>>()
        .join(" + ");

    Ok(json_result(&json!({
        "concept": params.concept,
        "primitives": primitives,
        "dominant": dominant,
        "composition": composition,
        "primitive_count": primitives.len(),
        "tier": tier,
        "suggestion": if primitives.is_empty() {
            "This concept may be T3 domain-specific. Try decomposing it into sub-concepts first."
        } else {
            "Use the identified primitives to guide your Rust type design and solution architecture."
        }
    })))
}

/// Given T1 primitives, describe what they compose into.
pub fn compose(params: PrimitiveComposeParams) -> Result<CallToolResult, McpError> {
    let resolved: Vec<&T1Primitive> = params
        .primitives
        .iter()
        .filter_map(|name| find_by_name_or_symbol(name))
        .collect();

    let unresolved: Vec<&String> = params
        .primitives
        .iter()
        .filter(|name| find_by_name_or_symbol(name).is_none())
        .collect();

    // Known composition patterns
    let symbols: Vec<&str> = resolved.iter().map(|p| p.symbol).collect();
    let mut patterns = Vec::new();

    // Check for known compositions
    if symbols.contains(&"σ") && symbols.contains(&"μ") && symbols.contains(&"∂") {
        patterns.push("Filtered Pipeline: Iterator.filter().map() with Result boundary");
    }
    if symbols.contains(&"ς") && symbols.contains(&"κ") && symbols.contains(&"∂") {
        patterns.push("Validation: State checked against Comparison at Boundary");
    }
    if symbols.contains(&"∃") && symbols.contains(&"π") && symbols.contains(&"∝") {
        patterns.push("Audit Log: Existence persisted irreversibly");
    }
    if symbols.contains(&"μ") && symbols.contains(&"π") && symbols.contains(&"∂") {
        patterns.push("Cache: Mapping with Persistence and eviction Boundary");
    }
    if symbols.contains(&"ν") && symbols.contains(&"∂") && symbols.contains(&"ς") {
        patterns.push("Rate Limiter: Frequency tracked in State with Boundary cap");
    }
    if symbols.contains(&"σ") && symbols.contains(&"ρ") {
        patterns.push("Tree Traversal: Recursive Sequence over nested structure");
    }
    if symbols.contains(&"Σ") && symbols.contains(&"κ") {
        patterns.push("Pattern Matching: Sum type with Comparison dispatch");
    }
    if symbols.contains(&"×") && symbols.contains(&"μ") {
        patterns.push("Record Mapping: Product type transformation");
    }
    if symbols.contains(&"→") && symbols.contains(&"σ") {
        patterns.push("Event Pipeline: Causal chain in sequence");
    }
    if symbols.contains(&"∅") && symbols.contains(&"Σ") {
        patterns.push("Optional Variant: Void + Sum = Option<T>");
    }
    if symbols.contains(&"ς") && symbols.contains(&"→") && symbols.contains(&"ν") {
        patterns.push("State Machine: State transitions caused at frequency intervals");
    }
    if symbols.contains(&"λ") && symbols.contains(&"μ") {
        patterns.push("Router: Location-based Mapping (URL dispatch, path resolution)");
    }

    let composition = resolved
        .iter()
        .map(|p| format!("{}({})", p.symbol, p.name))
        .collect::<Vec<_>>()
        .join(" + ");

    let tier = match resolved.len() {
        0 => "Unknown",
        1 => "T1-Universal",
        2..=3 => "T2-P",
        _ => "T2-C",
    };

    Ok(json_result(&json!({
        "composition": composition,
        "primitives": resolved.iter().map(|p| json!({
            "symbol": p.symbol,
            "name": p.name,
            "meaning": p.meaning,
            "rust_pattern": p.rust_pattern,
        })).collect::<Vec<_>>(),
        "known_patterns": patterns,
        "tier": tier,
        "unresolved": unresolved,
    })))
}

/// List all 16 T1 Lex Primitiva with full metadata.
pub fn list_all() -> Result<CallToolResult, McpError> {
    let all: Vec<serde_json::Value> = PRIMITIVES
        .iter()
        .map(|p| {
            json!({
                "symbol": p.symbol,
                "name": p.name,
                "meaning": p.meaning,
                "rust_pattern": p.rust_pattern,
            })
        })
        .collect();

    Ok(json_result(&json!({
        "count": 16,
        "root_primitives": ["→ (Causality)", "N (Quantity)"],
        "primitives": all,
    })))
}
