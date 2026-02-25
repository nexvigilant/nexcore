#![forbid(unsafe_code)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

//! # NexVigilant Core — Skills Engine
//!
//! Skill registry, routing, validation, and code generation.
//!
//! ## Modules
//!
//! - **registry** - Skill discovery and registration
//! - **routing** - Multi-strategy skill routing
//! - **validation** - Diamond compliance validation
//! - **codegen** - SMST to code generation
//! - **taxonomy** - Compile-time lookup tables (O(1) access)
//! - **hooks** - File organization and validation hooks
//! - **sqi** - Skill Quality Index (chemistry-derived scoring)
//! - **foundation** - Inlined foundation utilities (metadata parsing, fuzzy search, SMST extraction)

pub mod foundation;
pub mod grounding;

pub mod algorithm;
pub mod assist_index;
pub mod builder;
pub mod codegen;
pub mod dtree_router;
pub mod hooks;
pub mod ksb_verify;
// maturation module omitted — was undeclared in original (dead code)
pub mod registry;
pub mod routing;
pub mod smst_v2;
pub mod sqi;
pub mod taxonomy;
pub mod validation;

pub use assist_index::{
    SkillKnowledgeEntry, SkillKnowledgeIndex, SkillSearchResult, default_skills_path, search_skills,
};
pub use builder::{
    BatchSummary, BuildError, BuildOptions, BuildReport, StructureCheck, build_skill,
    build_skills_batch, summarize_batch as summarize_build_batch,
};
pub use ksb_verify::{
    CheckResult, KsbBatchSummary, KsbError, KsbValidation, summarize_batch, verify_ksb,
    verify_ksb_batch,
};
pub use registry::{SkillInfo, SkillRegistry, default_skills_cache_path};
pub use routing::RoutingEngine;
pub use smst_v2::{ComponentScores, SmstV2Error, SmstV2Result, extract_smst_v2, validate_smst_v2};
pub use sqi::{
    DimensionScore, EcosystemSqiResult, SqiDimension, SqiError, SqiGrade, SqiResult,
    compute_ecosystem_sqi, compute_sqi, sensitivity_analysis,
};
pub use taxonomy::{
    COMPLIANCE_LEVELS, ComplianceLevel, NODE_TYPES, NodeType, SKILL_CATEGORIES, SMST_COMPONENTS,
    SkillCategory, SmstComponent, TaxonomyListResult, TaxonomyQueryResult, all_compliance_levels,
    all_skill_categories, all_smst_components, compute_intensive_categories, list_taxonomy,
    lookup_compliance_level, lookup_node_type, lookup_skill_category, lookup_smst_component,
    query_taxonomy, required_smst_components,
};
pub use validation::{DiamondValidation, validate_diamond};
