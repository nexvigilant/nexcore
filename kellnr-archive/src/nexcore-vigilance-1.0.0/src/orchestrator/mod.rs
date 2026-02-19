//! # nexcore Orchestrator
//!
//! Agent orchestration engine for skill chain execution.
//!
//! 5-phase pipeline: Perceive → Match → Plan → Execute → Learn

#![allow(missing_docs)] // Consolidated module - docs to be added incrementally

pub mod agent;
pub mod chain_planner;
pub mod executor;
pub mod feedback;
pub mod forensics;
pub mod models;
pub mod perception;
pub mod skill_matcher;
pub mod state;

pub use agent::ChainOrchestratorAgent;
pub use chain_planner::{ChainPlanner, Preset};
pub use executor::ExecutorEngine;
pub use feedback::{FeedbackLoop, StatsSummary, Suggestion, SuggestionKind};
pub use forensics::{FailureCause, ForensicAnalyzer, ForensicReport, RecoveryAction, RecoveryStep};
pub use models::*;
pub use perception::PerceptionLayer;
pub use skill_matcher::SkillMatcher;

#[cfg(test)]
mod tests {
    use super::*;
    use std::path::Path;

    #[tokio::test]
    async fn test_orchestration_flow() {
        let skills = vec![
            Skill {
                name: "algorithm".to_string(),
                description: "Design an algorithm".to_string(),
                domain: "pharmacovigilance".to_string(),
                keywords: vec!["signal".into(), "algorithm".into()],
                triggers: vec!["build".into()],
                input_schema: serde_json::json!({}),
                output_schema: serde_json::json!({}),
            },
            Skill {
                name: "test-runner".to_string(),
                description: "Run tests".to_string(),
                domain: "pharmacovigilance".to_string(),
                keywords: vec!["test".into()],
                triggers: vec!["test".into()],
                input_schema: serde_json::json!({}),
                output_schema: serde_json::json!({}),
            },
        ];

        let mut agent = ChainOrchestratorAgent::new(skills, Path::new("/tmp/metrics"));
        let result = agent
            .process("Build a signal detection algorithm and test it")
            .await
            .unwrap();

        assert_eq!(result.status, ExecutionStatus::Completed);
        assert!(!result.skill_results.is_empty());
    }

    #[test]
    fn test_perception_layer() {
        let perception = PerceptionLayer::new();
        let analysis = perception.analyze("Build a signal detection algorithm");
        assert_eq!(analysis.intent, "build");
        assert_eq!(analysis.domain, "pharmacovigilance");
    }

    #[test]
    fn test_complexity_estimation() {
        let perception = PerceptionLayer::new();
        let simple = perception.analyze("just run a quick test");
        assert_eq!(simple.complexity, Complexity::Simple);
        let complex = perception.analyze("build a comprehensive full system and then deploy");
        assert_eq!(complex.complexity, Complexity::Complex);
    }

    #[test]
    fn test_execution_context() {
        let mut ctx = ExecutionContext::new();
        assert!(ctx.data.is_empty());

        ctx.merge(&serde_json::json!({"key": "value", "count": 42}));
        assert_eq!(ctx.data.len(), 2);
        assert!(ctx.data.get("key").is_some_and(|v| v == "value"));

        ctx.add_artifact("output.txt".to_string());
        ctx.add_message("Step completed".to_string());
        assert_eq!(ctx.artifacts.len(), 1);
        assert_eq!(ctx.messages.len(), 1);

        let formatted = ctx.format_for_skill();
        assert!(formatted.contains("Prior Context"));
        assert!(formatted.contains("key"));
    }

    #[test]
    fn test_parse_expression() {
        let planner = ChainPlanner::new();
        let skill_lookup = std::collections::HashMap::new();

        // Sequential chain
        let chain = planner.parse_expression("algorithm → test → deploy", &skill_lookup);
        assert_eq!(chain.nodes.len(), 3);
        assert_eq!(chain.nodes[0].skill_name, "algorithm");
        assert_eq!(chain.nodes[0].operator, ChainOperator::Sequential);
        assert_eq!(chain.nodes[2].operator, ChainOperator::End);

        // Parallel chain
        let chain = planner.parse_expression("lint && typecheck && test", &skill_lookup);
        assert_eq!(chain.nodes.len(), 3);
        assert_eq!(chain.nodes[0].operator, ChainOperator::Parallel);
        assert_eq!(chain.nodes[1].operator, ChainOperator::Parallel);

        // Normalized arrow
        let chain = planner.parse_expression("a -> b", &skill_lookup);
        assert_eq!(chain.nodes.len(), 2);
        assert_eq!(chain.nodes[0].operator, ChainOperator::Sequential);
    }

    #[test]
    fn test_suggest_chain() {
        let skills = vec![
            ScoredSkill {
                skill: Skill {
                    name: "algorithm-designer".to_string(),
                    description: "Design algorithms".to_string(),
                    domain: "general".to_string(),
                    keywords: vec!["algorithm".into(), "design".into()],
                    triggers: vec![],
                    input_schema: serde_json::json!({}),
                    output_schema: serde_json::json!({}),
                },
                score: 0.9,
                match_reasons: vec![],
            },
            ScoredSkill {
                skill: Skill {
                    name: "test-runner".to_string(),
                    description: "Run tests".to_string(),
                    domain: "general".to_string(),
                    keywords: vec!["test".into(), "validate".into()],
                    triggers: vec![],
                    input_schema: serde_json::json!({}),
                    output_schema: serde_json::json!({}),
                },
                score: 0.8,
                match_reasons: vec![],
            },
        ];

        let planner = ChainPlanner::new();
        let suggestion = planner.suggest_chain(&skills);
        assert!(suggestion.contains("→"));
        assert!(suggestion.contains("algorithm-designer"));
    }

    #[test]
    fn test_preset_support() {
        let planner = ChainPlanner::new();

        // Test is_preset detection
        assert!(ChainPlanner::is_preset("@implement"));
        assert!(ChainPlanner::is_preset("@research"));
        assert!(!ChainPlanner::is_preset("implement"));
        assert!(!ChainPlanner::is_preset("algorithm"));

        // Test preset retrieval - implement is a default preset
        let preset = planner.get_preset("@implement");
        assert!(preset.is_some());
        let preset = preset.unwrap(); // INVARIANT: checked above
        assert_eq!(preset.name, "implement");
        assert!(preset.chain.contains("algorithm"));

        // Test preset without @ prefix - quality is a default preset
        let preset = planner.get_preset("quality");
        assert!(preset.is_some());
        let preset = preset.unwrap(); // INVARIANT: checked above
        assert_eq!(preset.name, "quality");
        assert!(preset.description.contains("QA"));

        // Test preset names
        let names = planner.preset_names();
        assert!(names.contains(&"research"));
        assert!(names.contains(&"implement"));
        assert!(names.contains(&"quality"));
        assert!(names.contains(&"secure"));
        assert!(names.contains(&"ship"));
    }

    #[test]
    fn test_expand_preset() {
        let planner = ChainPlanner::new();
        let skill_lookup = std::collections::HashMap::new();

        // Expand @implement preset - it's a default preset
        let chain_opt = planner.expand_preset("@implement", &skill_lookup);
        assert!(chain_opt.is_some());
        let chain = chain_opt.unwrap(); // INVARIANT: checked above
        assert_eq!(chain.preset_name, Some("implement".to_string()));
        assert!(!chain.nodes.is_empty());
        assert!(chain.nodes.iter().any(|n| n.skill_name == "algorithm"));
        assert!(chain.nodes.iter().any(|n| n.skill_name == "proceed-lite"));

        // Expand @quality preset (has parallel skills) - it's a default preset
        let chain_opt = planner.expand_preset("quality", &skill_lookup);
        assert!(chain_opt.is_some());
        let chain = chain_opt.unwrap(); // INVARIANT: checked above
        assert!(chain.nodes.iter().any(|n| n.skill_name == "lint"));
        assert!(
            chain
                .nodes
                .iter()
                .any(|n| n.operator == ChainOperator::Parallel)
        );

        // Non-existent preset returns None
        assert!(
            planner
                .expand_preset("@nonexistent", &skill_lookup)
                .is_none()
        );
    }

    #[test]
    fn test_feedback_persistence() {
        use tempfile::tempdir;

        // Create temp dir - if this fails, the test should fail
        let temp_dir = tempdir();
        assert!(temp_dir.is_ok());
        let temp_dir = temp_dir.unwrap(); // INVARIANT: checked above
        let metrics_path = temp_dir.path();

        // Create feedback loop - should initialize empty
        let mut feedback = FeedbackLoop::new(metrics_path);
        assert!(feedback.skill_success_rates.is_empty());
        assert!(feedback.chain_rankings.is_empty());

        // Create a mock chain and result
        let chain = Chain {
            nodes: vec![
                ChainNode {
                    skill_name: "algorithm".to_string(),
                    operator: ChainOperator::Sequential,
                    level: 0,
                    dependencies: vec![],
                },
                ChainNode {
                    skill_name: "proceed-lite".to_string(),
                    operator: ChainOperator::End,
                    level: 1,
                    dependencies: vec!["algorithm".to_string()],
                },
            ],
            analysis: None,
            confidence: 0.9,
            preset_name: None,
            safety_manifold: None,
        };

        let result = ExecutionResult {
            chain: chain.clone(),
            status: ExecutionStatus::Completed,
            skill_results: vec![
                SkillResult {
                    skill_name: "algorithm".to_string(),
                    status: ExecutionStatus::Completed,
                    signal: AndonSignal::Green,
                    output: serde_json::json!({}),
                    artifacts: vec![],
                    error: None,
                    duration_ms: 100,
                },
                SkillResult {
                    skill_name: "proceed-lite".to_string(),
                    status: ExecutionStatus::Completed,
                    signal: AndonSignal::Green,
                    output: serde_json::json!({}),
                    artifacts: vec![],
                    error: None,
                    duration_ms: 200,
                },
            ],
            total_duration_seconds: 0.3,
            halt_reason: None,
            context_accumulated: None,
        };

        // Record execution
        feedback.record(&chain, &result);

        // Verify in-memory stats updated
        assert!(feedback.skill_success_rates.contains_key("algorithm"));
        assert!(feedback.skill_success_rates.contains_key("proceed-lite"));

        // Verify files created
        assert!(metrics_path.join("chain_metrics.jsonl").exists());
        assert!(metrics_path.join("skill_stats.json").exists());

        // Create new feedback loop - should load persisted stats
        let feedback2 = FeedbackLoop::new(metrics_path);
        assert!(feedback2.skill_success_rates.contains_key("algorithm"));
        assert!(feedback2.skill_success_rates.contains_key("proceed-lite"));

        // Test metrics history
        let history = feedback2.read_metrics_history(10);
        assert_eq!(history.len(), 1);
        assert!(history[0].success);

        // Test stats summary
        let summary = feedback2.get_stats_summary();
        assert_eq!(summary.total_skills_tracked, 2);
        assert_eq!(summary.total_chains_tracked, 1);
    }
}
