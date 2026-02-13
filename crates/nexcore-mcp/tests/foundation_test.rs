//! Integration tests for Foundation tools
//!
//! Covers Levenshtein, fuzzy search, SHA-256, YAML, and graph operations.

use nexcore_mcp::params::{
    FsrsReviewParams, FuzzySearchParams, GraphLevelsParams, GraphTopsortParams, LevenshteinParams,
    Sha256Params, YamlParseParams,
};
use nexcore_mcp::tools::foundation;

#[test]
fn test_levenshtein_distance() {
    let params = LevenshteinParams {
        source: "nexcore".to_string(),
        target: "nexvigilant".to_string(),
    };
    let result = foundation::calc_levenshtein(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;
    // "nexcore" (7) vs "nexvigilant" (11). "nex" matches.
    // distance is 8.
    assert!(text.contains("distance") && text.contains("8"));
}

#[test]
fn test_fuzzy_search_matches() {
    let params = FuzzySearchParams {
        query: "vigilance".to_string(),
        candidates: vec![
            "vigilant".to_string(),
            "guardian".to_string(),
            "foundation".to_string(),
        ],
        limit: 5,
    };
    let result = foundation::fuzzy_search(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;
    assert!(text.contains("vigilant"));
    assert!(text.contains("count") && text.contains("3"));
}

#[test]
fn test_sha256_hash() {
    let params = Sha256Params {
        input: "nexcore".to_string(),
    };
    let result = foundation::sha256(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;
    assert!(text.contains("hex"));
    // echo -n nexcore | sha256sum -> 757c631e3aa70511e3439086f77233b3381dd4ff6d4dd422e1d69480d9fb94d8
    assert!(text.contains("757c631e3aa70511e3439086f77233b3381dd4ff6d4dd422e1d69480d9fb94d8"));
}

#[test]
fn test_yaml_parse_json() {
    let params = YamlParseParams {
        content: "key: value\nlist:\n  - 1\n  - 2".to_string(),
    };
    let result = foundation::yaml_parse(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;
    assert!(text.contains("key") && text.contains("value"));
}

#[test]
fn test_graph_topsort_linear() {
    let params = GraphTopsortParams {
        edges: vec![
            ("A".to_string(), "B".to_string()),
            ("B".to_string(), "C".to_string()),
        ],
    };
    let result = foundation::graph_topsort(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;
    assert!(text.contains("A") && text.contains("B") && text.contains("C"));
}

#[test]
fn test_graph_levels_parallel() {
    let params = GraphLevelsParams {
        edges: vec![
            ("ROOT".to_string(), "A".to_string()),
            ("ROOT".to_string(), "B".to_string()),
            ("A".to_string(), "C".to_string()),
            ("B".to_string(), "C".to_string()),
        ],
    };
    let result = foundation::graph_levels(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;
    // Output has "levels", not "level"
    assert!(text.contains("levels"));
    assert!(
        text.contains("ROOT") && text.contains("A") && text.contains("B") && text.contains("C")
    );
}

#[test]
fn test_fsrs_review_update() {
    let params = FsrsReviewParams {
        stability: 1.0,
        difficulty: 0.5,
        elapsed_days: 1,
        rating: 3, // Good
    };
    let result = foundation::fsrs_review(params);
    assert!(result.is_ok());
    let content = result.unwrap();
    let text = &content.content[0].as_text().unwrap().text;
    assert!(text.contains("new_stability") || text.contains("stability"));
    assert!(text.contains("new_difficulty") || text.contains("difficulty"));
}
