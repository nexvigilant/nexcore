//! Unit tests for nexcore MCP Server
//!
//! Tests tool implementations, parameter validation, and JSON serialization.

use crate::params::*;
use crate::tools::{foundation, game_theory, pv, skills, vigilance};

// ============================================================================
// Foundation Tool Tests
// ============================================================================

mod foundation_tests {
    use super::*;

    #[test]
    fn test_levenshtein_basic() {
        let params = LevenshteinParams {
            source: "kitten".to_string(),
            target: "sitting".to_string(),
        };
        let result = foundation::calc_levenshtein(params);
        assert!(result.is_ok());
        let content = format!("{:?}", result.unwrap());
        assert!(content.contains("distance"));
    }

    #[test]
    fn test_levenshtein_identical() {
        let params = LevenshteinParams {
            source: "hello".to_string(),
            target: "hello".to_string(),
        };
        let result = foundation::calc_levenshtein(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_levenshtein_empty() {
        let params = LevenshteinParams {
            source: String::new(),
            target: "hello".to_string(),
        };
        let result = foundation::calc_levenshtein(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sha256_known_value() {
        let params = Sha256Params {
            input: "hello world".to_string(),
        };
        let result = foundation::sha256(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_sha256_empty() {
        let params = Sha256Params {
            input: String::new(),
        };
        let result = foundation::sha256(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_yaml_parse_valid() {
        let params = YamlParseParams {
            content: "name: test\nversion: 1.0".to_string(),
        };
        let result = foundation::yaml_parse(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_yaml_parse_invalid() {
        let params = YamlParseParams {
            content: "invalid: yaml: content: [".to_string(),
        };
        let result = foundation::yaml_parse(params);
        // Should return Ok with error message, not Err
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_topsort_valid() {
        let params = GraphTopsortParams {
            edges: vec![
                ("a".to_string(), "b".to_string()),
                ("b".to_string(), "c".to_string()),
            ],
        };
        let result = foundation::graph_topsort(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_graph_topsort_cycle() {
        let params = GraphTopsortParams {
            edges: vec![
                ("a".to_string(), "b".to_string()),
                ("b".to_string(), "a".to_string()),
            ],
        };
        let result = foundation::graph_topsort(params);
        assert!(result.is_ok()); // Returns Ok with cycle info
    }

    #[test]
    fn test_graph_levels() {
        let params = GraphLevelsParams {
            edges: vec![
                ("a".to_string(), "b".to_string()),
                ("a".to_string(), "c".to_string()),
                ("b".to_string(), "d".to_string()),
                ("c".to_string(), "d".to_string()),
            ],
        };
        let result = foundation::graph_levels(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fsrs_review_valid() {
        let params = FsrsReviewParams {
            stability: 10.0,
            difficulty: 0.5,
            elapsed_days: 5,
            rating: 3, // Good
        };
        let result = foundation::fsrs_review(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_fsrs_review_invalid_rating() {
        let params = FsrsReviewParams {
            stability: 10.0,
            difficulty: 0.5,
            elapsed_days: 5,
            rating: 99, // Invalid
        };
        let result = foundation::fsrs_review(params);
        assert!(result.is_ok()); // Returns Ok with error message
    }

    #[test]
    fn test_fuzzy_search() {
        let params = FuzzySearchParams {
            query: "comit".to_string(),
            candidates: vec![
                "commit".to_string(),
                "comment".to_string(),
                "comet".to_string(),
            ],
            limit: 3,
        };
        let result = foundation::fuzzy_search(params);
        assert!(result.is_ok());
    }
}

// ============================================================================
// PV Signal Detection Tests
// ============================================================================

mod pv_tests {
    use super::*;

    fn test_table() -> ContingencyTableParams {
        ContingencyTableParams {
            a: 15,
            b: 85,
            c: 100,
            d: 9800,
        }
    }

    #[test]
    fn test_signal_complete() {
        let params = SignalCompleteParams {
            table: test_table(),
            prr_threshold: 2.0,
            min_n: 3,
            fdr_correction: false,
        };
        let result = pv::signal_complete(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_prr() {
        let params = SignalAlgorithmParams {
            table: test_table(),
        };
        let result = pv::signal_prr(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_ror() {
        let params = SignalAlgorithmParams {
            table: test_table(),
        };
        let result = pv::signal_ror(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_ic() {
        let params = SignalAlgorithmParams {
            table: test_table(),
        };
        let result = pv::signal_ic(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_signal_ebgm() {
        let params = SignalAlgorithmParams {
            table: test_table(),
        };
        let result = pv::signal_ebgm(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_chi_square() {
        let params = SignalAlgorithmParams {
            table: test_table(),
        };
        let result = pv::chi_square(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_naranjo_probable() {
        let params = NaranjoParams {
            temporal: 1,
            dechallenge: 1,
            rechallenge: 2,
            alternatives: 1,
            previous: 1,
        };
        let result = pv::naranjo_quick(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_who_umc_certain() {
        let params = WhoUmcParams {
            temporal: 1,
            dechallenge: 1,
            rechallenge: 1,
            alternatives: -1,
            plausibility: 1,
        };
        let result = pv::who_umc_quick(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_zero_counts() {
        let params = SignalAlgorithmParams {
            table: ContingencyTableParams {
                a: 0,
                b: 0,
                c: 0,
                d: 0,
            },
        };
        // Should handle gracefully, not panic
        let result = pv::signal_prr(params);
        assert!(result.is_ok());
    }
}

// ========================================================================
// Game Theory Tests
// ========================================================================

mod game_theory_tests {
    use super::*;

    #[test]
    fn test_nash_2x2_coordination_game() {
        let params = GameTheoryNash2x2Params {
            row_payoffs: vec![vec![2.0, 0.0], vec![0.0, 1.0]],
            col_payoffs: vec![vec![2.0, 0.0], vec![0.0, 1.0]],
        };
        let result = game_theory::nash_2x2(params);
        assert!(result.is_ok());
        let content = format!("{:?}", result.unwrap());
        assert!(content.contains("pure_equilibria"));
    }
}

// ============================================================================
// Vigilance Tests
// ============================================================================

mod vigilance_tests {
    use super::*;

    #[test]
    fn test_safety_margin() {
        let params = SafetyMarginParams {
            prr: 14.85,
            ror_lower: 9.65,
            ic025: 2.73,
            eb05: 2.63,
            n: 15,
        };
        let result = vigilance::safety_margin(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_risk_score() {
        let params = RiskScoreParams {
            drug: "TestDrug".to_string(),
            event: "TestEvent".to_string(),
            prr: 14.85,
            ror_lower: 9.65,
            ic025: 2.73,
            eb05: 2.63,
            n: 15,
        };
        let result = vigilance::risk_score(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_harm_types() {
        let result = vigilance::harm_types();
        assert!(result.is_ok());
    }

    #[test]
    fn test_map_to_tov_valid() {
        for level in 1..=8 {
            let params = MapToTovParams { level };
            let result = vigilance::map_to_tov(params);
            assert!(result.is_ok());
        }
    }

    #[test]
    fn test_map_to_tov_invalid() {
        let params = MapToTovParams { level: 99 };
        let result = vigilance::map_to_tov(params);
        assert!(result.is_ok()); // Returns Ok with error message
    }
}

// ============================================================================
// Skills Tests
// ============================================================================

mod skills_tests {
    use super::*;
    use nexcore_vigilance::skills::SkillRegistry;
    use parking_lot::RwLock;
    use std::sync::Arc;

    fn test_registry() -> Arc<RwLock<SkillRegistry>> {
        Arc::new(RwLock::new(SkillRegistry::new()))
    }

    #[test]
    fn test_skill_list_empty() {
        let registry = test_registry();
        let result = skills::list(&registry);
        assert!(result.is_ok());
    }

    #[test]
    fn test_skill_get_not_found() {
        let registry = test_registry();
        let params = SkillGetParams {
            name: "nonexistent".to_string(),
        };
        let result = skills::get(&registry, params);
        assert!(result.is_ok()); // Returns Ok with error message
    }

    #[test]
    fn test_skill_scan_invalid_dir() {
        let registry = test_registry();
        let params = SkillScanParams {
            directory: "/nonexistent/path".to_string(),
        };
        let result = skills::scan(&registry, params);
        assert!(result.is_ok()); // Returns Ok with error message
    }

    #[test]
    fn test_taxonomy_query() {
        let params = TaxonomyQueryParams {
            taxonomy_type: "compliance".to_string(),
            key: "diamond".to_string(),
        };
        let result = skills::taxonomy_query(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_taxonomy_list() {
        let params = TaxonomyListParams {
            taxonomy_type: "compliance".to_string(),
        };
        let result = skills::taxonomy_list(params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_categories_compute_intensive() {
        let result = skills::categories_compute_intensive();
        assert!(result.is_ok());
    }

    #[test]
    fn test_search_by_tag_empty_registry() {
        let registry = test_registry();
        let params = SkillSearchByTagParams {
            tag: "algorithm".to_string(),
        };
        let result = skills::search_by_tag(&registry, params);
        assert!(result.is_ok());
    }

    #[test]
    fn test_orchestration_analyze_nonexistent_path() {
        let params = SkillOrchestrationAnalyzeParams {
            path_or_pattern: "/nonexistent/skill/path".to_string(),
            include_recommendations: true,
        };
        let result = skills::orchestration_analyze(params);
        assert!(result.is_ok()); // Returns Ok with error message
        let call_result = result.expect("orchestration_analyze should return Ok");
        let content = format!("{:?}", call_result);
        assert!(content.contains("error"));
    }

    #[test]
    fn test_orchestration_analyze_tilde_expansion() {
        // Test that tilde expansion works
        let params = SkillOrchestrationAnalyzeParams {
            path_or_pattern: "~/.claude/skills/forge".to_string(),
            include_recommendations: true,
        };
        let result = skills::orchestration_analyze(params);
        // Should succeed or return structured response (not panic)
        assert!(result.is_ok());
    }

    #[test]
    fn test_orchestration_analyze_glob_pattern() {
        // Test glob pattern handling
        let params = SkillOrchestrationAnalyzeParams {
            path_or_pattern: "~/.claude/skills/*".to_string(),
            include_recommendations: false,
        };
        let result = skills::orchestration_analyze(params);
        assert!(result.is_ok());
        let call_result = result.expect("orchestration_analyze should return Ok for glob");
        let content = format!("{:?}", call_result);
        // Should contain summary section
        assert!(content.contains("summary") || content.contains("total_skills_analyzed"));
    }
}

mod principles_tests {
    use crate::params::{PrinciplesGetParams, PrinciplesListParams, PrinciplesSearchParams};
    use crate::tools::principles;

    #[test]
    fn test_principles_list() {
        let params = PrinciplesListParams {};
        let result = principles::list_principles(params);
        assert!(result.is_ok());
        let content = format!("{:?}", result.unwrap());
        assert!(content.contains("count"));
        assert!(content.contains("principles"));
    }

    #[test]
    fn test_principles_get_dalio() {
        let params = PrinciplesGetParams {
            name: "dalio-principles".to_string(),
        };
        let result = principles::get_principle(params);
        assert!(result.is_ok());
        let content = format!("{:?}", result.unwrap());
        assert!(content.contains("Dalio"));
    }

    #[test]
    fn test_principles_get_fuzzy_match() {
        let params = PrinciplesGetParams {
            name: "dalio".to_string(),
        };
        let result = principles::get_principle(params);
        assert!(result.is_ok());
        let content = format!("{:?}", result.unwrap());
        // Should fuzzy match to dalio-principles
        assert!(content.contains("dalio") || content.contains("Dalio"));
    }

    #[test]
    fn test_principles_get_not_found() {
        let params = PrinciplesGetParams {
            name: "nonexistent-principle-xyz".to_string(),
        };
        let result = principles::get_principle(params);
        assert!(result.is_ok());
        let content = format!("{:?}", result.unwrap());
        assert!(content.contains("error") || content.contains("not found"));
    }

    #[test]
    fn test_principles_search_open_minded() {
        let params = PrinciplesSearchParams {
            query: "open-minded".to_string(),
            limit: Some(5),
        };
        let result = principles::search_principles(params);
        assert!(result.is_ok());
        let content = format!("{:?}", result.unwrap());
        assert!(content.contains("query"));
        assert!(content.contains("results"));
    }

    #[test]
    fn test_principles_search_meritocracy() {
        let params = PrinciplesSearchParams {
            query: "meritocracy".to_string(),
            limit: None,
        };
        let result = principles::search_principles(params);
        assert!(result.is_ok());
        let content = format!("{:?}", result.unwrap());
        assert!(content.contains("meritocracy") || content.contains("results"));
    }
}

// ============================================================================
// GCloud Tests
// ============================================================================

mod gcloud_tests {
    use super::*;
    use crate::tools::gcloud;

    #[tokio::test]
    async fn test_gcloud_auth_list() {
        let params = GcloudConfigGetParams {
            property: "project".into(),
        };
        let json = serde_json::to_string(&params).unwrap(); // INVARIANT: test
        assert!(json.contains("project"));
    }

    #[test]
    fn test_gcloud_params_serialization() {
        let params = GcloudServiceListParams {
            project: Some("my-project".into()),
            region: Some("us-central1".into()),
        };
        let json = serde_json::to_string(&params).unwrap(); // INVARIANT: test
        assert!(json.contains("my-project"));
        assert!(json.contains("us-central1"));
    }
}

// ============================================================================
// Validation Tests
// ============================================================================

mod validation_tests {
    use super::*;
    use crate::tools::validation;

    #[test]
    fn test_validation_domains_list() {
        let params = ValidationDomainsParams {};
        let result = validation::domains(params);
        assert!(result.is_ok());
        let content = format!("{:?}", result.unwrap()); // INVARIANT: test
        assert!(content.contains("domains"));
        assert!(content.contains("L1"));
        assert!(content.contains("Coherence"));
    }

    #[test]
    fn test_validation_check_l2() {
        let params = ValidationCheckParams {
            target: "Cargo.toml".into(),
            domain: None,
        };
        let result = validation::check(params);
        assert!(result.is_ok());
    }
}
