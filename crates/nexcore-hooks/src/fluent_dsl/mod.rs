//! Fluent DSL for declarative hook rule building.
//!
//! Ported from claude-hooks library. Provides type-safe, ergonomic rule construction.
//!
//! # Overview
//!
//! This module provides two complementary APIs:
//!
//! | API | Use Case |
//! |-----|----------|
//! | [`Dsl`] | Fluent builder for individual rules |
//! | [`HookChain`] | Chain multiple validators with aggregation strategies |
//!
//! # Fluent DSL Example
//!
//! ```rust,ignore
//! use nexcore_hooks::fluent_dsl::{Dsl, DslEvaluator};
//!
//! let rules = Dsl::new()
//!     .when_tool("Bash")
//!         .command_contains("rm -rf /")
//!         .deny("Dangerous rm command blocked")
//!     .when_tool("Write|Edit")
//!         .path_matches(r"\.env$")
//!         .deny("Environment files are protected")
//!     .when_tool("mcp__*__delete_*")
//!         .ask("Confirm MCP deletion?")
//!     .build();
//!
//! let evaluator = DslEvaluator::new(rules);
//! let input = serde_json::json!({ "command": "rm -rf /" });
//! if let Some((action, msg)) = evaluator.evaluate("Bash", &input) {
//!     println!("Action: {:?}, Message: {}", action, msg);
//! }
//! ```
//!
//! # Hook Chaining Example
//!
//! ```rust,ignore
//! use nexcore_hooks::fluent_dsl::{HookChain, AggregationStrategy, RuleSet};
//!
//! let chain = HookChain::new()
//!     .with_strategy(AggregationStrategy::FirstBlock)
//!     .add_rules("security", security_rules)
//!     .add_rules("project", project_rules);
//!
//! let result = chain.check_bash("dangerous command");
//! if result.is_blocked() {
//!     eprintln!("Blocked by: {}", result.matches[0].validator_name);
//! }
//! ```
//!
//! # Aggregation Strategies
//!
//! | Strategy | Behavior |
//! |----------|----------|
//! | `FirstBlock` | Stop at first blocking rule (fail-fast) |
//! | `CollectAll` | Run all validators, collect all matches |
//! | `AllMustPass` | All validators must pass (strictest) |
//! | `AnyAllow` | First allow rule permits (most permissive) |
//!
//! # Pattern Syntax
//!
//! Tool patterns support glob and OR syntax:
//! - `"Bash"` - exact match
//! - `"Write|Edit"` - OR pattern
//! - `"mcp__*__delete_*"` - glob pattern
//!
//! # Primitive Foundation
//!
//! Built from T1/T2 primitives with high transfer confidence:
//! - Builder pattern (T2): 0.91 confidence
//! - Strategy pattern (T2): 0.90 confidence
//! - Pattern matching (T2): 0.94 confidence

mod compose;
mod rules;

pub use compose::{AggregationStrategy, ChainOutput, ChainResult, ChainedValidator, HookChain};
pub use rules::{
    CompiledRule, Dsl, DslEvaluator, Rule, RuleAction, RuleBuilder, RuleCategory, RuleMatch,
    RuleSet, RulesEngine,
};
