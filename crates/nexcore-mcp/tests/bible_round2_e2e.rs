//! End-to-end tests for AI Engineering Bible Round 2 tools.
//!
//! Tests: drift_detection (4), rate_limiter (3), rank_fusion (3)

use nexcore_mcp::params::*;
use nexcore_mcp::tools::{drift_detection, rank_fusion, rate_limiter};

// ============================================================================
// Drift Detection Tests
// ============================================================================

#[test]
fn test_drift_ks_test_no_drift() {
    // Two samples from same distribution — should detect no drift
    let params = DriftKsTestParams {
        reference: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        current: vec![1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5, 8.5, 9.5, 10.5],
        alpha: Some(0.05),
    };
    let result = drift_detection::drift_ks_test(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("STABLE"), "Expected STABLE verdict: {}", text);
}

#[test]
fn test_drift_ks_test_with_drift() {
    // Two very different distributions — should detect drift
    let params = DriftKsTestParams {
        reference: vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 1.5, 2.5, 3.5, 4.5, 5.5, 6.5, 7.5,
            8.5, 9.5, 10.5,
        ],
        current: vec![
            50.0, 51.0, 52.0, 53.0, 54.0, 55.0, 56.0, 57.0, 58.0, 59.0, 50.5, 51.5, 52.5, 53.5,
            54.5, 55.5, 56.5, 57.5, 58.5, 59.5,
        ],
        alpha: Some(0.05),
    };
    let result = drift_detection::drift_ks_test(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("DRIFT"), "Expected DRIFT verdict: {}", text);
}

#[test]
fn test_drift_psi_stable() {
    // Very similar bin distributions
    let params = DriftPsiParams {
        reference: vec![0.10, 0.15, 0.20, 0.25, 0.30],
        current: vec![0.11, 0.14, 0.21, 0.24, 0.30],
        bins: None,
        raw_data: Some(false),
    };
    let result = drift_detection::drift_psi(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(text.contains("NO_DRIFT"), "Expected NO_DRIFT: {}", text);
}

#[test]
fn test_drift_psi_significant() {
    // Very different distributions
    let params = DriftPsiParams {
        reference: vec![0.50, 0.30, 0.10, 0.05, 0.05],
        current: vec![0.05, 0.05, 0.10, 0.30, 0.50],
        bins: None,
        raw_data: Some(false),
    };
    let result = drift_detection::drift_psi(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("SIGNIFICANT_DRIFT") || text.contains("MODERATE_DRIFT"),
        "Expected drift detected: {}",
        text
    );
}

#[test]
fn test_drift_psi_raw_data() {
    let params = DriftPsiParams {
        reference: vec![1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0],
        current: vec![5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 11.0, 12.0, 13.0, 14.0],
        bins: Some(5),
        raw_data: Some(true),
    };
    let result = drift_detection::drift_psi(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("population_stability_index"),
        "Expected PSI result: {}",
        text
    );
}

#[test]
fn test_drift_jsd_identical() {
    // Identical distributions → JSD ≈ 0
    let params = DriftJsdParams {
        p: vec![0.25, 0.25, 0.25, 0.25],
        q: vec![0.25, 0.25, 0.25, 0.25],
        base2: Some(true),
    };
    let result = drift_detection::drift_jsd(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("minimal"),
        "Expected minimal divergence: {}",
        text
    );
}

#[test]
fn test_drift_jsd_different() {
    // Very different distributions
    let params = DriftJsdParams {
        p: vec![0.9, 0.05, 0.025, 0.025],
        q: vec![0.025, 0.025, 0.05, 0.9],
        base2: Some(true),
    };
    let result = drift_detection::drift_jsd(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    // Should be substantial or severe
    assert!(
        text.contains("substantial") || text.contains("severe"),
        "Expected high divergence: {}",
        text
    );
}

#[test]
fn test_drift_detect_composite_stable() {
    let params = DriftDetectParams {
        reference: vec![
            1.0, 2.0, 3.0, 4.0, 5.0, 6.0, 7.0, 8.0, 9.0, 10.0, 1.1, 2.1, 3.1, 4.1, 5.1, 6.1, 7.1,
            8.1, 9.1, 10.1,
        ],
        current: vec![
            1.2, 2.2, 3.2, 4.2, 5.2, 6.2, 7.2, 8.2, 9.2, 10.2, 1.3, 2.3, 3.3, 4.3, 5.3, 6.3, 7.3,
            8.3, 9.3, 10.3,
        ],
        bins: Some(5),
        alpha: Some(0.05),
        psi_threshold: Some(0.25),
    };
    let result = drift_detection::drift_detect(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("STABLE") || text.contains("WATCH"),
        "Expected stable/watch: {}",
        text
    );
}

#[test]
fn test_drift_detect_composite_drifted() {
    // Use overlapping range so binning captures distribution shape differences.
    // Reference: concentrated at low end (1-10), Current: concentrated at high end (40-50),
    // both spanning 1-50 range so bins capture the shift.
    let mut reference = Vec::new();
    for i in 0..40 {
        reference.push(1.0 + (i as f64) * 0.25);
    } // 1.0 to 10.75
    let mut current = Vec::new();
    for i in 0..40 {
        current.push(40.0 + (i as f64) * 0.25);
    } // 40.0 to 49.75
    // Add a few overlapping points so binning has shared range
    reference.extend_from_slice(&[25.0, 30.0, 35.0, 40.0, 45.0]);
    current.extend_from_slice(&[5.0, 10.0, 15.0, 20.0, 25.0]);

    let params = DriftDetectParams {
        reference,
        current,
        bins: Some(10),
        alpha: Some(0.05),
        psi_threshold: Some(0.25),
    };
    let result = drift_detection::drift_detect(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("DRIFT_CONFIRMED") || text.contains("DRIFT_LIKELY") || text.contains("WATCH"),
        "Expected drift signal: {}",
        text
    );
    // At minimum KS should detect drift
    assert!(
        text.contains("\"drift\": true"),
        "KS should detect drift: {}",
        text
    );
}

// ============================================================================
// Rate Limiter Tests
// ============================================================================

#[test]
fn test_rate_limit_token_bucket_allow() {
    let params = RateLimitTokenBucketParams {
        bucket_id: "test_e2e_bucket_1".to_string(),
        capacity: Some(100),
        refill_rate: Some(10.0),
        cost: Some(1),
        dry_run: Some(false),
    };
    let result = rate_limiter::rate_limit_token_bucket(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("\"allowed\": true"),
        "Expected allowed: {}",
        text
    );
}

#[test]
fn test_rate_limit_token_bucket_exhaust() {
    // Exhaust bucket with large cost
    let params = RateLimitTokenBucketParams {
        bucket_id: "test_e2e_bucket_exhaust".to_string(),
        capacity: Some(5),
        refill_rate: Some(0.01), // Very slow refill
        cost: Some(10),          // More than capacity
        dry_run: Some(false),
    };
    let result = rate_limiter::rate_limit_token_bucket(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("\"allowed\": false"),
        "Expected rejected: {}",
        text
    );
    assert!(
        text.contains("retry_after_secs"),
        "Expected retry info: {}",
        text
    );
}

#[test]
fn test_rate_limit_token_bucket_dry_run() {
    let params = RateLimitTokenBucketParams {
        bucket_id: "test_e2e_bucket_dry".to_string(),
        capacity: Some(100),
        refill_rate: Some(10.0),
        cost: Some(1),
        dry_run: Some(true),
    };
    let result = rate_limiter::rate_limit_token_bucket(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("\"dry_run\": true"),
        "Expected dry_run flag: {}",
        text
    );
    assert!(
        text.contains("\"allowed\": true"),
        "Expected allowed: {}",
        text
    );
}

#[test]
fn test_rate_limit_sliding_window_allow() {
    let params = RateLimitSlidingWindowParams {
        window_id: "test_e2e_window_1".to_string(),
        max_requests: Some(60),
        window_secs: Some(60),
        dry_run: Some(false),
    };
    let result = rate_limiter::rate_limit_sliding_window(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("\"allowed\": true"),
        "Expected allowed: {}",
        text
    );
    assert!(
        text.contains("\"remaining\""),
        "Expected remaining count: {}",
        text
    );
}

#[test]
fn test_rate_limit_sliding_window_exhaust() {
    // Create window with max_requests=2, then exhaust it
    for i in 0..3 {
        let params = RateLimitSlidingWindowParams {
            window_id: "test_e2e_window_exhaust".to_string(),
            max_requests: Some(2),
            window_secs: Some(60),
            dry_run: Some(false),
        };
        let result = rate_limiter::rate_limit_sliding_window(params);
        assert!(result.is_ok());
        let text = extract_text(&result.unwrap());
        if i >= 2 {
            assert!(
                text.contains("\"allowed\": false"),
                "3rd request should be rejected: {}",
                text
            );
        }
    }
}

#[test]
fn test_rate_limit_status() {
    // First create a bucket
    let _ = rate_limiter::rate_limit_token_bucket(RateLimitTokenBucketParams {
        bucket_id: "test_e2e_status_bucket".to_string(),
        capacity: Some(50),
        refill_rate: Some(5.0),
        cost: Some(1),
        dry_run: Some(false),
    });

    // Then query status
    let params = RateLimitStatusParams { id: None };
    let result = rate_limiter::rate_limit_status(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("token_buckets"),
        "Expected bucket list: {}",
        text
    );
    assert!(
        text.contains("sliding_windows"),
        "Expected window list: {}",
        text
    );
}

#[test]
fn test_rate_limit_status_specific() {
    // Create then query specific bucket
    let _ = rate_limiter::rate_limit_token_bucket(RateLimitTokenBucketParams {
        bucket_id: "test_e2e_specific".to_string(),
        capacity: Some(25),
        refill_rate: Some(2.0),
        cost: Some(1),
        dry_run: Some(false),
    });

    let params = RateLimitStatusParams {
        id: Some("test_e2e_specific".to_string()),
    };
    let result = rate_limiter::rate_limit_status(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("token_bucket"),
        "Expected bucket type: {}",
        text
    );
    assert!(
        text.contains("\"capacity\": 25"),
        "Expected capacity 25: {}",
        text
    );
}

// ============================================================================
// Rank Fusion Tests
// ============================================================================

#[test]
fn test_rank_fusion_rrf_basic() {
    let mut rankings = std::collections::HashMap::new();
    rankings.insert(
        "bm25".to_string(),
        vec![
            "doc1".to_string(),
            "doc3".to_string(),
            "doc2".to_string(),
            "doc5".to_string(),
        ],
    );
    rankings.insert(
        "semantic".to_string(),
        vec![
            "doc2".to_string(),
            "doc1".to_string(),
            "doc4".to_string(),
            "doc3".to_string(),
        ],
    );

    let params = RankFusionRrfParams {
        rankings,
        k: Some(60),
        limit: Some(5),
        weights: None,
    };
    let result = rank_fusion::rank_fusion_rrf(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("reciprocal_rank_fusion"),
        "Expected RRF algo: {}",
        text
    );
    assert!(text.contains("doc1"), "Expected doc1 in results: {}", text);
    assert!(text.contains("doc2"), "Expected doc2 in results: {}", text);
    // doc1 and doc2 should be top-ranked (both appear in both lists at high positions)
}

#[test]
fn test_rank_fusion_rrf_weighted() {
    let mut rankings = std::collections::HashMap::new();
    rankings.insert(
        "bm25".to_string(),
        vec!["doc1".to_string(), "doc2".to_string()],
    );
    rankings.insert(
        "neural".to_string(),
        vec!["doc2".to_string(), "doc1".to_string()],
    );

    let mut weights = std::collections::HashMap::new();
    weights.insert("bm25".to_string(), 0.3);
    weights.insert("neural".to_string(), 0.7);

    let params = RankFusionRrfParams {
        rankings,
        k: Some(60),
        limit: Some(5),
        weights: Some(weights),
    };
    let result = rank_fusion::rank_fusion_rrf(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    // With higher neural weight, doc2 (ranked #1 by neural) should score higher
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("valid JSON");
    let results = parsed["results"].as_array().expect("results array");
    assert!(!results.is_empty(), "Expected results");
    let top_item = results[0]["item_id"].as_str().expect("item_id");
    assert_eq!(
        top_item, "doc2",
        "doc2 should be top with neural weight 0.7"
    );
}

#[test]
fn test_rank_fusion_hybrid() {
    let mut dense = std::collections::HashMap::new();
    dense.insert("doc1".to_string(), 0.95);
    dense.insert("doc2".to_string(), 0.80);
    dense.insert("doc3".to_string(), 0.60);

    let mut sparse = std::collections::HashMap::new();
    sparse.insert("doc1".to_string(), 12.5);
    sparse.insert("doc2".to_string(), 15.0);
    sparse.insert("doc4".to_string(), 18.0);

    let params = RankFusionHybridParams {
        dense_scores: dense,
        sparse_scores: sparse,
        alpha: Some(0.6),
        limit: Some(10),
        normalize_sparse: Some(true),
    };
    let result = rank_fusion::rank_fusion_hybrid(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("hybrid_interpolation"),
        "Expected hybrid algo: {}",
        text
    );
    assert!(
        text.contains("dense_only"),
        "doc3 should be dense_only: {}",
        text
    );
    assert!(
        text.contains("sparse_only"),
        "doc4 should be sparse_only: {}",
        text
    );
    assert!(
        text.contains("both"),
        "doc1/doc2 should be in both: {}",
        text
    );
}

#[test]
fn test_rank_fusion_hybrid_alpha_effect() {
    let mut dense = std::collections::HashMap::new();
    dense.insert("doc_dense".to_string(), 1.0);
    dense.insert("doc_sparse".to_string(), 0.0);

    let mut sparse = std::collections::HashMap::new();
    sparse.insert("doc_dense".to_string(), 0.0);
    sparse.insert("doc_sparse".to_string(), 1.0);

    // alpha=1.0 → pure dense, doc_dense should win
    let params = RankFusionHybridParams {
        dense_scores: dense.clone(),
        sparse_scores: sparse.clone(),
        alpha: Some(1.0),
        limit: Some(2),
        normalize_sparse: Some(false),
    };
    let result = rank_fusion::rank_fusion_hybrid(params);
    assert!(result.is_ok());
    let parsed: serde_json::Value =
        serde_json::from_str(&extract_text(&result.unwrap())).expect("JSON");
    let top = parsed["results"][0]["item_id"].as_str().expect("item_id");
    assert_eq!(top, "doc_dense", "alpha=1.0 should favor dense: {}", top);

    // alpha=0.0 → pure sparse, doc_sparse should win
    let params = RankFusionHybridParams {
        dense_scores: dense,
        sparse_scores: sparse,
        alpha: Some(0.0),
        limit: Some(2),
        normalize_sparse: Some(false),
    };
    let result = rank_fusion::rank_fusion_hybrid(params);
    assert!(result.is_ok());
    let parsed: serde_json::Value =
        serde_json::from_str(&extract_text(&result.unwrap())).expect("JSON");
    let top = parsed["results"][0]["item_id"].as_str().expect("item_id");
    assert_eq!(top, "doc_sparse", "alpha=0.0 should favor sparse: {}", top);
}

#[test]
fn test_rank_fusion_borda() {
    let mut rankings = std::collections::HashMap::new();
    rankings.insert(
        "system_a".to_string(),
        vec!["doc1".to_string(), "doc2".to_string(), "doc3".to_string()],
    );
    rankings.insert(
        "system_b".to_string(),
        vec!["doc2".to_string(), "doc3".to_string(), "doc1".to_string()],
    );
    rankings.insert(
        "system_c".to_string(),
        vec!["doc2".to_string(), "doc1".to_string(), "doc3".to_string()],
    );

    let params = RankFusionBordaParams {
        rankings,
        limit: Some(5),
    };
    let result = rank_fusion::rank_fusion_borda(params);
    assert!(result.is_ok());
    let text = extract_text(&result.unwrap());
    assert!(
        text.contains("borda_count"),
        "Expected Borda algo: {}",
        text
    );
    // doc2 is ranked 1st by 2 systems, 2nd by 1 → highest Borda score
    let parsed: serde_json::Value = serde_json::from_str(&text).expect("JSON");
    let top = parsed["results"][0]["item_id"].as_str().expect("item_id");
    assert_eq!(top, "doc2", "doc2 should win Borda count");
}

// ============================================================================
// Error Handling Tests
// ============================================================================

#[test]
fn test_drift_ks_empty_input() {
    let params = DriftKsTestParams {
        reference: vec![],
        current: vec![1.0, 2.0],
        alpha: None,
    };
    let result = drift_detection::drift_ks_test(params);
    assert!(result.is_err(), "Empty reference should error");
}

#[test]
fn test_drift_jsd_mismatched_length() {
    let params = DriftJsdParams {
        p: vec![0.5, 0.5],
        q: vec![0.33, 0.33, 0.34],
        base2: None,
    };
    let result = drift_detection::drift_jsd(params);
    assert!(result.is_err(), "Mismatched lengths should error");
}

#[test]
fn test_drift_psi_mismatched_bins() {
    let params = DriftPsiParams {
        reference: vec![0.5, 0.5],
        current: vec![0.33, 0.33, 0.34],
        bins: None,
        raw_data: Some(false),
    };
    let result = drift_detection::drift_psi(params);
    assert!(result.is_err(), "Mismatched bin counts should error");
}

#[test]
fn test_rank_fusion_rrf_empty() {
    let params = RankFusionRrfParams {
        rankings: std::collections::HashMap::new(),
        k: None,
        limit: None,
        weights: None,
    };
    let result = rank_fusion::rank_fusion_rrf(params);
    assert!(result.is_err(), "Empty rankings should error");
}

#[test]
fn test_rate_limit_status_not_found() {
    let params = RateLimitStatusParams {
        id: Some("nonexistent_bucket_xyz".to_string()),
    };
    let result = rate_limiter::rate_limit_status(params);
    assert!(result.is_err(), "Nonexistent bucket should error");
}

// ============================================================================
// Helper
// ============================================================================

fn extract_text(result: &rmcp::model::CallToolResult) -> String {
    result.content[0]
        .as_text()
        .map(|t| t.text.clone())
        .unwrap_or_default()
}
