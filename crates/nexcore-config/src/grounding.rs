//! # GroundsTo implementations for nexcore-config types
//!
//! Primitive grounding for configuration consolidation types: ClaudeConfig,
//! GeminiConfig, GitConfig, HookRegistry, VocabSkillMap, AsrConfig, and
//! all supporting structs and enums.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::asr::{
    AsrConfig, DoDChecklist, DoDItem, FlywheelConfig, FlywheelStage, Model, RoutingConfig,
};
use crate::claude::{
    ClaudeConfig, FeatureFlags, McpServerConfig, ModelUsage, OAuthAccount, PerformanceStats,
    ProjectConfig, SkillUsageStats, VulnerabilityCache,
};
use crate::gemini::GeminiConfig;
use crate::gemini::HookType as GeminiHookType;
use crate::git::{
    CredentialHelper, GitConfig, GitCore, GitDiff, GitFetch, GitInit, GitPull, GitUser,
};
use crate::hooks::{HookEvent, HookMeta, HookRegistry, HookTier};
use crate::vocab::{SkillChain, SkillMapping, VocabSkillMap};
use crate::AllConfigs;

// ============================================================================
// AllConfigs: T3 (ς + × + π + μ + σ + ∃), dominant ×
// Top-level consolidation product — aggregates all config sources.
// ============================================================================

impl GroundsTo for AllConfigs {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Product,
            LexPrimitiva::State,
            LexPrimitiva::Persistence,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Existence,
        ])
        .with_dominant(LexPrimitiva::Product, 0.70)
    }
}

// ============================================================================
// ClaudeConfig: T3 (ς + μ + × + π + ∃ + N), dominant ς
// Rich stateful config with maps, nested structs, persistence.
// ============================================================================

impl GroundsTo for ClaudeConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
            LexPrimitiva::Product,
            LexPrimitiva::Persistence,
            LexPrimitiva::Existence,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::State, 0.70)
    }
}

// ============================================================================
// ProjectConfig: T2-C (ς + μ + σ + ×), dominant ς
// Per-project config state with collections and nested maps.
// ============================================================================

impl GroundsTo for ProjectConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::Product,
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ============================================================================
// McpServerConfig: T2-P (Σ + ς + σ), dominant Σ
// Tagged enum (Stdio variant) — sum type for server transport.
// ============================================================================

impl GroundsTo for McpServerConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ============================================================================
// FeatureFlags: T2-P (κ + ς + μ), dominant κ
// Boolean feature gates — comparison predicates with state.
// ============================================================================

impl GroundsTo for FeatureFlags {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::State,
            LexPrimitiva::Mapping,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ============================================================================
// PerformanceStats: T2-C (N + ς + × + π), dominant N
// Numeric measurements with state and product structure.
// ============================================================================

impl GroundsTo for PerformanceStats {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity,
            LexPrimitiva::State,
            LexPrimitiva::Product,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.80)
    }
}

// ============================================================================
// ModelUsage: T2-P (N + ×), dominant N
// Numeric counters — token and cost measurements.
// ============================================================================

impl GroundsTo for ModelUsage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Product])
            .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

// ============================================================================
// VulnerabilityCache: T2-C (κ + ∅ + ς + π), dominant κ
// Detection state with optional fields and persistence.
// ============================================================================

impl GroundsTo for VulnerabilityCache {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Comparison,
            LexPrimitiva::Void,
            LexPrimitiva::State,
            LexPrimitiva::Persistence,
        ])
        .with_dominant(LexPrimitiva::Comparison, 0.75)
    }
}

// ============================================================================
// SkillUsageStats: T2-P (N + ν), dominant N
// Usage count and last-used timestamp — quantity with frequency.
// ============================================================================

impl GroundsTo for SkillUsageStats {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Quantity, LexPrimitiva::Frequency])
            .with_dominant(LexPrimitiva::Quantity, 0.85)
    }
}

// ============================================================================
// OAuthAccount: T2-P (ς + ×), dominant ς
// Identity state — product of credential fields.
// ============================================================================

impl GroundsTo for OAuthAccount {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Product])
            .with_dominant(LexPrimitiva::State, 0.90)
    }
}

// ============================================================================
// GeminiConfig: T2-P (ς + σ), dominant ς
// Config state with a sequence of hook definitions.
// ============================================================================

impl GroundsTo for GeminiConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ============================================================================
// GeminiHookType: T1 (Σ), dominant Σ
// Pure three-variant enum — run/validation/post-process.
// ============================================================================

impl GroundsTo for GeminiHookType {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

// ============================================================================
// GitConfig: T2-C (ς + × + μ + σ), dominant ς
// Composite configuration state with maps and nested products.
// ============================================================================

impl GroundsTo for GitConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Product,
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ============================================================================
// GitUser: T2-P (ς + ×), dominant ς
// Simple identity state — name + email product.
// ============================================================================

impl GroundsTo for GitUser {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Product])
            .with_dominant(LexPrimitiva::State, 0.90)
    }
}

// ============================================================================
// GitInit: T1 (ς), dominant ς
// Single-field state — default branch name.
// ============================================================================

impl GroundsTo for GitInit {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State]).with_dominant(LexPrimitiva::State, 1.0)
    }
}

// ============================================================================
// GitPull: T2-P (ς + κ), dominant ς
// Boolean config state — rebase yes/no comparison.
// ============================================================================

impl GroundsTo for GitPull {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ============================================================================
// GitFetch: T2-P (ς + κ), dominant ς
// Boolean config state — prune yes/no comparison.
// ============================================================================

impl GroundsTo for GitFetch {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ============================================================================
// GitDiff: T2-P (ς + ∅), dominant ς
// Optional config state — color setting may be absent.
// ============================================================================

impl GroundsTo for GitDiff {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Void])
            .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ============================================================================
// GitCore: T2-P (ς + ∅), dominant ς
// Optional config state — editor and autocrlf may be absent.
// ============================================================================

impl GroundsTo for GitCore {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Void])
            .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ============================================================================
// CredentialHelper: T2-P (λ + ×), dominant λ
// URL pattern + command path — location-based credential routing.
// ============================================================================

impl GroundsTo for CredentialHelper {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Location, LexPrimitiva::Product])
            .with_dominant(LexPrimitiva::Location, 0.85)
    }
}

// ============================================================================
// HookMeta: T2-P (ς + μ), dominant ς
// Registry metadata — version state with tier description mapping.
// ============================================================================

impl GroundsTo for HookMeta {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::State, LexPrimitiva::Mapping])
            .with_dominant(LexPrimitiva::State, 0.85)
    }
}

// ============================================================================
// HookTier: T1 (Σ), dominant Σ
// Pure three-variant enum — Dev/Review/Deploy.
// ============================================================================

impl GroundsTo for HookTier {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

// ============================================================================
// HookEvent: T1 (Σ), dominant Σ
// Pure multi-variant enum — event types for hook dispatch.
// ============================================================================

impl GroundsTo for HookEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

// ============================================================================
// HookRegistry: T2-C (μ + σ + ς + Σ), dominant μ
// Mapping from events to hooks — registry with filtering and generation.
// ============================================================================

impl GroundsTo for HookRegistry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::Sequence,
            LexPrimitiva::State,
            LexPrimitiva::Sum,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ============================================================================
// VocabSkillMap: T2-C (μ + ς + σ + ×), dominant μ
// Mapping structure — vocabulary to skills with chain sequences.
// ============================================================================

impl GroundsTo for VocabSkillMap {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
            LexPrimitiva::Product,
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

// ============================================================================
// SkillMapping: T2-P (μ + σ), dominant μ
// Maps a vocabulary term to skills — primary + secondary sequence.
// ============================================================================

impl GroundsTo for SkillMapping {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Mapping, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ============================================================================
// SkillChain: T2-P (σ + →), dominant σ
// Ordered skill invocation chain with causal trigger.
// ============================================================================

impl GroundsTo for SkillChain {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Causality])
            .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

// ============================================================================
// AsrConfig: T2-C (ς + × + μ + N), dominant ς
// Composite configuration — routing, flywheel, and DoD sub-configs.
// ============================================================================

impl GroundsTo for AsrConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Product,
            LexPrimitiva::Mapping,
            LexPrimitiva::Quantity,
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ============================================================================
// RoutingConfig: T2-C (ς + κ + N + λ), dominant ς
// Routing state with confidence threshold, model selection, and log path.
// ============================================================================

impl GroundsTo for RoutingConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Comparison,
            LexPrimitiva::Quantity,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::State, 0.75)
    }
}

// ============================================================================
// FlywheelConfig: T2-C (ς + σ + N + λ), dominant ς
// Cycle configuration — current stage state with numeric durations.
// ============================================================================

impl GroundsTo for FlywheelConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,
            LexPrimitiva::Sequence,
            LexPrimitiva::Quantity,
            LexPrimitiva::Location,
        ])
        .with_dominant(LexPrimitiva::State, 0.75)
    }
}

// ============================================================================
// FlywheelStage: T2-P (Σ + σ), dominant Σ
// Cyclic three-variant enum with next() progression — sum with sequence.
// ============================================================================

impl GroundsTo for FlywheelStage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum, LexPrimitiva::Sequence])
            .with_dominant(LexPrimitiva::Sum, 0.85)
    }
}

// ============================================================================
// DoDItem: T2-P (κ + ς), dominant κ
// Checklist item — boolean completion check with state.
// ============================================================================

impl GroundsTo for DoDItem {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Comparison, LexPrimitiva::State])
            .with_dominant(LexPrimitiva::Comparison, 0.85)
    }
}

// ============================================================================
// DoDChecklist: T2-P (σ + κ), dominant σ
// Ordered sequence of checklist items with aggregate completion check.
// ============================================================================

impl GroundsTo for DoDChecklist {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sequence, LexPrimitiva::Comparison])
            .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

// ============================================================================
// Model: T1 (Σ), dominant Σ
// Pure three-variant enum — Haiku/Sonnet/Opus.
// ============================================================================

impl GroundsTo for Model {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![LexPrimitiva::Sum]).with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    // -- AllConfigs --

    #[test]
    fn all_configs_is_product_dominant_t3() {
        let comp = AllConfigs::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Product));
        assert_eq!(comp.unique().len(), 6);
        assert_eq!(AllConfigs::tier(), Tier::T3DomainSpecific);
    }

    // -- Claude types --

    #[test]
    fn claude_config_is_state_dominant() {
        let comp = ClaudeConfig::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(ClaudeConfig::tier(), Tier::T3DomainSpecific);
    }

    #[test]
    fn project_config_is_state_dominant() {
        let comp = ProjectConfig::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(ProjectConfig::tier(), Tier::T2Composite);
    }

    #[test]
    fn mcp_server_config_is_sum_dominant() {
        let comp = McpServerConfig::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(McpServerConfig::tier(), Tier::T2Primitive);
    }

    #[test]
    fn feature_flags_is_comparison_dominant() {
        let comp = FeatureFlags::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert_eq!(FeatureFlags::tier(), Tier::T2Primitive);
    }

    #[test]
    fn performance_stats_is_quantity_dominant() {
        let comp = PerformanceStats::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(PerformanceStats::tier(), Tier::T2Composite);
    }

    #[test]
    fn model_usage_is_quantity_dominant() {
        let comp = ModelUsage::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(ModelUsage::tier(), Tier::T2Primitive);
    }

    #[test]
    fn vulnerability_cache_is_comparison_dominant() {
        let comp = VulnerabilityCache::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert_eq!(VulnerabilityCache::tier(), Tier::T2Composite);
    }

    #[test]
    fn skill_usage_stats_is_quantity_dominant() {
        let comp = SkillUsageStats::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Quantity));
        assert_eq!(SkillUsageStats::tier(), Tier::T2Primitive);
    }

    #[test]
    fn oauth_account_is_state_dominant() {
        let comp = OAuthAccount::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(OAuthAccount::tier(), Tier::T2Primitive);
    }

    // -- Gemini types --

    #[test]
    fn gemini_config_is_state_dominant() {
        let comp = GeminiConfig::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(GeminiConfig::tier(), Tier::T2Primitive);
    }

    #[test]
    fn gemini_hook_type_is_pure_sum() {
        let comp = GeminiHookType::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert!(comp.is_pure());
        assert_eq!(GeminiHookType::tier(), Tier::T1Universal);
    }

    // -- Git types --

    #[test]
    fn git_config_is_state_dominant() {
        let comp = GitConfig::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(GitConfig::tier(), Tier::T2Composite);
    }

    #[test]
    fn git_user_is_state_dominant() {
        let comp = GitUser::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(GitUser::tier(), Tier::T2Primitive);
    }

    #[test]
    fn git_init_is_pure_state() {
        let comp = GitInit::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert!(comp.is_pure());
        assert_eq!(GitInit::tier(), Tier::T1Universal);
    }

    #[test]
    fn credential_helper_is_location_dominant() {
        let comp = CredentialHelper::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Location));
        assert_eq!(CredentialHelper::tier(), Tier::T2Primitive);
    }

    // -- Hook types --

    #[test]
    fn hook_tier_is_pure_sum() {
        let comp = HookTier::primitive_composition();
        assert!(comp.is_pure());
        assert_eq!(HookTier::tier(), Tier::T1Universal);
    }

    #[test]
    fn hook_event_is_pure_sum() {
        let comp = HookEvent::primitive_composition();
        assert!(comp.is_pure());
        assert_eq!(HookEvent::tier(), Tier::T1Universal);
    }

    #[test]
    fn hook_registry_is_mapping_dominant() {
        let comp = HookRegistry::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(HookRegistry::tier(), Tier::T2Composite);
    }

    // -- Vocab types --

    #[test]
    fn vocab_skill_map_is_mapping_dominant() {
        let comp = VocabSkillMap::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(VocabSkillMap::tier(), Tier::T2Composite);
    }

    #[test]
    fn skill_mapping_is_mapping_dominant() {
        let comp = SkillMapping::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Mapping));
        assert_eq!(SkillMapping::tier(), Tier::T2Primitive);
    }

    #[test]
    fn skill_chain_is_sequence_dominant() {
        let comp = SkillChain::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert_eq!(SkillChain::tier(), Tier::T2Primitive);
    }

    // -- ASR types --

    #[test]
    fn asr_config_is_state_dominant() {
        let comp = AsrConfig::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(AsrConfig::tier(), Tier::T2Composite);
    }

    #[test]
    fn routing_config_is_state_dominant() {
        let comp = RoutingConfig::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::State));
        assert_eq!(RoutingConfig::tier(), Tier::T2Composite);
    }

    #[test]
    fn flywheel_stage_is_sum_dominant() {
        let comp = FlywheelStage::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sum));
        assert_eq!(FlywheelStage::tier(), Tier::T2Primitive);
    }

    #[test]
    fn dod_item_is_comparison_dominant() {
        let comp = DoDItem::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Comparison));
        assert_eq!(DoDItem::tier(), Tier::T2Primitive);
    }

    #[test]
    fn dod_checklist_is_sequence_dominant() {
        let comp = DoDChecklist::primitive_composition();
        assert_eq!(comp.dominant, Some(LexPrimitiva::Sequence));
        assert_eq!(DoDChecklist::tier(), Tier::T2Primitive);
    }

    #[test]
    fn model_is_pure_sum() {
        let comp = Model::primitive_composition();
        assert!(comp.is_pure());
        assert_eq!(Model::tier(), Tier::T1Universal);
    }

    // -- Aggregate checks --

    #[test]
    fn all_types_have_dominant_primitive() {
        assert!(AllConfigs::dominant_primitive().is_some());
        assert!(ClaudeConfig::dominant_primitive().is_some());
        assert!(ProjectConfig::dominant_primitive().is_some());
        assert!(McpServerConfig::dominant_primitive().is_some());
        assert!(FeatureFlags::dominant_primitive().is_some());
        assert!(PerformanceStats::dominant_primitive().is_some());
        assert!(ModelUsage::dominant_primitive().is_some());
        assert!(VulnerabilityCache::dominant_primitive().is_some());
        assert!(SkillUsageStats::dominant_primitive().is_some());
        assert!(OAuthAccount::dominant_primitive().is_some());
        assert!(GeminiConfig::dominant_primitive().is_some());
        assert!(GeminiHookType::dominant_primitive().is_some());
        assert!(GitConfig::dominant_primitive().is_some());
        assert!(GitUser::dominant_primitive().is_some());
        assert!(GitInit::dominant_primitive().is_some());
        assert!(GitPull::dominant_primitive().is_some());
        assert!(GitFetch::dominant_primitive().is_some());
        assert!(GitDiff::dominant_primitive().is_some());
        assert!(GitCore::dominant_primitive().is_some());
        assert!(CredentialHelper::dominant_primitive().is_some());
        assert!(HookMeta::dominant_primitive().is_some());
        assert!(HookTier::dominant_primitive().is_some());
        assert!(HookEvent::dominant_primitive().is_some());
        assert!(HookRegistry::dominant_primitive().is_some());
        assert!(VocabSkillMap::dominant_primitive().is_some());
        assert!(SkillMapping::dominant_primitive().is_some());
        assert!(SkillChain::dominant_primitive().is_some());
        assert!(AsrConfig::dominant_primitive().is_some());
        assert!(RoutingConfig::dominant_primitive().is_some());
        assert!(FlywheelConfig::dominant_primitive().is_some());
        assert!(FlywheelStage::dominant_primitive().is_some());
        assert!(DoDItem::dominant_primitive().is_some());
        assert!(DoDChecklist::dominant_primitive().is_some());
        assert!(Model::dominant_primitive().is_some());
    }

    #[test]
    fn pure_sum_types_are_t1() {
        assert_eq!(HookTier::tier(), Tier::T1Universal);
        assert_eq!(HookEvent::tier(), Tier::T1Universal);
        assert_eq!(GeminiHookType::tier(), Tier::T1Universal);
        assert_eq!(Model::tier(), Tier::T1Universal);
        assert_eq!(GitInit::tier(), Tier::T1Universal);
    }
}
