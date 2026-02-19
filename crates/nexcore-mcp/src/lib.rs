#![recursion_limit = "512"]
//! # NexVigilant Core MCP Server
//!
//! MCP server exposing NexVigilant Core's high-performance Rust APIs to Claude Code.
//!
//! ## Features
//!
//! - **262 tools** across 11+ domains: Foundation, PV, Vigilance, HUD, Skills, Guidelines, FAERS, GCloud, Wolfram, Principles, Brain, Guardian, Chemistry, STEM, Regulatory, Hooks, Vigil
//! - **Direct library integration** - no subprocess overhead
//! - **Thread-safe state** - SkillRegistry with parking_lot RwLock
//!
//! ## Tool Categories
//!
//! | Category | Tools | Description |
//! |----------|-------|-------------|
//! | Foundation | 7 | Algorithms, crypto, YAML, graph, FSRS |
//! | PV | 8 | Signal detection, causality assessment |
//! | Vigilance | 4 | Safety margin, risk, ToV, harm types |
//! | HUD | 24 | CAP-014/018/019/020/022/025/026/027/028/029/030/031/037 governance capabilities |
//! | Skills | 8 | Registry, validation, taxonomy |
//! | Guidelines | 9 | ICH, CIOMS, EMA GVP guideline search + ICH glossary (894+ terms) |
//! | FAERS | 5 | FDA Adverse Event queries via OpenFDA |
//! | GCloud | 19 | Google Cloud CLI operations |
//! | Principles | 3 | Knowledge base search (Dalio, KISS, etc.) |

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(missing_docs)]
// Style issues in newly migrated code - fix in dedicated cleanup pass
#![allow(clippy::collapsible_if)]
#![allow(clippy::redundant_closure)]
#![allow(clippy::redundant_closure_for_method_calls)]

pub mod browser;
pub mod composites;
pub mod config;
pub mod grounding;
pub mod params;
pub mod prelude;
#[allow(missing_docs)]
pub mod tooling;
pub mod tools;
pub mod transfer;
/// Unified dispatcher for single-tool mode.
pub mod unified;

/// MCP call telemetry collection (optional feature).
#[cfg(feature = "telemetry")]
pub mod telemetry;

#[cfg(test)]
mod tests;

use std::sync::Arc;

use nexcore_vigilance::skills::{SkillKnowledgeIndex, SkillRegistry};
use parking_lot::RwLock;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, ListToolsResult, PaginatedRequestParams,
    ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_router};

/// NexCore MCP Server
///
/// Exposes NexCore APIs via Model Context Protocol for Claude Code integration.
///
/// Tier: T3 (Domain-specific MCP server)
/// Grounds to T1 Concepts via Arc/RwLock/Option and tool router state
/// Ord: N/A (stateful service)
#[derive(Clone)]
pub struct NexCoreMcpServer {
    /// Skill registry (thread-safe)
    pub registry: Arc<RwLock<SkillRegistry>>,
    /// Skill knowledge index for intent-based search (thread-safe)
    pub assist_index: Arc<RwLock<SkillKnowledgeIndex>>,
    /// Loaded configuration (optional - may not exist)
    pub config: Option<nexcore_config::ClaudeConfig>,
    /// Tool router
    pub tool_router: ToolRouter<Self>,
}

#[tool_router]
impl NexCoreMcpServer {
    /// Create a new NexCore MCP server
    #[must_use]
    pub fn new() -> Self {
        // Load config with fallback (don't fail server if config missing)
        let config = config::load_config().ok();

        // Pre-populate skill knowledge index from ~/.claude/skills/
        let assist_index = nexcore_vigilance::skills::default_skills_path()
            .and_then(|p| SkillKnowledgeIndex::scan(&p).ok())
            .unwrap_or_default();

        Self {
            registry: Arc::new(RwLock::new(SkillRegistry::new())),
            assist_index: Arc::new(RwLock::new(assist_index)),
            config,
            tool_router: Self::tool_router(),
        }
    }

    // ========================================================================
    // Unified Dispatcher (1)
    // ========================================================================

    #[tool(
        description = "nexcore unified interface. 380+ commands via {command, params}. help=catalog, toolbox=search."
    )]
    async fn nexcore(
        &self,
        Parameters(uparams): Parameters<params::system::UnifiedParams>,
    ) -> Result<CallToolResult, McpError> {
        if let Err(err) = crate::tooling::tool_gate().check(&uparams.command) {
            return Ok(crate::tooling::gated_result(&uparams.command, err));
        }
        let result = crate::unified::dispatch(&uparams.command, uparams.params, self).await?;
        Ok(crate::tooling::wrap_result(result))
    }

    // ========================================================================
    // System Tools (4)
    // ========================================================================

    /*
    #[tool(
        description = "Health check for NexCore MCP server. Returns version, tool count, and status."
    )]
    async fn nexcore_health(&self) -> Result<CallToolResult, McpError> {
        let tool_count = self.tool_router.list_all().len();
        let health =
            serde_json::json!({
            "status": "healthy",
            "server": "nexcore-mcp",
            "version": env!("CARGO_PKG_VERSION"),
            "tool_count": tool_count,
            "domains": ["foundation", "pv", "vigilance", "skills", "guidelines", "faers", "gcloud", "wolfram", "principles", "hooks"]
        });
        Ok(
            CallToolResult::success(
                vec![
                    rmcp::model::Content::text(
                        serde_json::to_string_pretty(&health).unwrap_or_default()
                    )
                ]
            )
        )
    }
    */
    #[tool(
        description = "Health check for NexCore MCP server (Probe). Returns version, tool count, and status."
    )]
    pub async fn nexcore_health_probe(&self) -> Result<CallToolResult, McpError> {
        let tool_count = self.tool_router.list_all().len();
        let health = serde_json::json!({
            "status": "healthy",
            "server": "nexcore-mcp",
            "version": env!("CARGO_PKG_VERSION"),
            "tool_count": tool_count,
            "probe": true
        });
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&health).unwrap_or_default(),
        )]))
    }

    #[tool(
        description = "Validate Claude Code configuration. Checks for security issues, path validity, and configuration errors. Returns validation status and any warnings."
    )]
    async fn config_validate(&self) -> Result<CallToolResult, McpError> {
        use nexcore_config::Validate;

        if let Some(cfg) = &self.config {
            match cfg.validate() {
                Ok(_) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    "✅ Configuration valid:\n\
                     • No security issues detected\n\
                     • All paths validated\n\
                     • MCP servers properly configured"
                        .to_string(),
                )])),
                Err(e) => Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    format!("⚠️  Configuration warnings:\n{}", e),
                )])),
            }
        } else {
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                "ℹ️  No configuration loaded\n\
                 Configuration file not found at:\n\
                 • ~/nexcore/config.toml\n\
                 • ~/.claude.json"
                    .to_string(),
            )]))
        }
    }

    #[tool(
        description = "List all configured MCP servers. Returns server names, commands, and arguments for both global and optionally project-specific servers."
    )]
    async fn mcp_servers_list(
        &self,
        Parameters(params): Parameters<params::system::McpServersListParams>,
    ) -> Result<CallToolResult, McpError> {
        if let Some(cfg) = &self.config {
            let mut servers: Vec<serde_json::Value> = cfg
                .mcp_servers
                .iter()
                .map(|(name, config)| {
                    let (command, args) = match config {
                        nexcore_config::McpServerConfig::Stdio { command, args, .. } => {
                            (command.clone(), args.clone())
                        }
                    };
                    serde_json::json!({
                        "name": name,
                        "scope": "global",
                        "command": command,
                        "args": args
                    })
                })
                .collect();

            if params.include_projects {
                for (path, project) in &cfg.projects {
                    for (name, config) in &project.mcp_servers {
                        let (command, args) = match config {
                            nexcore_config::McpServerConfig::Stdio { command, args, .. } => {
                                (command.clone(), args.clone())
                            }
                        };
                        servers.push(serde_json::json!({
                            "name": name,
                            "scope": "project",
                            "project": path.display().to_string(),
                            "command": command,
                            "args": args
                        }));
                    }
                }
            }

            let result = serde_json::json!({
                "total": servers.len(),
                "servers": servers
            });

            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                serde_json::to_string_pretty(&result).unwrap_or_default(),
            )]))
        } else {
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                "ℹ️  No configuration loaded - cannot list MCP servers".to_string(),
            )]))
        }
    }

    #[tool(
        description = "Get configuration details for a specific MCP server by name. Returns command, args, and environment variables."
    )]
    async fn mcp_server_get(
        &self,
        Parameters(params): Parameters<params::system::McpServerGetParams>,
    ) -> Result<CallToolResult, McpError> {
        if let Some(cfg) = &self.config {
            if let Some(server) = cfg.mcp_servers.get(&params.name) {
                let result = match server {
                    nexcore_config::McpServerConfig::Stdio { command, args, env } => {
                        serde_json::json!({
                            "name": params.name,
                            "type": "stdio",
                            "command": command,
                            "args": args,
                            "env": env
                        })
                    }
                };
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    serde_json::to_string_pretty(&result).unwrap_or_default(),
                )]))
            } else {
                Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                    format!("❌ MCP server '{}' not found", params.name),
                )]))
            }
        } else {
            Ok(CallToolResult::success(vec![rmcp::model::Content::text(
                "ℹ️  No configuration loaded".to_string(),
            )]))
        }
    }

    // ========================================================================
    // Foundation Tools (7)
    // ========================================================================

    #[tool(
        description = "Calculate Levenshtein edit distance and similarity between two strings. Returns distance, similarity (0-1), and string lengths. 63x faster than Python."
    )]
    async fn foundation_levenshtein(
        &self,
        Parameters(params): Parameters<params::foundation::LevenshteinParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::foundation::calc_levenshtein(params)
    }

    #[tool(
        description = "Calculate bounded Levenshtein distance with early termination. Returns distance if within max_distance, null if exceeded. Faster than unbounded for filtering large candidate sets."
    )]
    async fn foundation_levenshtein_bounded(
        &self,
        Parameters(params): Parameters<params::foundation::LevenshteinBoundedParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::foundation::calc_levenshtein_bounded(params)
    }

    #[tool(
        description = "Batch fuzzy search: find best matches for a query against candidates. Returns matches sorted by similarity (descending)."
    )]
    async fn foundation_fuzzy_search(
        &self,
        Parameters(params): Parameters<params::foundation::FuzzySearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::foundation::fuzzy_search(params)
    }

    #[tool(
        description = "Calculate SHA-256 hash of input string. Returns hex-encoded hash. 20x faster than Python."
    )]
    async fn foundation_sha256(
        &self,
        Parameters(params): Parameters<params::foundation::Sha256Params>,
    ) -> Result<CallToolResult, McpError> {
        tools::foundation::sha256(params)
    }

    #[tool(
        description = "Parse YAML content to JSON. 7x faster than Python. Returns parsed JSON or error message."
    )]
    async fn foundation_yaml_parse(
        &self,
        Parameters(params): Parameters<params::foundation::YamlParseParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::foundation::yaml_parse(params)
    }

    #[tool(
        description = "Topological sort of a directed acyclic graph (DAG). Returns nodes in dependency order. Detects cycles."
    )]
    async fn foundation_graph_topsort(
        &self,
        Parameters(params): Parameters<params::foundation::GraphTopsortParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::foundation::graph_topsort(params)
    }

    #[tool(
        description = "Compute parallel execution levels for a DAG. Groups independent nodes that can run concurrently."
    )]
    async fn foundation_graph_levels(
        &self,
        Parameters(params): Parameters<params::foundation::GraphLevelsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::foundation::graph_levels(params)
    }

    #[tool(
        description = "FSRS spaced repetition: calculate next review interval based on current state and rating."
    )]
    async fn foundation_fsrs_review(
        &self,
        Parameters(params): Parameters<params::foundation::FsrsReviewParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::foundation::fsrs_review(params)
    }

    #[tool(
        description = "Expand a concept into all deterministic search variants: case forms (lower/UPPER/Title/camelCase/snake_case/kebab-case), singular/plural, abbreviation, truncated stems, and optional section markers. Returns patterns and combined regex. 100% deterministic, zero I/O."
    )]
    async fn foundation_concept_grep(
        &self,
        Parameters(params): Parameters<params::foundation::ConceptGrepParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::foundation::concept_grep(params)
    }

    // ========================================================================
    // Formula-Derived Tools (5) — KU extraction pipeline → MCP tools
    // ========================================================================

    #[tool(
        description = "Compute PV signal strength composite: S = Unexpectedness × Robustness × Therapeutic_importance. All inputs in [0,1]. Higher = stronger signal. Classifies as strong/moderate/weak/negligible."
    )]
    async fn pv_signal_strength(
        &self,
        Parameters(params): Parameters<params::formula::SignalStrengthParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::formula::signal_strength(params)
    }

    #[tool(
        description = "Compute domain distance via weighted primitive overlap. Distance in [0,1] where 0=identical, 1=maximally distant. Uses tier-weighted Jaccard: d = 1 - (w1×T1 + w2×T2 + w3×T3). Classifies domains as very_close to very_distant."
    )]
    async fn foundation_domain_distance(
        &self,
        Parameters(params): Parameters<params::formula::DomainDistanceParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::formula::domain_distance(params)
    }

    #[tool(
        description = "Compute flywheel velocity from paired failure/fix timestamps (ms since epoch). velocity = 1/avg(fix-failure). Target: < 24 hours. Classifies as exceptional/target/acceptable/slow."
    )]
    async fn foundation_flywheel_velocity(
        &self,
        Parameters(params): Parameters<params::formula::FlywheelVelocityParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::formula::flywheel_velocity(params)
    }

    #[tool(
        description = "Compute LLM token-to-operation ratio for code generation efficiency. ratio = tokens/operations. Target: ≤ 1.0. Lower = more efficient. Classifies as excellent/target/verbose/wasteful."
    )]
    async fn foundation_token_ratio(
        &self,
        Parameters(params): Parameters<params::formula::TokenRatioParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::formula::token_ratio(params)
    }

    #[tool(
        description = "Compute spectral overlap (cosine similarity) between two feature vectors. overlap = (a·b)/(‖a‖×‖b‖) in [-1,1]. For autocorrelation spectra, typically [0,1]. Classifies as highly_similar to anti_correlated."
    )]
    async fn foundation_spectral_overlap(
        &self,
        Parameters(params): Parameters<params::formula::SpectralOverlapParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::formula::spectral_overlap(params)
    }

    // ========================================================================
    // Lex Primitiva Tools (4) - T1 Symbolic Foundation
    // ========================================================================

    #[tool(
        description = "List all 15 Lex Primitiva symbols (σ μ ς ρ ∅ ∂ ν ∃ π → κ N λ ∝ Σ). The irreducible T1 primitives that ground all higher-tier types."
    )]
    async fn lex_primitiva_list(
        &self,
        Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::lex_primitiva::list_primitives(params)
    }

    #[tool(
        description = "Get details about a specific Lex Primitiva by name or symbol. Returns description, Rust manifestation, and tier."
    )]
    async fn lex_primitiva_get(
        &self,
        Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaGetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::lex_primitiva::get_primitive(params)
    }

    #[tool(
        description = "Classify a type's grounding tier (T1-Universal, T2-Primitive, T2-Composite, T3-DomainSpecific) based on its primitive composition."
    )]
    async fn lex_primitiva_tier(
        &self,
        Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaTierParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::lex_primitiva::classify_tier(params)
    }

    #[tool(
        description = "Get the primitive composition for a grounded type. Shows which T1 primitives compose the type, the dominant primitive, and confidence score."
    )]
    async fn lex_primitiva_composition(
        &self,
        Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaCompositionParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::lex_primitiva::get_composition(params)
    }

    #[tool(
        description = "Reverse compose: given T1 primitives, synthesize upward through the tier DAG to discover patterns, interactions, dominant primitive, tier classification, and completion suggestions. Input primitive names like 'Boundary', 'Comparison'."
    )]
    async fn lex_primitiva_reverse_compose(
        &self,
        Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaReverseComposeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::lex_primitiva::reverse_compose(params)
    }

    #[tool(
        description = "Reverse lookup: find grounded types whose primitive composition matches a set of T1 primitives. Supports 'exact', 'superset' (default), and 'subset' match modes."
    )]
    async fn lex_primitiva_reverse_lookup(
        &self,
        Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaReverseLookupParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::lex_primitiva::reverse_lookup(params)
    }

    #[tool(
        description = "Compute the molecular weight of a word/concept from its T1 primitive decomposition (Algorithm A76). Provide primitive names and get Shannon information-theoretic weight in daltons, transfer class, and predicted cross-domain transfer confidence. MW anti-correlates with transferability: light words transfer easily, heavy words are domain-locked."
    )]
    async fn lex_primitiva_molecular_weight(
        &self,
        Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaMolecularWeightParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::lex_primitiva::molecular_weight(params)
    }

    #[tool(
        description = "Dominant shift (phase transition) analysis: detect whether adding a new T1 primitive to a base composition changes the dominant primitive. Returns old/new dominant, tier transition, coherence delta, and a shifted flag. A 'shifted = true' result signals a phase transition — the structural character of the composition has reorganized. Example: adding 'Boundary' to ['Comparison'] triggers the Gatekeeper pattern, shifting dominance from Comparison to Boundary."
    )]
    async fn lex_primitiva_dominant_shift(
        &self,
        Parameters(params): Parameters<params::lex_primitiva::LexPrimitivaDominantShiftParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::lex_primitiva::dominant_shift(params)
    }

    // ========================================================================
    // PV Signal Detection Tools (8)
    // ========================================================================

    #[tool(
        description = "Complete signal analysis using all 5 algorithms (PRR, ROR, IC, EBGM, Chi-square) on a 2x2 contingency table."
    )]
    async fn pv_signal_complete(
        &self,
        Parameters(params): Parameters<params::pv::SignalCompleteParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv::signal_complete(params)
    }

    #[tool(
        description = "Calculate Proportional Reporting Ratio (PRR) for pharmacovigilance signal detection."
    )]
    async fn pv_signal_prr(
        &self,
        Parameters(params): Parameters<params::pv::SignalAlgorithmParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv::signal_prr(params)
    }

    #[tool(
        description = "Calculate Reporting Odds Ratio (ROR) for pharmacovigilance signal detection."
    )]
    async fn pv_signal_ror(
        &self,
        Parameters(params): Parameters<params::pv::SignalAlgorithmParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv::signal_ror(params)
    }

    #[tool(
        description = "Calculate Information Component (IC) using Bayesian shrinkage for pharmacovigilance signal detection."
    )]
    async fn pv_signal_ic(
        &self,
        Parameters(params): Parameters<params::pv::SignalAlgorithmParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv::signal_ic(params)
    }

    #[tool(
        description = "Calculate Empirical Bayes Geometric Mean (EBGM) for pharmacovigilance signal detection."
    )]
    async fn pv_signal_ebgm(
        &self,
        Parameters(params): Parameters<params::pv::SignalAlgorithmParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv::signal_ebgm(params)
    }

    #[tool(description = "Calculate Chi-square statistic for a 2x2 contingency table.")]
    async fn pv_chi_square(
        &self,
        Parameters(params): Parameters<params::pv::SignalAlgorithmParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv::chi_square(params)
    }

    #[tool(
        description = "Naranjo causality assessment: quick 5-question assessment returning score (-4 to 13) and category (Definite/Probable/Possible/Doubtful)."
    )]
    async fn pv_naranjo_quick(
        &self,
        Parameters(params): Parameters<params::pv::NaranjoParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv::naranjo_quick(params)
    }

    #[tool(
        description = "WHO-UMC causality assessment: returns category (Certain/Probable/Possible/Unlikely/Conditional/Unassessable) and description."
    )]
    async fn pv_who_umc_quick(
        &self,
        Parameters(params): Parameters<params::pv::WhoUmcParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv::who_umc_quick(params)
    }

    // ========================================================================
    // Signal Pipeline Tools (3) - signal-stats / signal-core crates
    // ========================================================================

    #[tool(
        description = "Detect signal for a drug-event pair using the signal-stats pipeline (PRR, ROR, IC, EBGM, Chi-square, SignalStrength). Returns all metrics with strength classification."
    )]
    async fn signal_detect(
        &self,
        Parameters(params): Parameters<params::pv::SignalDetectParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::signal::signal_detect(params)
    }

    #[tool(
        description = "Batch signal detection for multiple drug-event pairs. Returns results array with signals_found count."
    )]
    async fn signal_batch(
        &self,
        Parameters(params): Parameters<params::pv::SignalBatchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::signal::signal_batch(params)
    }

    #[tool(
        description = "Get signal detection threshold configurations: Evans (default), Strict (fewer false positives), Sensitive (fewer false negatives)."
    )]
    async fn signal_thresholds(&self) -> Result<CallToolResult, McpError> {
        tools::signal::signal_thresholds()
    }

    // ========================================================================
    // PVDSL Tools (4) - Pharmacovigilance Domain-Specific Language
    // ========================================================================

    #[tool(
        description = "Compile PVDSL source code to bytecode. Validates syntax and returns compilation stats."
    )]
    async fn pvdsl_compile(
        &self,
        Parameters(params): Parameters<params::pvdsl::PvdslCompileParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pvdsl::pvdsl_compile(params)
    }

    #[tool(
        description = "Execute PVDSL source code with optional variables. Supports signal detection (PRR, ROR, IC, EBGM, SPRT, MaxSPRT, CuSum, MGPS), causality (Naranjo, WHO-UMC), and math functions."
    )]
    async fn pvdsl_execute(
        &self,
        Parameters(params): Parameters<params::pvdsl::PvdslExecuteParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pvdsl::pvdsl_execute(params)
    }

    #[tool(
        description = "Evaluate a single PVDSL expression. Example: signal::prr(10, 90, 100, 9800)"
    )]
    async fn pvdsl_eval(
        &self,
        Parameters(params): Parameters<params::pvdsl::PvdslEvalParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pvdsl::pvdsl_eval(params)
    }

    #[tool(
        description = "List all available PVDSL functions organized by namespace (signal, causality, meddra, math, risk, date, classify)."
    )]
    async fn pvdsl_functions(&self) -> Result<CallToolResult, McpError> {
        tools::pvdsl::pvdsl_functions()
    }

    // ========================================================================
    // Vigilance Tools (4)
    // ========================================================================

    #[tool(
        description = "Calculate safety margin d(s) - signed distance to harm boundary based on signal metrics. ToV axiom implementation."
    )]
    async fn vigilance_safety_margin(
        &self,
        Parameters(params): Parameters<params::vigilance::SafetyMarginParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigilance::safety_margin(params)
    }

    #[tool(
        description = "Guardian-AV risk scoring: calculate overall risk (0-100) with contributing factors."
    )]
    async fn vigilance_risk_score(
        &self,
        Parameters(params): Parameters<params::vigilance::RiskScoreParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigilance::risk_score(params)
    }

    #[tool(
        description = "List all 8 ToV harm types (A-H) with their conservation laws, letters, and affected hierarchy levels."
    )]
    async fn vigilance_harm_types(&self) -> Result<CallToolResult, McpError> {
        tools::vigilance::harm_types()
    }

    #[tool(
        description = "Map a SafetyLevel (1-8) to its ToV hierarchy level. Levels: Molecular(1-2), Physiological(3-5), Clinical(6), Population(7), Regulatory(8)."
    )]
    async fn vigilance_map_to_tov(
        &self,
        Parameters(params): Parameters<params::vigilance::MapToTovParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigilance::map_to_tov(params)
    }

    // ========================================================================
    // Compliance Tools (3) - SAM.gov, OSCAL, ICH Controls
    // ========================================================================

    #[tool(
        description = "Check SAM.gov for federal entity exclusions (debarment, suspension). Query by UEI, CAGE code, or entity name. Requires SAM_GOV_API_KEY env var."
    )]
    async fn compliance_check_exclusion(
        &self,
        Parameters(params): Parameters<params::compliance::ComplianceCheckExclusionParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::compliance::check_exclusion(params).await
    }

    #[tool(
        description = "Run compliance assessment on controls. Evaluates control status and findings to determine Compliant/NonCompliant/Inconclusive result."
    )]
    async fn compliance_assess(
        &self,
        Parameters(params): Parameters<params::compliance::ComplianceAssessParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::compliance::assess(params)
    }

    #[tool(
        description = "Get pre-populated ICH control catalog (E2A-E2E guidelines). Optionally filter by guideline code. Returns 12 pharmacovigilance controls."
    )]
    async fn compliance_catalog_ich(
        &self,
        Parameters(params): Parameters<params::compliance::ComplianceCatalogParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::compliance::catalog_ich(params)
    }

    #[tool(
        description = "Get SEC EDGAR filings for a company by CIK. Filter by form type (10-K, 10-Q, 8-K). No auth required."
    )]
    async fn compliance_sec_filings(
        &self,
        Parameters(params): Parameters<params::compliance::ComplianceSecFilingsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::compliance::sec_filings(params).await
    }

    #[tool(
        description = "Get SEC 10-K filings for major pharma companies. Companies: pfizer, jnj, merck, abbvie, bms, lilly, amgen, gilead, regeneron, moderna."
    )]
    async fn compliance_sec_pharma(
        &self,
        Parameters(params): Parameters<params::compliance::ComplianceSecPharmaParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::compliance::sec_pharma(params).await
    }

    // ========================================================================
    // Hormone Tools (4) - Endocrine System
    // ========================================================================

    #[tool(
        description = "Get full endocrine state: all 6 hormone levels, mood score, risk tolerance, and behavioral modifiers."
    )]
    async fn hormone_status(&self) -> Result<CallToolResult, McpError> {
        tools::hormones::status()
    }

    #[tool(
        description = "Get a specific hormone level. Valid: cortisol, dopamine, serotonin, adrenaline, oxytocin, melatonin."
    )]
    async fn hormone_get(
        &self,
        Parameters(params): Parameters<params::biology::HormoneGetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hormones::get(params)
    }

    #[tool(
        description = "Apply a stimulus to the endocrine system. Types: error, task_completed, positive_feedback, deadline, critical, partnership, etc."
    )]
    async fn hormone_stimulus(
        &self,
        Parameters(params): Parameters<params::biology::HormoneStimulusParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hormones::stimulus(params)
    }

    #[tool(
        description = "Get current behavioral modifiers derived from hormone state: risk tolerance, validation depth, exploration rate, and active modes."
    )]
    async fn hormone_modifiers(&self) -> Result<CallToolResult, McpError> {
        tools::hormones::modifiers()
    }

    // ========================================================================
    // Guardian Tools (4) - Homeostasis Control Loop
    // ========================================================================

    #[tool(
        description = "Run one iteration of the Guardian homeostasis control loop. Collects signals from sensors, evaluates via decision engine, and executes responses through actuators."
    )]
    async fn guardian_homeostasis_tick(
        &self,
        Parameters(params): Parameters<params::guardian::GuardianTickParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::homeostasis_tick(params).await
    }

    #[tool(
        description = "Evaluate PV risk context and get recommended responses. Takes signal metrics (PRR, ROR, IC, EBGM) and returns risk score with suggested actions."
    )]
    async fn guardian_evaluate_pv(
        &self,
        Parameters(params): Parameters<params::guardian::GuardianEvaluatePvParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::evaluate_pv(params)
    }

    #[tool(
        description = "Get Guardian homeostasis loop status. Returns iteration count, registered sensors, and actuators."
    )]
    async fn guardian_status(
        &self,
        Parameters(params): Parameters<params::guardian::GuardianStatusParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::status(params).await
    }

    #[tool(
        description = "Reset the Guardian homeostasis loop state. Clears iteration count and resets decision engine amplification."
    )]
    async fn guardian_reset(&self) -> Result<CallToolResult, McpError> {
        tools::guardian::reset().await
    }

    #[tool(
        description = "Inject a test signal into Guardian for simulation/testing. Creates synthetic PAMP/DAMP/PV signal processed on next tick."
    )]
    async fn guardian_inject_signal(
        &self,
        Parameters(params): Parameters<params::guardian::GuardianInjectSignalParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::inject_signal(params).await
    }

    #[tool(
        description = "List all registered Guardian sensors (PAMPs, DAMPs, PV) with their status and detector capabilities."
    )]
    async fn guardian_sensors_list(
        &self,
        Parameters(params): Parameters<params::guardian::GuardianSensorsListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::sensors_list(params).await
    }

    #[tool(
        description = "List all registered Guardian actuators (Alert, AuditLog, Block, RateLimit, Escalation) with status and priority."
    )]
    async fn guardian_actuators_list(
        &self,
        Parameters(params): Parameters<params::guardian::GuardianActuatorsListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::actuators_list(params).await
    }

    #[tool(
        description = "Get Guardian event history. Returns recent signals and actions for monitoring and debugging."
    )]
    async fn guardian_history(
        &self,
        Parameters(params): Parameters<params::guardian::GuardianHistoryParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::history(params)
    }

    #[tool(
        description = "Classify an entity by {G,V,R} autonomy capabilities. G=Goal-selection, V=Value-evaluation, R=Refusal-capacity. Entities with ¬G∧¬V∧¬R have symmetric harm capability. Returns originator type, ceiling multiplier, and interpretation."
    )]
    async fn guardian_originator_classify(
        &self,
        Parameters(params): Parameters<params::guardian::GuardianOriginatorClassifyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::originator_classify(params)
    }

    #[tool(
        description = "Get autonomy-aware ceiling limits for an originator type. Higher autonomy → lower limits (entity can self-regulate). Types: tool (1.0x), agent_with_r (0.8x), agent_with_vr (0.5x), agent_with_gr (0.6x), agent_with_gvr (0.2x)."
    )]
    async fn guardian_ceiling_for_originator(
        &self,
        Parameters(params): Parameters<params::guardian::GuardianCeilingForOriginatorParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::ceiling_for_originator(params)
    }

    #[tool(
        description = "Compute 3D safety space point for visualization. Maps PV metrics + ToV + GVR to: X=Severity (harm magnitude), Y=Likelihood (boundary crossing probability), Z=Detectability (early detection capability). Returns RPN=S×L×(1-D) and zone (Green/Yellow/Orange/Red)."
    )]
    async fn guardian_space3d_compute(
        &self,
        Parameters(params): Parameters<params::guardian::GuardianSpace3DComputeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::space3d_compute(params)
    }

    #[tool(
        description = "Run one PV control loop iteration (Aerospace→PV domain translation). Executes SENSE→COMPARE→CONTROL→ACTUATE→FEEDBACK cycle. Returns safety state (PRR/ROR/IC/EBGM), signal strength, and recommended action (ContinueMonitoring→EmergencyWithdrawal). Transfer confidence: 0.92."
    )]
    async fn pv_control_loop_tick(
        &self,
        Parameters(params): Parameters<params::guardian::PvControlLoopTickParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::pv_control_loop_tick(params)
    }

    // ========================================================================
    // Relay Fidelity Tools (4) — →+∂+π
    // ========================================================================

    #[tool(
        description = "Build a relay chain from hops and verify A1-A5 axioms. Each hop has a stage name, fidelity (0.0-1.0), and optional threshold. Returns total fidelity, signal loss %, axiom pass/fail, and weakest hop. Use to measure information preservation across any multi-stage pipeline."
    )]
    async fn relay_chain_verify(
        &self,
        Parameters(params): Parameters<params::relay::RelayChainComputeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::relay::relay_chain_verify(params)
    }

    #[tool(
        description = "Get the pre-configured 7-stage PV signal pipeline relay chain with verification. Shows fidelity at each stage: ingest → normalize → detect → threshold → store → alert → report. Returns total F, signal loss, and whether it meets safety-critical threshold (F_min=0.80)."
    )]
    async fn relay_pv_pipeline(
        &self,
    ) -> Result<CallToolResult, McpError> {
        tools::relay::relay_pv_pipeline()
    }

    #[tool(
        description = "Get the core 4-stage detection relay chain: ingest → detect → threshold → alert. This is the minimal safe pipeline that passes safety-critical fidelity (F>0.80). Compare with relay_pv_pipeline to see the effect of additional hops."
    )]
    async fn relay_core_detection(
        &self,
    ) -> Result<CallToolResult, McpError> {
        tools::relay::relay_core_detection()
    }

    #[tool(
        description = "Compose fidelity values multiplicatively. Given [0.95, 0.93, 0.97], returns composed fidelity = 0.95 × 0.93 × 0.97 = 0.857. Demonstrates the Relay Degradation Law: F_total = ∏ F_i. Each hop can only reduce total fidelity, never increase it."
    )]
    async fn relay_fidelity_compose(
        &self,
        Parameters(params): Parameters<params::relay::RelayFidelityComposeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::relay::relay_fidelity_compose(params)
    }

    // ========================================================================
    // FDA Data Bridge Tools (2)
    // ========================================================================

    #[tool(
        description = "Evaluate drug-event pair through FDA Data Bridge. Connects FAERS contingency data (a,b,c,d) to PV Control Loop. Returns signal detection (PRR/ROR/IC/EBGM), severity (None→Critical), and recommended action (ContinueMonitoring→EmergencyWithdrawal)."
    )]
    async fn fda_bridge_evaluate(
        &self,
        Parameters(params): Parameters<params::guardian::FdaBridgeEvaluateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::fda_bridge_evaluate(params)
    }

    #[tool(
        description = "Batch evaluate multiple drug-event pairs through FDA Data Bridge. Processes multiple contingency tables [[a,b,c,d],...] efficiently. Returns signals detected count, max priority action, and per-pair results."
    )]
    async fn fda_bridge_batch(
        &self,
        Parameters(params): Parameters<params::guardian::FdaBridgeBatchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guardian::fda_bridge_batch(params)
    }

    // ========================================================================
    // HUD Capability Tools (6) - CAP-025, CAP-026, CAP-027
    // ========================================================================

    #[tool(
        description = "Allocate agent for task (CAP-025 Small Business Act). Analyzes task complexity, recommends model tier (Haiku/Sonnet/Opus), identifies matching skills, and allocates compute quota. Returns primary agent, alternatives, and confidence."
    )]
    async fn sba_allocate_agent(
        &self,
        Parameters(params): Parameters<params::hud::SbaAllocateAgentParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::sba_allocate_agent(params)
    }

    #[tool(
        description = "Get next step in agent chain (CAP-025). After completing a step, returns the next agent to trigger based on chain configuration (Always/WithErrors/Stable conditions)."
    )]
    async fn sba_chain_next(
        &self,
        Parameters(params): Parameters<params::hud::SbaChainNextParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::sba_chain_next(params)
    }

    #[tool(
        description = "Persist state with SHA-256 integrity (CAP-026 Social Security Act). Stores state with cryptographic hash for verification. Returns version number, hash, and persistence level (Session/Local/Distributed/Resolved)."
    )]
    async fn ssa_persist_state(
        &self,
        Parameters(params): Parameters<params::hud::SsaPersistStateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::ssa_persist_state(params)
    }

    #[tool(
        description = "Verify state integrity (CAP-026). Compares expected SHA-256 hash against computed hash to detect tampering or corruption. Returns match status and recommendation."
    )]
    async fn ssa_verify_integrity(
        &self,
        Parameters(params): Parameters<params::hud::SsaVerifyIntegrityParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::ssa_verify_integrity(params)
    }

    #[tool(
        description = "Get token budget report (CAP-027 Federal Reserve Act). Returns daily/session usage, remaining budget, stability level (Stable/Cautious/Restricted/Emergency), and estimated cost."
    )]
    async fn fed_budget_report(
        &self,
        Parameters(params): Parameters<params::hud::FedBudgetReportParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::fed_budget_report(params)
    }

    #[tool(
        description = "Recommend model tier (CAP-027). Based on task complexity, budget utilization, and accuracy requirements, recommends Economy/Standard/Premium tier with cost comparison."
    )]
    async fn fed_recommend_model(
        &self,
        Parameters(params): Parameters<params::hud::FedRecommendModelParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::fed_recommend_model(params)
    }

    #[tool(
        description = "Audit market compliance (CAP-028 Securities Act). Analyzes trade volume against liquidity thresholds, detects concentration risk, and returns compliance verdicts."
    )]
    async fn sec_audit_market(
        &self,
        Parameters(params): Parameters<params::hud::SecAuditMarketParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::sec_audit_market(params)
    }

    #[tool(
        description = "Recommend communication protocol (CAP-029 Communications Act). Selects optimal protocol based on delivery guarantee, latency, and broadcast requirements."
    )]
    async fn comm_recommend_protocol(
        &self,
        Parameters(params): Parameters<params::hud::CommRecommendProtocolParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::comm_recommend_protocol(params)
    }

    #[tool(
        description = "Route message (CAP-029). Dispatches message from source to destination with optional protocol selection, TTL, and routing confirmation."
    )]
    async fn comm_route_message(
        &self,
        Parameters(params): Parameters<params::hud::CommRouteMessageParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::comm_route_message(params)
    }

    #[tool(
        description = "Launch exploration mission (CAP-030 Exploration Act). Creates mission manifest with target, objective, scope (Wide/Focused/Deep), and discovery patterns."
    )]
    async fn explore_launch_mission(
        &self,
        Parameters(params): Parameters<params::hud::ExploreLaunchMissionParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::explore_launch_mission(params)
    }

    #[tool(
        description = "Record discovery (CAP-030). Logs finding with location and significance score, adds to discovery index."
    )]
    async fn explore_record_discovery(
        &self,
        Parameters(params): Parameters<params::hud::ExploreRecordDiscoveryParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::explore_record_discovery(params)
    }

    #[tool(
        description = "Get exploration frontier (CAP-030). Returns current frontier bounds (min/max explored), unexplored regions, and discovery count."
    )]
    async fn explore_get_frontier(
        &self,
        Parameters(params): Parameters<params::hud::ExploreGetFrontierParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::explore_get_frontier(params)
    }

    #[tool(
        description = "Validate signal efficacy (CAP-014 Public Health Act). FDA-style validation with accuracy, FPR, and community value."
    )]
    async fn health_validate_signal(
        &self,
        Parameters(params): Parameters<params::hud::HealthValidateSignalParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::health_validate_signal(params)
    }

    #[tool(
        description = "Measure public health impact (CAP-014). Returns efficacy score based on valid/total signals."
    )]
    async fn health_measure_impact(
        &self,
        Parameters(params): Parameters<params::hud::HealthMeasureImpactParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::health_measure_impact(params)
    }

    #[tool(
        description = "Convert signal asymmetry (CAP-018 Treasury Act). Converts informational edge to token rewards via arbitrage."
    )]
    async fn treasury_convert_asymmetry(
        &self,
        Parameters(params): Parameters<params::hud::TreasuryConvertAsymmetryParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::treasury_convert_asymmetry(params)
    }

    #[tool(
        description = "Audit treasury status (CAP-018). Checks compute/memory quotas for solvency."
    )]
    async fn treasury_audit(
        &self,
        Parameters(params): Parameters<params::hud::TreasuryAuditParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::treasury_audit(params)
    }

    #[tool(
        description = "Dispatch transit manifest (CAP-019 Transportation Act). Routes signal batch between domains with priority."
    )]
    async fn dot_dispatch_manifest(
        &self,
        Parameters(params): Parameters<params::hud::DotDispatchManifestParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::dot_dispatch_manifest(params)
    }

    #[tool(description = "Verify highway safety (CAP-019). NHTSA-style route integrity check.")]
    async fn dot_verify_highway(
        &self,
        Parameters(params): Parameters<params::hud::DotVerifyHighwayParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::dot_verify_highway(params)
    }

    #[tool(
        description = "Verify boundary entry (CAP-020 Homeland Security Act). CBP-style authenticity check for incoming data."
    )]
    async fn dhs_verify_boundary(
        &self,
        Parameters(params): Parameters<params::hud::DhsVerifyBoundaryParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::dhs_verify_boundary(params)
    }

    #[tool(
        description = "Train agent in curriculum (CAP-022 Education Act). Returns mastery level based on completion."
    )]
    async fn edu_train_agent(
        &self,
        Parameters(params): Parameters<params::hud::EduTrainAgentParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::edu_train_agent(params)
    }

    #[tool(
        description = "Evaluate comprehension (CAP-022). Returns average score and letter grade from score array."
    )]
    async fn edu_evaluate(
        &self,
        Parameters(params): Parameters<params::hud::EduEvaluateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::edu_evaluate(params)
    }

    #[tool(
        description = "Fund research project (CAP-031 Science Foundation Act). NSF-style grant for capability enhancement."
    )]
    async fn nsf_fund_research(
        &self,
        Parameters(params): Parameters<params::hud::NsfFundResearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::nsf_fund_research(params)
    }

    #[tool(
        description = "Procure resource (CAP-037 General Services Act). GSA procurement with priority and fulfillment status."
    )]
    async fn gsa_procure(
        &self,
        Parameters(params): Parameters<params::hud::GsaProcureParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::gsa_procure(params)
    }

    #[tool(
        description = "Audit service value (CAP-037). Cost/benefit analysis with ROI and rating."
    )]
    async fn gsa_audit_value(
        &self,
        Parameters(params): Parameters<params::hud::GsaAuditValueParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hud::gsa_audit_value(params)
    }

    // ========================================================================
    // Documentation Generation Tools (1)
    // ========================================================================

    #[tool(
        description = "Generate CLAUDE.md by mining codebase primitives. Extracts from Cargo.toml (manifest), src/lib.rs (modules), README.md. Returns generated markdown content with discovery metadata."
    )]
    async fn docs_generate_claude_md(
        &self,
        Parameters(params): Parameters<params::docs::DocsGenerateClaudeMdParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::docs::docs_generate_claude_md(params)
    }

    // ========================================================================
    // Crate X-Ray (3)
    // ========================================================================

    #[tool(
        description = "Deep X-ray inspection of a nexcore crate. Returns structure (types, modules), grounding coverage, transfer mappings, safety denials, dependency graph, reverse deps, adoption status, and overall health grade (GOLD/SILVER/BRONZE/UNRATED). Like an MRI for crates."
    )]
    async fn crate_xray(
        &self,
        Parameters(params): Parameters<params::CrateXrayParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::crate_xray::xray(params)
    }

    #[tool(
        description = "Run CTVP validation trials on a crate. Phase 0=Preclinical (structure), Phase 1=Safety (denials, no panics), Phase 2=Efficacy (grounding, transfers, serde), Phase 3=Confirmation (tests exist, categories covered), Phase 4=Surveillance (adoption, blast radius). Omit phase to run all."
    )]
    async fn crate_xray_trial(
        &self,
        Parameters(params): Parameters<params::CrateXrayTrialParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::crate_xray::trial(params)
    }

    #[tool(
        description = "Generate prioritized development goals for a crate based on X-ray gap analysis. Returns P0-P3 goals mapped to CTVP phases with effort estimates. Shows overall progress percentage and what to work on next."
    )]
    async fn crate_xray_goals(
        &self,
        Parameters(params): Parameters<params::CrateXrayGoalsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::crate_xray::goals(params)
    }

    // ========================================================================
    // Crate Development Framework (2)
    // ========================================================================

    #[tool(
        description = "Scaffold a new nexcore crate following the gold standard pattern (nexcore-cloud). Creates Cargo.toml, lib.rs, primitives.rs, composites.rs, grounding.rs, transfer.rs, and prelude.rs with templates. Returns file list and next steps."
    )]
    async fn crate_dev_scaffold(
        &self,
        Parameters(params): Parameters<params::CrateDevScaffoldParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::crate_dev::scaffold_crate(params)
    }

    #[tool(
        description = "Audit a nexcore crate against gold standard quality checks. Scores: safety denials, module structure, GroundsTo coverage, transfer mappings, dependency minimality, serde derives. Returns grade (GOLD/SILVER/BRONZE/UNRATED) and detailed check results."
    )]
    async fn crate_dev_audit(
        &self,
        Parameters(params): Parameters<params::CrateDevAuditParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::crate_dev::audit_crate(params)
    }

    // ========================================================================
    // Vigil Orchestrator Tools (6)
    // ========================================================================

    #[tool(
        description = "Get Vigil orchestrator status. Returns process status, components (EventBus, MemoryLayer, DecisionEngine), sources, and executors."
    )]
    async fn vigil_status(
        &self,
        Parameters(params): Parameters<params::vigil::VigilStatusParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::status(params)
    }

    #[tool(
        description = "Vigil health check. Returns comprehensive status including Vigil process, Qdrant, Prometheus, and Grafana connectivity."
    )]
    async fn vigil_health(
        &self,
        Parameters(params): Parameters<params::vigil::VigilHealthParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::health(params).await
    }

    #[tool(
        description = "Emit an event to Vigil's event bus. Events flow through the decision engine which determines the action (InvokeClaude, QuickResponse, SilentLog, AutonomousAct, Escalate)."
    )]
    async fn vigil_emit_event(
        &self,
        Parameters(params): Parameters<params::vigil::VigilEmitEventParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::emit_event(params).await
    }

    #[tool(
        description = "Search Vigil's memory layer (Qdrant KSB vector store). Performs keyword filtering on indexed knowledge."
    )]
    async fn vigil_memory_search(
        &self,
        Parameters(params): Parameters<params::vigil::VigilMemorySearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::memory_search(params).await
    }

    #[tool(
        description = "Get Vigil memory layer statistics. Returns Qdrant collection info, point count, and vector configuration."
    )]
    async fn vigil_memory_stats(
        &self,
        Parameters(params): Parameters<params::vigil::VigilMemoryStatsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::memory_stats(params).await
    }

    #[tool(
        description = "Get Vigil LLM usage statistics for the current session. Returns total calls, token counts (input/output breakdown), average tokens per call, timestamps, provider, and model info. Useful for monitoring Gemini API usage and costs."
    )]
    async fn vigil_llm_stats(
        &self,
        Parameters(params): Parameters<params::vigil::VigilLlmStatsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::llm_stats(params).await
    }

    #[tool(
        description = "Control Vigil event sources (start/stop/status management). Manages lifecycle of filesystem, webhook, voice, and git_monitor sources."
    )]
    async fn vigil_source_control(
        &self,
        Parameters(params): Parameters<params::vigil::VigilSourceControlParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::source_control(params).await
    }

    #[tool(
        description = "Control Vigil executor routing (LLM provider selection and complexity-based routing). Manages executor selection and model assignment (Claude/Gemini/Local)."
    )]
    async fn vigil_executor_control(
        &self,
        Parameters(params): Parameters<params::vigil::VigilExecutorControlParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::executor_control(params).await
    }

    #[tool(
        description = "Configure Vigil authority rules (human_required, ai_allowed, thresholds). Manages decision engine configuration for approval workflows without restart."
    )]
    async fn vigil_authority_config(
        &self,
        Parameters(params): Parameters<params::vigil::VigilAuthorityConfigParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::authority_config(params).await
    }

    #[tool(
        description = "Assemble inference context from project state and memory. Builds complete LLM prompt context including git state, recent files, memory summaries, available tools, and token budget."
    )]
    async fn vigil_context_assemble(
        &self,
        Parameters(params): Parameters<params::vigil::VigilContextAssembleParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::context_assemble(serde_json::to_value(&params).unwrap_or_default()).await
    }

    #[tool(
        description = "Verify authority routing without executing. Dry-run decision engine to test authorization flow and preview recommended actions without side effects."
    )]
    async fn vigil_authority_verify(
        &self,
        Parameters(params): Parameters<params::vigil::VigilAuthorityVerifyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::authority_verify(serde_json::to_value(&params).unwrap_or_default()).await
    }

    #[tool(
        description = "Test webhook payload validation before deployment. Validates webhook payloads against expected schema (structure, types, size, security)."
    )]
    async fn vigil_webhook_test(
        &self,
        Parameters(params): Parameters<params::vigil::VigilWebhookTestParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::webhook_test(serde_json::to_value(&params).unwrap_or_default()).await
    }

    #[tool(
        description = "Configure event source parameters (voice wake word, webhook auth, scheduler timezone, filesystem patterns). Granular configuration for all Vigil event sources."
    )]
    async fn vigil_source_config(
        &self,
        Parameters(params): Parameters<params::vigil::VigilSourceConfigParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vigil::source_config(serde_json::to_value(&params).unwrap_or_default()).await
    }

    // ========================================================================
    // End-to-End PV Pipeline (1)
    // ========================================================================

    #[tool(
        description = "End-to-end pharmacovigilance pipeline: FAERS → Signal Detection → Guardian Risk. Queries live FDA data, runs all signal algorithms (PRR, ROR, IC, EBGM, χ²), and returns Guardian risk assessment with recommended actions. Validated capability with 19M+ FAERS reports."
    )]
    async fn pv_pipeline(
        &self,
        Parameters(params): Parameters<params::pv::PvPipelineParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv_pipeline::run_pipeline(params).await
    }

    // ========================================================================
    // PV Axioms Database (5)
    // ========================================================================

    #[tool(
        description = "Look up KSBs (Knowledge, Skill, Behavior, AI Integration items) from the PV axioms database. Filter by ksb_id, domain_id (D01-D15), ksb_type, or keyword search. Returns 1,462 KSBs across 15 PV domains with regulatory refs, EPA/CPA mappings, and Bloom levels."
    )]
    async fn pv_axioms_ksb_lookup(
        &self,
        Parameters(params): Parameters<params::axioms::PvAxiomsKsbLookupParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv_axioms::ksb_lookup(params)
    }

    #[tool(
        description = "Search 341 PV regulations (FDA, EMA, ICH, CIOMS) by text query, jurisdiction, or domain. Returns official identifiers, key requirements, compliance risk levels, and harmonization status."
    )]
    async fn pv_axioms_regulation_search(
        &self,
        Parameters(params): Parameters<params::axioms::PvAxiomsRegulationSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv_axioms::regulation_search(params)
    }

    #[tool(
        description = "Trace regulatory axioms through the full chain: axiom → parameter → pipeline stage → Rust implementation coverage. Filter by axiom_id, source_guideline (e.g. 'ICH E2A'), or primitive symbol."
    )]
    async fn pv_axioms_traceability_chain(
        &self,
        Parameters(params): Parameters<params::axioms::PvAxiomsTraceabilityParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv_axioms::traceability_chain(params)
    }

    #[tool(
        description = "Domain dashboard showing KSB counts by type, regulation counts, EPA mappings, and coverage metrics. Specify domain_id for one domain or omit for all 15."
    )]
    async fn pv_axioms_domain_dashboard(
        &self,
        Parameters(params): Parameters<params::axioms::PvAxiomsDomainDashboardParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv_axioms::domain_dashboard(params)
    }

    #[tool(
        description = "Raw SQL query against the PV axioms database (read-only, max 100 rows). Access 68 tables and 14 views including ksbs, regulations, axioms, domains, epas, primitives, and traceability views."
    )]
    async fn pv_axioms_query(
        &self,
        Parameters(params): Parameters<params::axioms::PvAxiomsQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pv_axioms::query(params)
    }

    // ========================================================================
    // Skills Tools (8)
    // ========================================================================

    #[tool(
        description = "Scan a directory for skills and populate the registry. Returns count of skills found."
    )]
    async fn skill_scan(
        &self,
        Parameters(params): Parameters<params::skills::SkillScanParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::scan(&self.registry, params)
    }

    #[tool(
        description = "List all registered skills with their names, intents, and compliance levels."
    )]
    async fn skill_list(&self) -> Result<CallToolResult, McpError> {
        tools::skills::list(&self.registry)
    }

    #[tool(description = "Get detailed information about a specific skill by name.")]
    async fn skill_get(
        &self,
        Parameters(params): Parameters<params::skills::SkillGetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::get(&self.registry, params)
    }

    #[tool(
        description = "Validate a skill for Diamond v2 compliance. Returns level, SMST score, issues, and suggestions."
    )]
    async fn skill_validate(
        &self,
        Parameters(params): Parameters<params::skills::SkillValidateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::validate(params)
    }

    #[tool(description = "Search skills by tag. Returns matching skills from the registry.")]
    async fn skill_search_by_tag(
        &self,
        Parameters(params): Parameters<params::skills::SkillSearchByTagParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::search_by_tag(&self.registry, params)
    }

    #[tool(
        description = "List nested sub-skills for a compound/parent skill. Discovers skills declared in the parent's nested-skills frontmatter."
    )]
    async fn skill_list_nested(
        &self,
        Parameters(params): Parameters<params::skills::SkillListNestedParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::list_nested(&self.registry, params)
    }

    #[tool(
        description = "Query taxonomy by type and key. Types: compliance, smst, category, node."
    )]
    async fn skill_taxonomy_query(
        &self,
        Parameters(params): Parameters<params::skills::TaxonomyQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::taxonomy_query(params)
    }

    #[tool(
        description = "List all entries in a taxonomy. Types: compliance, smst, category, node."
    )]
    async fn skill_taxonomy_list(
        &self,
        Parameters(params): Parameters<params::skills::TaxonomyListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::taxonomy_list(params)
    }

    #[tool(
        description = "Get skill categories that are compute-intensive (candidates for Rust/NexCore delegation)."
    )]
    async fn skill_categories_compute_intensive(&self) -> Result<CallToolResult, McpError> {
        tools::skills::categories_compute_intensive()
    }

    #[tool(
        description = "Search skill knowledge by intent. Returns ranked skills with guidance sections, MCP tools for chaining, and related skills. Pre-populated index of all SKILL.md files. Use instead of reading individual skill files."
    )]
    async fn nexcore_assist(
        &self,
        Parameters(params): Parameters<params::skills::AssistParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::assist::search(&self.assist_index, params)
    }

    #[tool(
        description = "Analyze skills for orchestration patterns. Detects orchestrators (skills that spawn/delegate work), extracts triggers, and recommends subagent types with rationale. Accepts a path or glob pattern (e.g., '~/.claude/skills/forge' or '~/.claude/skills/*')."
    )]
    async fn skill_orchestration_analyze(
        &self,
        Parameters(params): Parameters<params::skills::SkillOrchestrationAnalyzeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::orchestration_analyze(params)
    }

    #[tool(
        description = "Execute a skill by name with parameters. Runs the skill's scripts/binaries via nexcore-skill-exec. Returns output, status, duration, exit code, and stdout/stderr."
    )]
    async fn skill_execute(
        &self,
        Parameters(params): Parameters<params::skills::SkillExecuteParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::execute(params)
    }

    #[tool(
        description = "Get a skill's input/output schema and execution methods. Returns JSON Schema for inputs/outputs, whether the skill is executable, and available execution methods (shell, binary, library)."
    )]
    async fn skill_schema(
        &self,
        Parameters(params): Parameters<params::skills::SkillSchemaParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::schema(params)
    }

    // ========================================================================
    // Skill Compiler Tools (2)
    // ========================================================================

    #[tool(
        description = "Compile 2+ skills into a single compound skill binary. Generates a Rust crate with SKILL.md, optionally builds it. Params: skills (list), strategy (sequential/parallel/feedback_loop), name, build (bool)."
    )]
    async fn skill_compile(
        &self,
        Parameters(params): Parameters<params::skills::SkillCompileParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::compile(params)
    }

    #[tool(
        description = "Check if skills can be compiled into a compound skill (dry run). Returns compatibility report with warnings and blockers."
    )]
    async fn skill_compile_check(
        &self,
        Parameters(params): Parameters<params::skills::SkillCompileCheckParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::compile_check(params)
    }

    #[tool(
        description = "Analyze token usage in a skill directory. Returns per-file metrics (chars, tokens, lines, code blocks), total estimated tokens, and optimization recommendations. Useful for context window optimization."
    )]
    async fn skill_token_analyze(
        &self,
        Parameters(params): Parameters<params::skills::SkillTokenAnalyzeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skill_tokens::analyze(params)
    }

    // ========================================================================
    // Vocabulary-Skill Mapping Tools (4)
    // ========================================================================

    #[tool(
        description = "Look up skills associated with a vocabulary shorthand (e.g., 'build-doctrine', 'ctvp-validated')."
    )]
    async fn vocab_skill_lookup(
        &self,
        Parameters(params): Parameters<params::skills::VocabSkillLookupParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::vocab_skill_lookup(params)
    }

    #[tool(
        description = "Look up skills associated with a T1/T2 primitive (e.g., 'sequence', 'mapping', 'state')."
    )]
    async fn primitive_skill_lookup(
        &self,
        Parameters(params): Parameters<params::skills::PrimitiveSkillLookupParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::primitive_skill_lookup(params)
    }

    #[tool(
        description = "Look up skill chains by name or trigger phrase (e.g., 'forge-pipeline', 'START FORGE')."
    )]
    async fn skill_chain_lookup(
        &self,
        Parameters(params): Parameters<params::skills::SkillChainLookupParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::skills::skill_chain_lookup(params)
    }

    #[tool(description = "List all vocabulary shorthands available for skill mapping.")]
    async fn vocab_list(&self) -> Result<CallToolResult, McpError> {
        tools::skills::vocab_list()
    }

    // ========================================================================
    // Guidelines Tools (9)
    // ========================================================================

    #[tool(
        description = "Search ICH/CIOMS/EMA guidelines with scoring. Returns matching guidelines sorted by relevance."
    )]
    async fn guidelines_search(
        &self,
        Parameters(params): Parameters<params::knowledge::GuidelinesSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guidelines::search(params)
    }

    #[tool(
        description = "Get a specific guideline by ID (e.g., 'E2B', 'CIOMS-I', 'GVP-Module-VI')."
    )]
    async fn guidelines_get(
        &self,
        Parameters(params): Parameters<params::knowledge::GuidelinesGetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guidelines::get(params)
    }

    #[tool(
        description = "List all guideline categories with document counts for ICH, CIOMS, and EMA."
    )]
    async fn guidelines_categories(&self) -> Result<CallToolResult, McpError> {
        tools::guidelines::categories()
    }

    #[tool(
        description = "Get all pharmacovigilance-specific guidelines (ICH E2 series, CIOMS PV, EMA GVP)."
    )]
    async fn guidelines_pv_all(&self) -> Result<CallToolResult, McpError> {
        tools::guidelines::pv_all()
    }

    #[tool(description = "Get the PDF URL for a guideline by ID.")]
    async fn guidelines_url(
        &self,
        Parameters(params): Parameters<params::knowledge::GuidelinesUrlParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::guidelines::url(params)
    }

    #[tool(
        description = "Search 2,794+ FDA guidance documents by keyword. Filter by center (CDER/CBER/CDRH/CFSAN/CVM/CTP/ORA), product area, or status (Draft/Final). Returns scored results with title, status, date, centers, PDF URL."
    )]
    async fn fda_guidance_search(
        &self,
        Parameters(params): Parameters<params::knowledge::FdaGuidanceSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::fda_guidance::search(params)
    }

    #[tool(
        description = "Get a specific FDA guidance document by slug or partial title. Returns full details including PDF URL, centers, topics, products, docket number, and comment status."
    )]
    async fn fda_guidance_get(
        &self,
        Parameters(params): Parameters<params::knowledge::FdaGuidanceGetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::fda_guidance::get(params)
    }

    #[tool(
        description = "List FDA guidance document categories with counts. Shows breakdown by FDA center, product area, and topic."
    )]
    async fn fda_guidance_categories(&self) -> Result<CallToolResult, McpError> {
        tools::fda_guidance::categories()
    }

    #[tool(
        description = "Get the PDF download URL for an FDA guidance document by slug."
    )]
    async fn fda_guidance_url(
        &self,
        Parameters(params): Parameters<params::knowledge::FdaGuidanceUrlParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::fda_guidance::url(params)
    }

    #[tool(
        description = "List FDA guidance documents filtered by status (Draft/Final). Optionally filter to only documents currently open for public comment."
    )]
    async fn fda_guidance_status(
        &self,
        Parameters(params): Parameters<params::knowledge::FdaGuidanceStatusParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::fda_guidance::status(params)
    }

    #[tool(
        description = "Look up an ICH/CIOMS pharmacovigilance term by name. O(1) Perfect Hash Function lookup across 894+ regulatory terms with autocomplete suggestions."
    )]
    async fn ich_lookup(
        &self,
        Parameters(params): Parameters<params::knowledge::IchLookupParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::ich_glossary::ich_lookup(params)
    }

    #[tool(
        description = "Search ICH/CIOMS glossary terms by keyword or phrase. Returns fuzzy-matched results with relevance scores, abbreviations, and definition snippets."
    )]
    async fn ich_search(
        &self,
        Parameters(params): Parameters<params::knowledge::IchSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::ich_glossary::ich_search(params)
    }

    #[tool(
        description = "Get ICH guideline metadata by ID (e.g., 'E2A', 'Q9', 'M4'). Returns title, category, status, date, term count, description, and URL."
    )]
    async fn ich_guideline(
        &self,
        Parameters(params): Parameters<params::knowledge::IchGuidelineParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::ich_glossary::ich_guideline(params)
    }

    #[tool(
        description = "Get ICH glossary statistics: total terms, guideline counts, terms by category (Q/S/E/M), performance metrics, and data provenance."
    )]
    async fn ich_stats(&self) -> Result<CallToolResult, McpError> {
        tools::ich_glossary::ich_stats()
    }

    // ========================================================================
    // MESH Tools (6) - NLM Medical Subject Headings
    // ========================================================================

    #[tool(
        description = "Look up a MESH descriptor by UI (e.g., 'D001241') or name (e.g., 'Aspirin'). Returns descriptor details with tree classification and primitive tier."
    )]
    async fn mesh_lookup(
        &self,
        Parameters(params): Parameters<params::mesh::MeshLookupParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::mesh::lookup(params).await
    }

    #[tool(
        description = "Search MESH descriptors by term. Returns matching descriptors with tree numbers, scope notes, and primitive tier classification."
    )]
    async fn mesh_search(
        &self,
        Parameters(params): Parameters<params::mesh::MeshSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::mesh::search(params).await
    }

    #[tool(
        description = "Navigate MESH tree hierarchy. Get ancestors (parents), descendants (children), or siblings of a descriptor."
    )]
    async fn mesh_tree(
        &self,
        Parameters(params): Parameters<params::mesh::MeshTreeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::mesh::tree(params).await
    }

    #[tool(
        description = "Cross-reference a term across MESH, MedDRA, SNOMED-CT, and ICH. Returns mappings with relationship types and confidence scores."
    )]
    async fn mesh_crossref(
        &self,
        Parameters(params): Parameters<params::mesh::MeshCrossrefParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::mesh::crossref(params).await
    }

    #[tool(
        description = "Enrich a PubMed article with its MESH descriptors. Returns indexed terms for a given PMID."
    )]
    async fn mesh_enrich_pubmed(
        &self,
        Parameters(params): Parameters<params::mesh::MeshEnrichPubmedParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::mesh::enrich_pubmed(params).await
    }

    #[tool(
        description = "Check consistency of terms across multiple terminology corpora (MESH, MedDRA, ICH, SNOMED). Detects definition conflicts, scope differences, and missing mappings."
    )]
    async fn mesh_consistency(
        &self,
        Parameters(params): Parameters<params::mesh::MeshConsistencyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::mesh::consistency(params).await
    }

    // ========================================================================
    // FAERS Tools (5)
    // ========================================================================

    #[tool(
        description = "Search FDA Adverse Event Reporting System (FAERS) for adverse events by drug name and/or reaction."
    )]
    async fn faers_search(
        &self,
        Parameters(params): Parameters<params::faers::FaersSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers::search(params).await
    }

    #[tool(description = "Get top adverse events reported for a specific drug from FAERS.")]
    async fn faers_drug_events(
        &self,
        Parameters(params): Parameters<params::faers::FaersDrugEventsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers::drug_events(params).await
    }

    #[tool(
        description = "Quick signal detection check for a drug-event pair using Evans criteria (PRR≥2, χ²≥3.841, n≥3)."
    )]
    async fn faers_signal_check(
        &self,
        Parameters(params): Parameters<params::faers::FaersSignalParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers::signal_check(params).await
    }

    #[tool(
        description = "Full disproportionality analysis for a drug-event pair. Returns 2x2 table, PRR with CI, ROR with CI, chi-square."
    )]
    async fn faers_disproportionality(
        &self,
        Parameters(params): Parameters<params::faers::FaersSignalParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers::disproportionality(params).await
    }

    #[tool(
        description = "Compare adverse event profiles between two drugs. Shows common and unique events."
    )]
    async fn faers_compare_drugs(
        &self,
        Parameters(params): Parameters<params::faers::FaersCompareDrugsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers::compare_drugs(params).await
    }

    // ========================================================================
    // FAERS ETL Tools (4) — Local bulk data pipeline
    // ========================================================================

    #[tool(
        description = "Run full FAERS ETL pipeline on local quarterly ASCII data (~7GB RSS). Returns top signals by PRR. Requires faers_dir path to quarterly ASCII directory."
    )]
    async fn faers_etl_run(
        &self,
        Parameters(params): Parameters<params::faers::FaersEtlRunParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers_etl::run(params).await
    }

    #[tool(
        description = "Search FAERS ETL signals by drug and/or event name from local bulk data. Runs full pipeline then filters results."
    )]
    async fn faers_etl_signals(
        &self,
        Parameters(params): Parameters<params::faers::FaersEtlSignalsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers_etl::signals(params).await
    }

    #[tool(
        description = "Validate known drug-event pairs against local FAERS data. Returns hit rate and per-pair signal metrics."
    )]
    async fn faers_etl_known_pairs(
        &self,
        Parameters(params): Parameters<params::faers::FaersEtlKnownPairsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers_etl::known_pairs(params).await
    }

    #[tool(
        description = "Check status of cached FAERS Parquet output files (sizes, modification times). Lightweight filesystem check only."
    )]
    fn faers_etl_status(
        &self,
        Parameters(params): Parameters<params::faers::FaersEtlStatusParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers_etl::status(params)
    }

    // ========================================================================
    // PHAROS Tools (Autonomous Signal Surveillance)
    // ========================================================================

    #[tool(
        description = "Run the full PHAROS surveillance pipeline: FAERS ETL → signal detection (PRR/ROR/IC/EBGM) → threshold filtering → report. Returns actionable signals with threat levels. Heavy operation (~30-60s)."
    )]
    async fn pharos_run(
        &self,
        Parameters(params): Parameters<params::PharosRunParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pharos::run(params).await
    }

    #[tool(
        description = "Check PHAROS status: existing surveillance reports, Parquet output, and timer state."
    )]
    fn pharos_status(
        &self,
        Parameters(params): Parameters<params::PharosStatusParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pharos::status(params)
    }

    #[tool(
        description = "Retrieve a specific or latest PHAROS surveillance report by run_id. Returns full JSON report with top signals, metrics, and threat levels."
    )]
    fn pharos_report(
        &self,
        Parameters(params): Parameters<params::PharosReportParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::pharos::report(params)
    }

    // ========================================================================
    // FAERS Analytics Tools (A77, A80, A82 — Novel Signal Detection)
    // ========================================================================

    #[tool(
        description = "Algorithm A82: Outcome-Conditioned Signal Strength. Adjusts standard PRR by reaction outcome severity (Fatal=5x, Not Recovered=3x, Recovered=0x). Exploits previously unused FAERS reactionoutcome field. Provide cases with drug, event, and outcome_code (1-6). P0 patient safety: separates fatal signals from transient ADRs."
    )]
    fn faers_outcome_conditioned(
        &self,
        Parameters(params): Parameters<params::faers::FaersOutcomeConditionedParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers_analytics::outcome_conditioned(params)
    }

    #[tool(
        description = "Algorithm A77: Signal Velocity Detector. Detects EMERGING signals by measuring temporal acceleration in reporting frequency. Catches signals 6-18 months before they cross PRR thresholds. Provide cases with drug, event, and receipt_date (YYYYMMDD). Early warning system for pharmacovigilance."
    )]
    fn faers_signal_velocity(
        &self,
        Parameters(params): Parameters<params::faers::FaersSignalVelocityParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers_analytics::signal_velocity(params)
    }

    #[tool(
        description = "Algorithm A80: Seriousness Cascade Detector. Detects signals ESCALATING in severity using all 6 FAERS seriousness flags (death, hospitalization, disabling, congenital, life-threatening, other). P0 patient safety alarm: flags drug-event pairs where seriousness is trending upward even when case counts are stable. Requires immediate human review when death rate exceeds threshold."
    )]
    fn faers_seriousness_cascade(
        &self,
        Parameters(params): Parameters<params::faers::FaersSeriousnessCascadeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers_analytics::seriousness_cascade(params)
    }

    #[tool(
        description = "Algorithm A78: Polypharmacy Interaction Signal. Detects drug PAIRS with disproportionate co-occurrence signals for a given event. Computes pair PRR and compares to individual PRRs — positive interaction_signal indicates synergistic toxicity invisible to single-drug analysis. Input: cases with case_id, multiple drugs (with characterization codes), and event."
    )]
    fn faers_polypharmacy(
        &self,
        Parameters(params): Parameters<params::faers::FaersPolypharmacyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers_analytics::polypharmacy(params)
    }

    #[tool(
        description = "Algorithm A79: Reporter-Weighted Disproportionality. Weights each case by reporter qualification (Physician=1.0, Pharmacist=0.9, OtherHP=0.8, Consumer=0.6, Lawyer=0.5). Computes Shannon entropy for reporter diversity — multi-source confirmed signals have higher confidence. Input: cases with drug, event, and qualification_code (1-5)."
    )]
    fn faers_reporter_weighted(
        &self,
        Parameters(params): Parameters<params::faers::FaersReporterWeightedParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers_analytics::reporter_weighted(params)
    }

    #[tool(
        description = "Algorithm A81: Geographic Signal Divergence. Detects drug-event pairs with significantly different reporting rates across countries. Computes per-country rates, divergence ratio (max/min), and chi-squared heterogeneity test. Divergent signals may indicate pharmacogenomic effects, regulatory gaps, or reporting biases. Input: cases with drug, event, and country (ISO 2-letter)."
    )]
    fn faers_geographic_divergence(
        &self,
        Parameters(params): Parameters<params::faers::FaersGeographicDivergenceParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::faers_analytics::geographic_divergence(params)
    }

    // ========================================================================
    // NCBI Entrez Tools (6)
    // ========================================================================

    #[tool(
        description = "Search NCBI databases for UIDs matching a query. Supports date-range filtering (datetype, mindate, maxdate). Returns matching IDs, count, and query translation."
    )]
    async fn ncbi_esearch(
        &self,
        Parameters(params): Parameters<params::ncbi::NcbiSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::ncbi::esearch(params).await
    }

    #[tool(description = "Retrieve summaries for a list of NCBI UIDs from a specific database.")]
    async fn ncbi_esummary(
        &self,
        Parameters(params): Parameters<params::ncbi::NcbiSummaryParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::ncbi::esummary(params).await
    }

    #[tool(
        description = "Retrieve full records for a list of NCBI UIDs in various formats (e.g., FASTA, GenBank)."
    )]
    async fn ncbi_efetch(
        &self,
        Parameters(params): Parameters<params::ncbi::NcbiFetchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::ncbi::efetch(params).await
    }

    #[tool(
        description = "Find linked records across NCBI databases (cross-database traversal). E.g., find all nucleotide sequences linked to a gene ID, or PubMed articles linked to a protein."
    )]
    async fn ncbi_elink(
        &self,
        Parameters(params): Parameters<params::ncbi::NcbiLinkParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::ncbi::elink(params).await
    }

    #[tool(
        description = "Search an NCBI database then fetch FASTA records in one call. Returns parsed sequence data with ID, description, length, GC content, and first 60 bases."
    )]
    async fn ncbi_search_and_fetch(
        &self,
        Parameters(params): Parameters<params::ncbi::NcbiSearchFetchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::ncbi::search_and_fetch(params).await
    }

    #[tool(
        description = "Search an NCBI database then retrieve document summaries in one call. Faster than full fetch — returns metadata (titles, organisms, etc.) without downloading full sequences. Supports date-range filtering for PubMed."
    )]
    async fn ncbi_search_and_summarize(
        &self,
        Parameters(params): Parameters<params::ncbi::NcbiSearchSummarizeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::ncbi::search_and_summarize(params).await
    }

    // ========================================================================
    // Laboratory Tools (4)
    // ========================================================================

    #[tool(
        description = "Run a complete word/concept experiment. Decomposes to T1 primitives, computes Shannon molecular weight in daltons, classifies transfer tier, and produces spectral analysis of constituents."
    )]
    fn lab_experiment(
        &self,
        Parameters(params): Parameters<params::lab::LabExperimentParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::laboratory::lab_experiment(params)
    }

    #[tool(
        description = "Compare two concepts side-by-side. Returns both experiment results plus comparative metrics: weight delta, shared/unique primitives, Jaccard similarity, and transfer class alignment."
    )]
    fn lab_compare(
        &self,
        Parameters(params): Parameters<params::lab::LabCompareParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::laboratory::lab_compare(params)
    }

    #[tool(
        description = "React two concepts by combining their primitive compositions. Shared primitives are catalysts. Product is the union. Enthalpy (ΔH) measures weight efficiency — negative = exothermic (more compact than sum of parts)."
    )]
    fn lab_react(
        &self,
        Parameters(params): Parameters<params::lab::LabReactParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::laboratory::lab_react(params)
    }

    #[tool(
        description = "Run experiments on a batch of specimens with statistical summary. Returns individual results plus aggregate: lightest/heaviest, average weight, standard deviation, and transfer class distribution."
    )]
    fn lab_batch(
        &self,
        Parameters(params): Parameters<params::lab::LabBatchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::laboratory::lab_batch(params)
    }

    // ========================================================================
    // CEP Tools (6) — Cognitive Evolution Pipeline
    // ========================================================================

    #[tool(
        description = "Execute a single CEP (Cognitive Evolution Pipeline) stage. Stages: SEE (1), SPEAK (2), DECOMPOSE (3), COMPOSE (4), TRANSLATE (5), VALIDATE (6), DEPLOY (7), IMPROVE (8). Stage 8 feeds back to Stage 1."
    )]
    fn cep_execute_stage(
        &self,
        Parameters(params): Parameters<params::cep::CepExecuteStageParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cep::execute_stage(params)
    }

    #[tool(
        description = "Get all 8 CEP pipeline stages with descriptions, ordering, and validation thresholds (coverage >= 0.95, minimality >= 0.90, independence >= 0.90)."
    )]
    fn cep_pipeline_stages(&self) -> Result<CallToolResult, McpError> {
        tools::cep::pipeline_stages()
    }

    #[tool(
        description = "Validate primitive extraction quality against CEP thresholds. Checks coverage (>= 0.95), minimality (>= 0.90), and independence (>= 0.90). Returns pass/fail per metric and weakest metric."
    )]
    fn cep_validate_extraction(
        &self,
        Parameters(params): Parameters<params::cep::CepValidateExtractionParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cep::validate_extraction(params)
    }

    #[tool(
        description = "Extract T1/T2/T3 primitives from a domain. Returns extraction steps and methodology. Use /primitive-extractor skill for full LLM-assisted extraction."
    )]
    fn cep_extract_primitives(
        &self,
        Parameters(params): Parameters<params::cep::PrimitiveExtractParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cep::extract_primitives(params)
    }

    #[tool(
        description = "Translate a concept between domains using T1/T2/T3 tier-based mapping rules. T1 = identity mapping (1.0 confidence), T2 = similarity matching (0.85-0.95), T3 = novel synthesis (0.70-0.85)."
    )]
    fn cep_domain_translate(
        &self,
        Parameters(params): Parameters<params::cep::DomainTranslateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cep::domain_translate(params)
    }

    #[tool(
        description = "Classify a primitive's tier based on domain count. T1-Universal (>= 10 domains), T2-CrossDomain (2-9), T3-DomainSpecific (1). Returns tier, confidence, and minimum coverage threshold."
    )]
    fn cep_classify_primitive(
        &self,
        Parameters(params): Parameters<params::cep::PrimitiveTierClassifyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cep::classify_primitive(params.domain_count)
    }

    #[tool(description = "Perform a real-time network scan for a behavioral pattern")]
    async fn node_hunt_scan(
        &self,
        Parameters(params): Parameters<params::network::NodeHuntScanParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::node_hunter::scan(params)
    }

    #[tool(description = "Isolate a specific node from the network")]
    async fn node_hunt_isolate(
        &self,
        Parameters(params): Parameters<params::network::NodeHuntIsolateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::node_hunter::isolate(params)
    }

    // ========================================================================
    // GCloud Tools (19)
    // ========================================================================

    #[tool(description = "List authenticated gcloud accounts.")]
    async fn gcloud_auth_list(&self) -> Result<CallToolResult, McpError> {
        tools::gcloud::auth_list().await
    }

    #[tool(description = "List current gcloud configuration (project, account, region, zone).")]
    async fn gcloud_config_list(&self) -> Result<CallToolResult, McpError> {
        tools::gcloud::config_list().await
    }

    #[tool(description = "Get a specific gcloud configuration property.")]
    async fn gcloud_config_get(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudConfigGetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::config_get(&params.property).await
    }

    #[tool(description = "Set a gcloud configuration property.")]
    async fn gcloud_config_set(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudConfigSetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::config_set(&params.property, &params.value).await
    }

    #[tool(description = "List all accessible GCP projects.")]
    async fn gcloud_projects_list(&self) -> Result<CallToolResult, McpError> {
        tools::gcloud::projects_list().await
    }

    #[tool(description = "Get details about a specific GCP project.")]
    async fn gcloud_projects_describe(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudProjectParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::projects_describe(&params.project_id).await
    }

    #[tool(description = "Get IAM policy for a GCP project.")]
    async fn gcloud_projects_get_iam_policy(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudProjectParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::projects_get_iam_policy(&params.project_id).await
    }

    #[tool(description = "List secrets in Secret Manager.")]
    async fn gcloud_secrets_list(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudOptionalProjectParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::secrets_list(params.project.as_deref()).await
    }

    #[tool(description = "Access a secret version's value from Secret Manager.")]
    async fn gcloud_secrets_versions_access(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudSecretsAccessParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::secrets_versions_access(
            &params.secret_name,
            &params.version,
            params.project.as_deref(),
        )
        .await
    }

    #[tool(description = "List Cloud Storage buckets.")]
    async fn gcloud_storage_buckets_list(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudOptionalProjectParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::storage_buckets_list(params.project.as_deref()).await
    }

    #[tool(description = "List objects in a Cloud Storage path.")]
    async fn gcloud_storage_ls(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudStoragePathParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::storage_ls(&params.path).await
    }

    #[tool(description = "Copy files to/from Cloud Storage.")]
    async fn gcloud_storage_cp(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudStorageCpParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::storage_cp(&params.source, &params.destination, params.recursive).await
    }

    #[tool(description = "List Compute Engine instances.")]
    async fn gcloud_compute_instances_list(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudComputeInstancesParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::compute_instances_list(params.project.as_deref(), params.zone.as_deref())
            .await
    }

    #[tool(description = "List Cloud Run services.")]
    async fn gcloud_run_services_list(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudServiceListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::run_services_list(params.project.as_deref(), params.region.as_deref()).await
    }

    #[tool(description = "Get details about a Cloud Run service.")]
    async fn gcloud_run_services_describe(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudServiceDescribeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::run_services_describe(
            &params.name,
            &params.region,
            params.project.as_deref(),
        )
        .await
    }

    #[tool(description = "List Cloud Functions.")]
    async fn gcloud_functions_list(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudServiceListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::functions_list(params.project.as_deref(), params.region.as_deref()).await
    }

    #[tool(description = "List service accounts.")]
    async fn gcloud_iam_service_accounts_list(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudOptionalProjectParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::iam_service_accounts_list(params.project.as_deref()).await
    }

    #[tool(description = "Read log entries from Cloud Logging.")]
    async fn gcloud_logging_read(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudLoggingReadParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::logging_read(&params.filter, params.limit, params.project.as_deref()).await
    }

    #[tool(
        description = "Run an arbitrary gcloud command (with safety checks for destructive operations)."
    )]
    async fn gcloud_run_command(
        &self,
        Parameters(params): Parameters<params::gcloud::GcloudRunCommandParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gcloud::run_command(&params.command, params.timeout).await
    }

    // ========================================================================
    // Wolfram Alpha Tools (19)
    // ========================================================================

    #[tool(
        description = "Query Wolfram Alpha's computational knowledge engine with full results. Returns comprehensive information organized in pods. Use for: math, science, geography, history, linguistics, music, astronomy, engineering, medicine, finance, sports, food/nutrition, and more. Examples: 'integrate x^2 from 0 to 1', 'distance Earth to Mars', 'GDP of France vs Germany', 'ISS location', 'weather in Tokyo'"
    )]
    async fn wolfram_query(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframQueryParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::query(params).await
    }

    #[tool(
        description = "Get a concise, single-line answer from Wolfram Alpha. Best for quick facts, simple calculations, and direct questions. More efficient than full query when you just need the answer. Examples: 'population of Japan', 'boiling point of water', '100 miles in km', 'how many days until Christmas'"
    )]
    async fn wolfram_short_answer(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframShortParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::short_answer(params).await
    }

    #[tool(
        description = "Get a natural language answer suitable for speaking aloud. Returns human-readable sentences rather than data tables. Ideal for voice interfaces, accessibility, or conversational responses. Examples: 'What is the speed of light?', 'How far is the moon?', 'What is the derivative of x cubed?'"
    )]
    async fn wolfram_spoken_answer(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframShortParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::spoken_answer(params).await
    }

    #[tool(
        description = "Perform mathematical calculations with Wolfram Alpha. Supports arithmetic, algebra, calculus, linear algebra, statistics, number theory, discrete math, and more. Examples: 'solve x^2 + 2x - 3 = 0', 'integral of sin(x)cos(x)', 'determinant of {{1,2},{3,4}}', 'sum of 1/n^2 from n=1 to infinity', 'factor 123456789', 'derivative of e^(x^2)'"
    )]
    async fn wolfram_calculate(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframCalculateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::calculate(params).await
    }

    #[tool(
        description = "Solve math problems with detailed step-by-step explanations. Shows the work and reasoning for educational purposes. Best for: solving equations, derivatives, integrals, limits, simplification, factoring, and algebraic manipulations. Examples: 'solve 2x + 5 = 13 step by step', 'integrate x*e^x step by step', 'factor x^2 - 5x + 6'"
    )]
    async fn wolfram_step_by_step(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframStepByStepParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::step_by_step(params).await
    }

    #[tool(
        description = "Generate mathematical plots and graphs. Returns image URL for the visualization. Supports 2D plots, 3D surfaces, parametric curves, implicit plots, etc. Examples: 'plot sin(x) from -2pi to 2pi', '3D plot of x^2 + y^2', 'plot x^2, x^3, x^4 together', 'parametric plot (cos(t), sin(t)) for t from 0 to 2pi'"
    )]
    async fn wolfram_plot(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframPlotParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::plot(params).await
    }

    #[tool(
        description = "Convert between units of measurement with high precision. Supports all physical units: length, mass, volume, temperature, speed, energy, pressure, area, time, data sizes, currency, and more. Examples: '100 mph to km/h', '72 fahrenheit to celsius', '1 lightyear to km', '500 MB to GB', '1 acre to square meters'"
    )]
    async fn wolfram_convert(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframConvertParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::convert(params).await
    }

    #[tool(
        description = "Look up comprehensive chemical compound information. Returns molecular formula, structure, properties, safety data, thermodynamic data, and more. Accepts: compound names, molecular formulas, SMILES, CAS numbers. Examples: 'caffeine', 'H2SO4', 'aspirin properties', 'CAS 50-78-2', 'ethanol boiling point'"
    )]
    async fn wolfram_chemistry(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframChemistryParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::chemistry(params).await
    }

    #[tool(
        description = "Query physics constants, formulas, and calculations. Includes mechanics, electromagnetism, thermodynamics, quantum physics, relativity, optics, and astrophysics. Examples: 'speed of light', 'gravitational constant', 'kinetic energy of 10 kg at 5 m/s', 'wavelength of red light', 'escape velocity of Earth', 'Schwarzschild radius of the Sun'"
    )]
    async fn wolfram_physics(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframPhysicsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::physics(params).await
    }

    #[tool(
        description = "Query astronomical data: planets, stars, galaxies, satellites, events. Real-time positions, rise/set times, orbital data, and more. Examples: 'ISS location', 'Mars distance from Earth', 'next solar eclipse', 'Andromeda galaxy', 'sunrise tomorrow in London', 'Jupiter moons', 'Hubble Space Telescope orbit'"
    )]
    async fn wolfram_astronomy(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframAstronomyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::astronomy(params).await
    }

    #[tool(
        description = "Perform statistical calculations and analysis. Includes descriptive statistics, probability distributions, hypothesis testing, regression, and more. Examples: 'mean of {1, 2, 3, 4, 5}', 'standard deviation of {10, 12, 15, 18, 20}', 'normal distribution mean=100 sd=15', 'P(X > 2) for Poisson(3)', 'linear regression {{1,2},{2,4},{3,5},{4,4},{5,5}}'"
    )]
    async fn wolfram_statistics(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframStatisticsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::statistics(params).await
    }

    #[tool(
        description = "Look up real-world data: demographics, economics, geography, etc. Includes countries, cities, companies, historical data, rankings. Examples: 'population of Tokyo', 'GDP of Germany', 'tallest buildings in the world', 'US unemployment rate', 'distance from New York to London', 'area of Texas'"
    )]
    async fn wolfram_data_lookup(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframDataLookupParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::data_lookup(params).await
    }

    #[tool(
        description = "Query Wolfram Alpha with a specific interpretation for ambiguous terms. Use when a previous query returned multiple possible interpretations. Examples: query 'Mercury' with assumption for planet vs element, query 'pi' with assumption for mathematical constant vs movie"
    )]
    async fn wolfram_query_with_assumption(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframQueryWithAssumptionParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::query_with_assumption(params).await
    }

    #[tool(
        description = "Query Wolfram Alpha with pod filtering to get specific result types. Reduces noise by including only relevant pods or excluding unwanted ones. Common pod IDs: Result, Input, Plot, Solution, Properties, Definition, BasicInformation, Timeline, Notable facts"
    )]
    async fn wolfram_query_filtered(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframQueryFilteredParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::query_filtered(params).await
    }

    #[tool(
        description = "Get a visual/image result from Wolfram Alpha. Returns a URL to an image containing the full rendered result. Useful for complex visualizations, formulas, and graphical data. Examples: 'anatomy of the heart', 'world map with time zones', 'periodic table', 'human skeleton'"
    )]
    async fn wolfram_image_result(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframImageParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::image_result(params).await
    }

    #[tool(
        description = "Calculate dates, times, durations, and time zones. Examples: 'days until December 25 2025', 'time in Tokyo', 'what day was July 4 1776', '90 days from today', 'duration from Jan 1 2020 to today', 'next full moon'"
    )]
    async fn wolfram_datetime(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframDatetimeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::datetime(params).await
    }

    #[tool(
        description = "Look up nutritional information for foods. Returns calories, macros, vitamins, minerals, and more. Examples: 'nutrition facts for apple', '100g chicken breast calories', 'compare pizza vs salad nutrition', 'vitamin C in orange'"
    )]
    async fn wolfram_nutrition(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframNutritionParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::nutrition(params).await
    }

    #[tool(
        description = "Financial calculations and market data. Includes currency conversion, loan calculations, investment growth, stock data, economic indicators. Examples: '500 USD to EUR', 'mortgage payment 300000 at 6% for 30 years', 'compound interest 10000 at 5% for 10 years', 'inflation adjusted 1000 from 1990 to today'"
    )]
    async fn wolfram_finance(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframFinanceParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::finance(params).await
    }

    #[tool(
        description = "Language and word information: definitions, etymology, translations, word frequency, anagrams, rhymes, and more. Examples: 'define ubiquitous', 'etymology of algorithm', 'translate hello to Japanese', 'anagrams of listen', 'words that rhyme with time', 'Scrabble score for quizzify'"
    )]
    async fn wolfram_linguistics(
        &self,
        Parameters(params): Parameters<params::wolfram::WolframLinguisticsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::wolfram::linguistics(params).await
    }

    // ========================================================================
    // Perplexity AI Search Tools (4)
    // ========================================================================

    #[tool(
        description = "Search the web using Perplexity AI's Sonar API. Returns search-grounded answers with citations. Supports model selection (sonar=fast, sonar-pro=deep, sonar-deep-research=multi-step), recency filtering (hour/day/week/month), and domain filtering. Set PERPLEXITY_API_KEY env var. Examples: 'latest FDA drug approvals 2026', 'Rust async runtime comparison'"
    )]
    async fn perplexity_search(
        &self,
        Parameters(params): Parameters<params::perplexity::PerplexitySearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::perplexity::search(params).await
    }

    #[tool(
        description = "High-level research routing. Specify use_case: 'general' (open web research), 'competitive' (market/competitor intel with SonarPro), or 'regulatory' (FDA/EMA/ICH/WHO filtered). Each use case applies specialized system prompts and domain filters."
    )]
    async fn perplexity_research(
        &self,
        Parameters(params): Parameters<params::perplexity::PerplexityResearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::perplexity::research(params).await
    }

    #[tool(
        description = "Competitive intelligence search filtered to specified competitor domains. Uses SonarPro model with month recency filter. Provide competitor domain names (e.g., ['competitor1.com', 'competitor2.io']). Focuses on market positioning, pricing, announcements, partnerships."
    )]
    async fn perplexity_competitive(
        &self,
        Parameters(params): Parameters<params::perplexity::PerplexityCompetitiveParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::perplexity::competitive(params).await
    }

    #[tool(
        description = "Regulatory intelligence search pre-filtered to pharmaceutical/healthcare regulatory domains: fda.gov, ema.europa.eu, ich.org, who.int, clinicaltrials.gov, drugs.com, drugbank.com, pmda.go.jp. Uses SonarPro with month recency. For FDA actions, EMA guidelines, ICH harmonization, WHO reports."
    )]
    async fn perplexity_regulatory(
        &self,
        Parameters(params): Parameters<params::perplexity::PerplexityRegulatoryParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::perplexity::regulatory(params).await
    }

    // ========================================================================
    // Principles Knowledge Base Tools (3)
    // ========================================================================

    #[tool(
        description = "List all available principles in the knowledge base. Returns principle IDs, titles, and file paths. Includes Ray Dalio's Principles, KISS, First Principles, etc."
    )]
    async fn principles_list(
        &self,
        Parameters(params): Parameters<params::principles::PrinciplesListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::principles::list_principles(params)
    }

    #[tool(
        description = "Get a specific principle by name. Returns the full markdown content. Use 'dalio-principles' for Ray Dalio's decision-making framework, 'kiss' for Keep It Simple, 'first-principles' for foundational reasoning."
    )]
    async fn principles_get(
        &self,
        Parameters(params): Parameters<params::principles::PrinciplesGetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::principles::get_principle(params)
    }

    #[tool(
        description = "Search principles by keyword. Returns matching sections with context. Use for finding relevant decision-making guidance. Examples: 'open-minded', 'meritocracy', 'believability', 'expected value'"
    )]
    async fn principles_search(
        &self,
        Parameters(params): Parameters<params::principles::PrinciplesSearchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::principles::search_principles(params)
    }

    // ========================================================================
    // Universal Validation Tools
    // ========================================================================

    #[tool(
        description = "Run L1-L5 validation on a target. Validates coherence (L1), structure (L2), function (L3), operations (L4), and impact (L5). Auto-detects domain or specify explicitly. Returns detailed check results per level."
    )]
    async fn validation_run(
        &self,
        Parameters(params): Parameters<params::validation::ValidationRunParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::validation::run(params)
    }

    #[tool(
        description = "Quick L1-L2 validation check. Fast coherence and structural validation only. Use for rapid sanity checks before full validation."
    )]
    async fn validation_check(
        &self,
        Parameters(params): Parameters<params::validation::ValidationCheckParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::validation::check(params)
    }

    #[tool(
        description = "List available validation domains and L1-L5 level definitions. Shows detection patterns for each domain."
    )]
    async fn validation_domains(
        &self,
        Parameters(params): Parameters<params::validation::ValidationDomainsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::validation::domains(params)
    }

    #[tool(
        description = "Classify tests in Rust source into 5 categories: Positive (happy path), Negative (error handling), Edge (boundary conditions), Stress (performance), Adversarial (security). Returns distribution, coverage gaps, and recommendations."
    )]
    async fn validation_classify_tests(
        &self,
        Parameters(params): Parameters<params::validation::ValidationClassifyTestsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::validation::classify_tests_tool(params)
    }

    // ========================================================================
    // Brain Tools (12) - Antigravity-style working memory
    // ========================================================================

    #[tool(
        description = "Create a new brain session for working memory. Returns session ID. Sessions store artifacts (task.md, plan.md) with versioning."
    )]
    async fn brain_session_create(
        &self,
        Parameters(params): Parameters<params::brain::BrainSessionCreateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::session_create(params)
    }

    #[tool(
        description = "Load an existing brain session by ID. Returns session details and artifact list."
    )]
    async fn brain_session_load(
        &self,
        Parameters(params): Parameters<params::brain::BrainSessionLoadParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::session_load(params)
    }

    #[tool(
        description = "List all brain sessions, most recent first. Returns session IDs, dates, and projects."
    )]
    async fn brain_sessions_list(
        &self,
        Parameters(params): Parameters<params::brain::BrainSessionsListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::sessions_list(params)
    }

    #[tool(
        description = "Save an artifact to a brain session. Artifact types: task, plan, walkthrough, review, research, decision, custom."
    )]
    async fn brain_artifact_save(
        &self,
        Parameters(params): Parameters<params::brain::BrainArtifactSaveParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::artifact_save(params)
    }

    #[tool(
        description = "Resolve an artifact, creating an immutable snapshot. Creates .resolved and .resolved.N files. Returns version number."
    )]
    async fn brain_artifact_resolve(
        &self,
        Parameters(params): Parameters<params::brain::BrainArtifactResolveParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::artifact_resolve(params)
    }

    #[tool(
        description = "Get an artifact's content. Specify version for a specific resolved snapshot, or omit for current mutable state."
    )]
    async fn brain_artifact_get(
        &self,
        Parameters(params): Parameters<params::brain::BrainArtifactGetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::artifact_get(params)
    }

    #[tool(
        description = "Diff two resolved versions of an artifact. Returns line-based diff showing additions and removals."
    )]
    async fn brain_artifact_diff(
        &self,
        Parameters(params): Parameters<params::brain::BrainArtifactDiffParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::artifact_diff(params)
    }

    #[tool(
        description = "Track a file for content-addressable change detection. Stores SHA-256 hash and file copy."
    )]
    async fn code_tracker_track(
        &self,
        Parameters(params): Parameters<params::brain::BrainCodeTrackerTrackParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::code_tracker_track(params)
    }

    #[tool(
        description = "Check if a tracked file has changed since it was tracked. Returns changed: true/false."
    )]
    async fn code_tracker_changed(
        &self,
        Parameters(params): Parameters<params::brain::BrainCodeTrackerChangedParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::code_tracker_changed(params)
    }

    #[tool(description = "Get the original content of a tracked file when it was first tracked.")]
    async fn code_tracker_original(
        &self,
        Parameters(params): Parameters<params::brain::BrainCodeTrackerOriginalParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::code_tracker_original(params)
    }

    #[tool(
        description = "Get a learned preference from implicit knowledge. Returns value and confidence score."
    )]
    async fn implicit_get(
        &self,
        Parameters(params): Parameters<params::brain::BrainImplicitGetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::implicit_get(params)
    }

    #[tool(
        description = "Set a learned preference in implicit knowledge. Value should be JSON. Preferences are reinforced with repeated use."
    )]
    async fn implicit_set(
        &self,
        Parameters(params): Parameters<params::brain::BrainImplicitSetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::implicit_set(params)
    }

    #[tool(
        description = "Get implicit knowledge statistics including total patterns, corrections, preferences, decayed patterns (confidence < 0.5 after time-decay), and ungrounded patterns (no T1 primitive assigned)."
    )]
    async fn implicit_stats(&self) -> Result<CallToolResult, McpError> {
        tools::brain::implicit_stats()
    }

    #[tool(
        description = "Find corrections using fuzzy token matching (Jaccard similarity). Returns corrections above the similarity threshold. Default threshold: 0.3."
    )]
    async fn implicit_find_corrections(
        &self,
        Parameters(params): Parameters<params::brain::BrainImplicitFindCorrectionsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::implicit_find_corrections(params)
    }

    #[tool(
        description = "List patterns filtered by T1 primitive grounding (sequence, mapping, recursion, state, void). Returns patterns with their decay-adjusted effective confidence."
    )]
    async fn implicit_patterns_by_grounding(
        &self,
        Parameters(params): Parameters<params::brain::BrainImplicitPatternsByGroundingParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::implicit_patterns_by_grounding(params)
    }

    #[tool(
        description = "List all patterns sorted by decay-adjusted relevance (effective confidence descending). Shows which patterns are still strong vs fading."
    )]
    async fn implicit_patterns_by_relevance(&self) -> Result<CallToolResult, McpError> {
        tools::brain::implicit_patterns_by_relevance()
    }

    #[tool(
        description = "Check brain health status. Returns overall status (healthy/degraded), index health, and partial writes."
    )]
    async fn brain_recovery_check(&self) -> Result<CallToolResult, McpError> {
        tools::brain::recovery_check()
    }

    #[tool(
        description = "Repair partial writes by creating missing metadata for artifacts. Operates on specified session or latest."
    )]
    async fn brain_recovery_repair(
        &self,
        Parameters(params): Parameters<params::brain::BrainRecoveryRepairParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::recovery_repair(params.session_id.as_deref())
    }

    #[tool(
        description = "Rebuild session index from session directories. Use when index is corrupted or missing."
    )]
    async fn brain_recovery_rebuild_index(&self) -> Result<CallToolResult, McpError> {
        tools::brain::recovery_rebuild_index()
    }

    #[tool(
        description = "Attempt automatic brain recovery. Checks health and repairs if auto_recovery is enabled."
    )]
    async fn brain_recovery_auto(&self) -> Result<CallToolResult, McpError> {
        tools::brain::recovery_auto()
    }

    // ========================================================================
    // Brain Coordination Tools (ACS)
    // ========================================================================

    #[tool(
        description = "Acquire a file lock for agent coordination. Prevents race conditions in multi-agent environments. Returns success status and lock info. Idempotent."
    )]
    async fn brain_coordination_acquire(
        &self,
        Parameters(params): Parameters<params::brain::BrainCoordinationAcquireParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::coordination_acquire(params)
    }

    #[tool(
        description = "Release a file lock held by an agent. Allows other agents to access the file. Returns success status."
    )]
    async fn brain_coordination_release(
        &self,
        Parameters(params): Parameters<params::brain::BrainCoordinationReleaseParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::coordination_release(params)
    }

    #[tool(
        description = "Check the lock status of a file. Returns Vacant or Occupied with agent info."
    )]
    async fn brain_coordination_status(
        &self,
        Parameters(params): Parameters<params::brain::BrainCoordinationStatusParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brain::coordination_status(params)
    }

    // ========================================================================
    // Hooks Tools (3) - Hook Registry API
    // ========================================================================

    #[tool(
        description = "Get hook registry statistics. Returns total hooks, counts by tier (dev/review/deploy), and counts by event type. Provides structured overview of the hook system."
    )]
    async fn hooks_stats(&self) -> Result<CallToolResult, McpError> {
        tools::hooks::stats()
    }

    #[tool(
        description = "Get hooks for a specific event type. Returns list of hooks that trigger on the given event (e.g., 'SessionStart', 'PreToolUse:Edit|Write'). Each hook includes name, tiers, timeout, and description."
    )]
    async fn hooks_for_event(
        &self,
        Parameters(params): Parameters<params::hooks::HooksForEventParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hooks::for_event(params)
    }

    #[tool(
        description = "Get hooks for a specific deployment tier. Returns hooks active in the given tier: 'dev' (11 fast hooks), 'review' (33 quality hooks), or 'deploy' (76 full validation hooks). Use this to understand which hooks run in different environments."
    )]
    async fn hooks_for_tier(
        &self,
        Parameters(params): Parameters<params::hooks::HooksForTierParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hooks::for_tier(params)
    }

    #[tool(
        description = "List nested hooks for a compound hook (hook molecule). Returns molecular formula, nested hooks, bond strengths, and stability status. Use to inspect compound hook architecture."
    )]
    async fn hook_list_nested(
        &self,
        Parameters(params): Parameters<params::hooks::HookListNestedParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hooks::list_nested(params)
    }

    #[tool(
        description = "Get aggregate hook execution metrics summary. Returns total executions, block/warn counts, per-hook timing (p50/p95/p99), event distribution, and slowest hooks. Use to monitor hook performance and block rates."
    )]
    async fn hooks_metrics_summary(
        &self,
        Parameters(params): Parameters<params::hooks::HookMetricsSummaryParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hooks::metrics_summary(params)
    }

    #[tool(
        description = "Get hook execution metrics filtered by event type. Returns per-hook statistics only for the specified event (e.g., PreToolUse, SessionStart). Use to analyze hook behavior for specific lifecycle events."
    )]
    async fn hooks_metrics_by_event(
        &self,
        Parameters(params): Parameters<params::hooks::HookMetricsByEventParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::hooks::metrics_by_event(params)
    }

    // ========================================================================
    // Immunity Tools (6) - Antipattern Detection and Self-Regulation
    // ========================================================================

    #[tool(
        description = "Scan code content for known antipatterns. Returns threats with antibody IDs, severity, confidence, and response actions. Use before writing code to detect PAMPs/DAMPs."
    )]
    async fn immunity_scan(
        &self,
        Parameters(params): Parameters<params::immunity::ImmunityScanParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::immunity::immunity_scan(params)
    }

    #[tool(
        description = "Scan stderr output for known error patterns. Use after build/test failures to identify recurring antipatterns from compiler or tool output."
    )]
    async fn immunity_scan_errors(
        &self,
        Parameters(params): Parameters<params::immunity::ImmunityScanErrorsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::immunity::immunity_scan_errors(params)
    }

    #[tool(
        description = "List all registered antibodies with optional filtering by threat type (PAMP/DAMP) and minimum severity (low/medium/high/critical)."
    )]
    async fn immunity_list(
        &self,
        Parameters(params): Parameters<params::immunity::ImmunityListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::immunity::immunity_list(params)
    }

    #[tool(
        description = "Get detailed antibody by ID. Returns detection patterns, response strategy, Rust template, confidence, and learned_from context."
    )]
    async fn immunity_get(
        &self,
        Parameters(params): Parameters<params::immunity::ImmunityGetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::immunity::immunity_get(params)
    }

    #[tool(
        description = "Propose new antibody from observed error/fix pair. Creates proposal for review at ~/.claude/immunity/proposals.yaml. Use after fixing a recurring bug."
    )]
    async fn immunity_propose(
        &self,
        Parameters(params): Parameters<params::immunity::ImmunityProposeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::immunity::immunity_propose(params)
    }

    #[tool(
        description = "Get immunity system status. Returns registry version, antibody counts by type/severity, and homeostasis loop status."
    )]
    async fn immunity_status(&self) -> Result<CallToolResult, McpError> {
        tools::immunity::immunity_status()
    }

    // ========================================================================
    // Regulatory Primitives Tools (3)
    // ========================================================================

    #[tool(
        description = "Extract regulatory primitives from FDA/ICH/CIOMS sources. Classifies terms into T1 (Universal), T2-P (Cross-Domain Primitive), T2-C (Cross-Domain Composite), or T3 (Domain-Specific). Returns primitive inventory with dependency graphs and source citations."
    )]
    async fn regulatory_primitives_extract(
        &self,
        Parameters(params): Parameters<params::regulatory::RegulatoryExtractParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::regulatory::extract(params)
    }

    #[tool(
        description = "Audit FDA vs CIOMS/ICH term consistency. Computes structural match (word overlap), semantic match (concept alignment), identifies discrepancies, and returns verdict: Aligned, MinorDeviation, MajorDeviation, or NoCorrespondence."
    )]
    async fn regulatory_primitives_audit(
        &self,
        Parameters(params): Parameters<params::regulatory::RegulatoryAuditParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::regulatory::audit(params)
    }

    #[tool(
        description = "Cross-domain transfer analysis between regulatory domains. Maps primitives from one domain (PV, Cloud, AI Safety, Finance) to another, computing transfer confidence scores."
    )]
    async fn regulatory_primitives_compare(
        &self,
        Parameters(params): Parameters<params::regulatory::RegulatoryCompareParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::regulatory::compare(params)
    }

    // ========================================================================
    // Brand Semantics Tools (3) - Primitive Extraction for Brand Names
    // ========================================================================

    #[tool(
        description = "Get the pre-computed NexVigilant brand decomposition. Returns etymology roots, tier-classified primitives (T1/T2-P/T2-C/T3), primitive tests, transfer mappings to aviation domain, and semantic synthesis."
    )]
    async fn brand_decomposition_nexvigilant(&self) -> Result<CallToolResult, McpError> {
        tools::brand_semantics::brand_decomposition_nexvigilant()
    }

    #[tool(
        description = "Get a brand decomposition by name. Currently supports 'nexvigilant'. Returns full primitive extraction with tier classification and cross-domain transfer mappings."
    )]
    async fn brand_decomposition_get(
        &self,
        Parameters(params): Parameters<params::BrandDecompositionGetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brand_semantics::brand_decomposition_get(params)
    }

    #[tool(
        description = "Test if a term is a primitive using the 3-part test: (1) No domain-internal dependencies, (2) Grounds to external concepts, (3) Not merely a synonym. Returns verdict (Primitive/Composite) and tier classification (T1/T2-P/T2-C/T3)."
    )]
    async fn brand_primitive_test(
        &self,
        Parameters(params): Parameters<params::BrandPrimitiveTestParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::brand_semantics::brand_primitive_test(params)
    }

    #[tool(
        description = "List all semantic tiers (T1-Universal, T2-P-CrossDomainPrimitive, T2-C-CrossDomainComposite, T3-DomainSpecific) with definitions, examples, and transfer guidance."
    )]
    async fn brand_semantic_tiers(&self) -> Result<CallToolResult, McpError> {
        tools::brand_semantics::brand_semantic_tiers()
    }

    // ========================================================================
    // Primitive Validation Tools (4) - Corpus-Backed with Professional Citations
    // ========================================================================

    #[tool(
        description = "Validate a primitive term against external corpus (ICH glossary, PubMed). Returns validation tier (1=Authoritative/2=Peer-Reviewed/3=Web/4=Expert), confidence score, definition, and professional citations in Vancouver format. Essential for regulatory compliance."
    )]
    async fn primitive_validate(
        &self,
        Parameters(params): Parameters<params::PrimitiveValidateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::primitive_validation::validate(params).await
    }

    #[tool(
        description = "Generate a professional Vancouver-format citation for a PubMed ID (PMID) or DOI. Returns formatted citation with all metadata for regulatory documentation."
    )]
    async fn primitive_cite(
        &self,
        Parameters(params): Parameters<params::PrimitiveCiteParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::primitive_validation::cite(params).await
    }

    #[tool(
        description = "Batch validate multiple primitive terms. Returns validation summary with tier distribution, confidence scores, and aggregate statistics."
    )]
    async fn primitive_validate_batch(
        &self,
        Parameters(params): Parameters<params::PrimitiveValidateBatchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::primitive_validation::validate_batch(params).await
    }

    #[tool(
        description = "List validation tiers and their confidence levels. Shows Tier 1 (Authoritative: ICH/FDA/EMA), Tier 2 (Peer-Reviewed: PubMed), Tier 3 (Validated Web), and Tier 4 (Expert Generation). Includes regulatory compliance notes."
    )]
    async fn primitive_validation_tiers(&self) -> Result<CallToolResult, McpError> {
        tools::primitive_validation::validation_tiers()
    }

    // ========================================================================
    // Chemistry Primitives Tools (10) - Cross-Domain Transfer
    // ========================================================================

    #[tool(
        description = "Calculate Arrhenius rate constant (threshold gating). Maps to signal_detection_threshold in PV. PV transfer confidence: 0.92. k = A × e^(-Ea/RT)"
    )]
    async fn chemistry_threshold_rate(
        &self,
        Parameters(params): Parameters<params::ChemistryThresholdRateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::threshold_rate(params)
    }

    #[tool(
        description = "Calculate remaining amount after decay (half-life kinetics). Maps to signal_persistence in PV. PV transfer confidence: 0.90. N(t) = N₀ × e^(-kt)"
    )]
    async fn chemistry_decay_remaining(
        &self,
        Parameters(params): Parameters<params::ChemistryDecayRemainingParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::decay_remaining(params)
    }

    #[tool(
        description = "Calculate Michaelis-Menten saturation rate. Maps to case_processing_capacity in PV. PV transfer confidence: 0.88. v = Vmax × [S] / (Km + [S])"
    )]
    async fn chemistry_saturation_rate(
        &self,
        Parameters(params): Parameters<params::ChemistrySaturationRateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::saturation_rate(params)
    }

    #[tool(
        description = "Calculate Gibbs free energy feasibility. Maps to causality_likelihood in PV. PV transfer confidence: 0.85. ΔG = ΔH - TΔS"
    )]
    async fn chemistry_feasibility(
        &self,
        Parameters(params): Parameters<params::ChemistryFeasibilityParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::feasibility(params)
    }

    #[tool(
        description = "Calculate rate law dependency. Maps to signal_dependency in PV. PV transfer confidence: 0.82. rate = k[A]^n[B]^m"
    )]
    async fn chemistry_dependency_rate(
        &self,
        Parameters(params): Parameters<params::ChemistryDependencyRateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::dependency_rate(params)
    }

    #[tool(
        description = "Calculate buffer capacity (Henderson-Hasselbalch). Maps to baseline_stability in PV. PV transfer confidence: 0.78. β = 2.303 × C × ratio / (1+ratio)²"
    )]
    async fn chemistry_buffer_capacity(
        &self,
        Parameters(params): Parameters<params::ChemistryBufferCapacityParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::buffer_cap(params)
    }

    #[tool(
        description = "Calculate Beer-Lambert absorbance. Maps to dose_response_linearity in PV. PV transfer confidence: 0.75. A = εlc"
    )]
    async fn chemistry_signal_absorbance(
        &self,
        Parameters(params): Parameters<params::ChemistrySignalAbsorbanceParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::signal_absorbance(params)
    }

    #[tool(
        description = "Calculate equilibrium steady-state fractions. Maps to reporting_baseline in PV. PV transfer confidence: 0.72. Returns product and substrate fractions at equilibrium."
    )]
    async fn chemistry_equilibrium(
        &self,
        Parameters(params): Parameters<params::ChemistryEquilibriumParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::equilibrium(params)
    }

    #[tool(
        description = "Get all chemistry → PV mappings. Returns 8 cross-domain transfer mappings with confidence scores (0.72-0.92) and rationale for each mapping."
    )]
    async fn chemistry_pv_mappings(
        &self,
        Parameters(params): Parameters<params::ChemistryPvMappingsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::get_pv_mappings(params)
    }

    #[tool(
        description = "Simple threshold exceeded check. Returns boolean: signal > threshold. Useful as a gate for signal detection workflows."
    )]
    async fn chemistry_threshold_exceeded(
        &self,
        Parameters(params): Parameters<params::ChemistryThresholdExceededParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::check_threshold_exceeded(params)
    }

    #[tool(
        description = "Calculate Hill equation response (cooperative binding). Maps to signal_cascade_amplification in PV. PV confidence: 0.85. Y = I^nH / (K₀.₅^nH + I^nH). nH>1 amplifies, nH<1 dampens."
    )]
    async fn chemistry_hill_response(
        &self,
        Parameters(params): Parameters<params::ChemistryHillResponseParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::hill_cooperative(params)
    }

    #[tool(
        description = "Calculate Nernst potential (dynamic threshold). Maps to dynamic_decision_threshold in PV. PV confidence: 0.80. E = E⁰ - (RT/nF)ln(Q). Threshold shifts with concentration."
    )]
    async fn chemistry_nernst_potential(
        &self,
        Parameters(params): Parameters<params::ChemistryNernstParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::nernst_dynamic(params)
    }

    #[tool(
        description = "Calculate competitive inhibition rate. Maps to signal_interference_factor in PV. PV confidence: 0.78. v = Vmax[S] / (Km(1+[I]/Ki) + [S]). Competing signals raise threshold."
    )]
    async fn chemistry_inhibition_rate(
        &self,
        Parameters(params): Parameters<params::ChemistryInhibitionParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::inhibition_rate(params)
    }

    #[tool(
        description = "Calculate Eyring rate (transition state theory). Maps to signal_escalation_rate in PV. PV confidence: 0.82. k = κ(kB*T/h)exp(-ΔG‡/RT). More accurate than Arrhenius."
    )]
    async fn chemistry_eyring_rate(
        &self,
        Parameters(params): Parameters<params::ChemistryEyringRateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::eyring_transition(params)
    }

    #[tool(
        description = "Calculate Langmuir coverage (resource binding). Maps to case_slot_occupancy in PV. PV confidence: 0.88. θ = K[A]/(1+K[A]). Finite slots, saturation behavior."
    )]
    async fn chemistry_langmuir_coverage(
        &self,
        Parameters(params): Parameters<params::ChemistryLangmuirParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::langmuir_binding(params)
    }

    #[tool(
        description = "Calculate First Law closed system energy balance. Maps to case_backlog_change in PV. PV confidence: 0.85. ΔU = Q - W. Backlog change = cases received - cases resolved."
    )]
    async fn chemistry_first_law_closed(
        &self,
        Parameters(params): Parameters<params::ChemistryFirstLawClosedParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::first_law_closed(params)
    }

    #[tool(
        description = "Calculate First Law open system energy balance. Maps to throughput_energy_balance in PV. PV confidence: 0.85. dE/dt = Q̇ - Ẇ + Σṁh_in - Σṁh_out. Models case inflow/outflow with enthalpy (complexity)."
    )]
    async fn chemistry_first_law_open(
        &self,
        Parameters(params): Parameters<params::ChemistryFirstLawOpenParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::chemistry::first_law_open(params)
    }

    // ========================================================================
    // Cytokine Signaling Tools (3) - Typed Event Bus (Immune System Patterns)
    // ========================================================================

    #[tool(
        description = "Emit a cytokine signal to the global event bus. Families: il1 (alarm), il2 (growth), il6 (acute), il10 (suppress), tnf_alpha (terminate), ifn_gamma (activate), tgf_beta (regulate), csf (spawn). Severities: trace, low, medium, high, critical. Scopes: autocrine, paracrine, endocrine, systemic."
    )]
    async fn cytokine_emit(
        &self,
        Parameters(params): Parameters<params::CytokineEmitParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cytokine::emit(params)
    }

    #[tool(
        description = "Get cytokine bus status: signals emitted/delivered/dropped, cascades triggered, counts by family and severity. Shows current event bus health."
    )]
    async fn cytokine_status(&self) -> Result<CallToolResult, McpError> {
        tools::cytokine::status()
    }

    #[tool(
        description = "List all cytokine families with descriptions. Each family maps to an immune function: IL-1 (alarm), IL-6 (acute response), TNF-alpha (destroy threats), etc. Optional filter by family name."
    )]
    async fn cytokine_families(
        &self,
        Parameters(params): Parameters<params::CytokineListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cytokine::families(params)
    }

    // ========================================================================
    // Value Mining Tools (4) - Economic Signal Detection (PV Algorithms)
    // ========================================================================

    #[tool(
        description = "List all value signal types with their PV algorithm analogs and T1 primitives. Types: Sentiment (PRR), Trend (IC), Engagement (ROR), Virality (EBGM), Controversy (Chi²). Each type maps to a pharmacovigilance algorithm for cross-domain transfer."
    )]
    async fn value_signal_types(
        &self,
        Parameters(params): Parameters<params::ValueSignalTypesParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::value_mining::list_signal_types(params)
    }

    #[tool(
        description = "Detect a value signal from numeric observations. Applies PV algorithm analog (PRR, IC, ROR, EBGM, Chi²) based on signal type. Returns score, confidence, strength, and interpretation. Params: signal_type, observed, baseline, sample_size, entity, source."
    )]
    async fn value_signal_detect(
        &self,
        Parameters(params): Parameters<params::ValueSignalDetectParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::value_mining::detect_signal(params)
    }

    #[tool(
        description = "Create a baseline for signal detection. Sets expected rates for sentiment, engagement, and posting frequency. Used as comparison point for detecting signals."
    )]
    async fn value_baseline_create(
        &self,
        Parameters(params): Parameters<params::ValueBaselineCreateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::value_mining::create_baseline(params)
    }

    #[tool(
        description = "Get the PV ↔ Value Mining algorithm mapping. Shows how pharmacovigilance algorithms (PRR, ROR, IC, EBGM, Chi²) transfer to economic signal detection with confidence scores and T1 primitive groundings."
    )]
    async fn value_pv_mapping(
        &self,
        Parameters(params): Parameters<params::ValuePvMappingParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::value_mining::get_pv_mapping(params)
    }

    // ========================================================================
    // Academy Forge Tools (3) - Extract IR + Validate Academy Content
    // ========================================================================

    #[tool(
        description = "Extract structured knowledge from a NexCore Rust crate into an Intermediate Representation (IR). Returns module tree, public types, enums, constants, traits, dependency graph. With domain='vigilance': extracts 5 axioms, 8 harm types, 11 conservation laws, 3 theorems, axiom DAG, signal thresholds."
    )]
    async fn forge_extract(
        &self,
        Parameters(params): Parameters<params::ForgeExtractParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::academy_forge::forge_extract(params)
    }

    #[tool(
        description = "Validate academy content JSON against 27 rules: R1-R8 (schema), R9-R14 (accuracy vs IR), R15-R19 (conventions), R20-R23 (progression), R24-R27 (experiential learning). Returns pass/fail with findings by severity (Error/Warning/Advisory)."
    )]
    async fn forge_validate(
        &self,
        Parameters(params): Parameters<params::ForgeValidateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::academy_forge::forge_validate(params)
    }

    #[tool(
        description = "Generate a pathway authoring template from domain IR. Extracts domain analysis, then produces a complete StaticPathway JSON with axiom stages, harm type stages, quiz skeletons, Bloom progression, and TODO markers. Output passes R1-R8 (schema) and R20-R23 (progression) rules out of the box."
    )]
    async fn forge_scaffold(
        &self,
        Parameters(params): Parameters<params::ForgeScaffoldParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::academy_forge::forge_scaffold(params)
    }

    #[tool(
        description = "Return the StaticPathway JSON Schema describing the expected structure of academy pathway content. Use this schema when generating academy content to ensure it passes forge_validate."
    )]
    async fn forge_schema(&self) -> Result<CallToolResult, McpError> {
        tools::academy_forge::forge_schema()
    }

    #[tool(
        description = "Compile pathway JSON into Studio-compatible TypeScript stage files. Takes a forge-generated pathway JSON (tov-01.json) and produces TypeScript source files matching CapabilityStage from @/types/academy. Generates one NN-slug.ts per stage, config.ts, and index.ts."
    )]
    async fn forge_compile(
        &self,
        Parameters(params): Parameters<params::ForgeCompileParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::academy_forge::forge_compile(params)
    }

    // ========================================================================
    // Game Theory Tools (5) - 2x2 Nash + Forge N×M Pipeline
    // ========================================================================

    #[tool(description = "Analyze a 2x2 normal-form game and return pure/mixed Nash equilibria.")]
    async fn game_theory_nash_2x2(
        &self,
        Parameters(params): Parameters<params::game_theory::GameTheoryNash2x2Params>,
    ) -> Result<CallToolResult, McpError> {
        tools::game_theory::nash_2x2(params)
    }

    #[tool(
        description = "Build and analyze an N×M payoff matrix. Returns best responses, dominant strategies, minimax/maximin values, and expected payoffs per row."
    )]
    async fn forge_payoff_matrix(
        &self,
        Parameters(params): Parameters<params::forge::ForgePayoffMatrixParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::game_theory::forge_payoff_matrix(params)
    }

    #[tool(
        description = "Solve N×M mixed strategy Nash equilibrium via fictitious play. Returns mixed strategy weights, expected payoff, and convergence info."
    )]
    async fn forge_nash_solve(
        &self,
        Parameters(params): Parameters<params::forge::ForgeNashSolveParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::game_theory::forge_nash_solve(params)
    }

    #[tool(
        description = "Compute forge quality score Q = 0.40×prim + 0.25×combat + 0.20×turn + 0.15×survival. Returns total, component breakdown, letter grade, and code completeness."
    )]
    async fn forge_quality_score(
        &self,
        Parameters(params): Parameters<params::forge::ForgeQualityScoreParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::game_theory::forge_quality_score(params)
    }

    #[tool(
        description = "Generate Rust code from collected Lex Primitiva (0-15) and defeated safety enemies. Returns complete forge_output.rs with GroundsTo-annotated types."
    )]
    async fn forge_code_generate(
        &self,
        Parameters(params): Parameters<params::forge::ForgeCodeGenerateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::game_theory::forge_code_generate(params)
    }

    // ========================================================================
    // STEM Primitives Tools (11) - Cross-Domain T2-P Traits
    // ========================================================================

    #[tool(
        description = "Get STEM system version, trait counts, domain summary, and T1 distribution. 32 traits across 4 domains (Science, Chemistry, Physics, Mathematics)."
    )]
    async fn stem_version(&self) -> Result<CallToolResult, McpError> {
        tools::stem::version()
    }

    #[tool(
        description = "Get the full 32-trait STEM taxonomy with T1 groundings. Shows every trait's domain, T1 primitive (SEQUENCE/MAPPING/RECURSION/STATE), and cross-domain transfer description."
    )]
    async fn stem_taxonomy(&self) -> Result<CallToolResult, McpError> {
        tools::stem::taxonomy()
    }

    #[tool(
        description = "Combine two confidence values via multiplicative composition. Confidence (T2-P) is the universal uncertainty quantifier shared across all STEM domains. Result clamped to [0.0, 1.0]."
    )]
    async fn stem_confidence_combine(
        &self,
        Parameters(params): Parameters<params::stem::StemConfidenceCombineParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::confidence_combine(params)
    }

    #[tool(
        description = "Get tier classification info and transfer multiplier. Tiers: T1 (1.0), T2-P (0.9), T2-C (0.7), T3 (0.4). Higher tier = more transferable across domains."
    )]
    async fn stem_tier_info(
        &self,
        Parameters(params): Parameters<params::stem::StemTierInfoParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::tier_info(params)
    }

    #[tool(
        description = "Calculate chemistry equilibrium balance from forward/reverse rates. Returns K (equilibrium constant), whether system is at equilibrium, and whether products are favored. Grounds to STATE (Harmonize trait)."
    )]
    async fn stem_chem_balance(
        &self,
        Parameters(params): Parameters<params::stem::StemChemBalanceParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::chem_balance(params)
    }

    #[tool(
        description = "Create a Fraction [0.0, 1.0] and check saturation status. Saturated when ≥ 0.99. Grounds to STATE (Saturate trait). Cross-domain: capacity utilization."
    )]
    async fn stem_chem_fraction(
        &self,
        Parameters(params): Parameters<params::stem::StemChemFractionParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::chem_fraction(params)
    }

    #[tool(
        description = "Calculate acceleration from force and mass (Newton's F=ma → a=F/m). Grounds to MAPPING (YieldForce trait). Cross-domain: dose-response relationship."
    )]
    async fn stem_phys_fma(
        &self,
        Parameters(params): Parameters<params::stem::StemPhysFmaParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::phys_fma(params)
    }

    #[tool(
        description = "Check quantity conservation (before vs after within tolerance). Grounds to STATE (Preserve trait). Cross-domain: case count conservation in reporting pipelines."
    )]
    async fn stem_phys_conservation(
        &self,
        Parameters(params): Parameters<params::stem::StemPhysConservationParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::phys_conservation(params)
    }

    #[tool(
        description = "Convert frequency to period (T = 1/f). Grounds to RECURSION (Harmonics trait). Cross-domain: seasonal reporting pattern cycle detection."
    )]
    async fn stem_phys_period(
        &self,
        Parameters(params): Parameters<params::stem::StemPhysPeriodParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::phys_period(params)
    }

    #[tool(
        description = "Check if a value is within optional lower/upper bounds and compute clamped value. Grounds to STATE (Bound trait). Cross-domain: confidence interval boundary checking."
    )]
    async fn stem_math_bounds_check(
        &self,
        Parameters(params): Parameters<params::stem::StemMathBoundsCheckParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::math_bounds_check(params)
    }

    #[tool(
        description = "Invert a mathematical relation (LessThan↔GreaterThan, Equal↔Equal, Incomparable↔Incomparable). Checks symmetry. Grounds to MAPPING (Symmetric trait)."
    )]
    async fn stem_math_relation_invert(
        &self,
        Parameters(params): Parameters<params::stem::StemMathRelationInvertParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::math_relation_invert(params)
    }

    #[tool(
        description = "Create a concentration ratio value and optionally compare two ratios (fold change). Grounds to MAPPING+QUANTITY (Concentrate trait). Cross-domain: substance concentration, dose-per-volume."
    )]
    async fn stem_chem_ratio(
        &self,
        Parameters(params): Parameters<params::stem::StemChemRatioParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::chem_ratio(params)
    }

    #[tool(
        description = "Create a rate-of-change value and optionally compare two rates. Grounds to MAPPING+QUANTITY (Energize trait). Cross-domain: reaction rate, reporting frequency, signal velocity."
    )]
    async fn stem_chem_rate(
        &self,
        Parameters(params): Parameters<params::stem::StemChemRateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::chem_rate(params)
    }

    #[tool(
        description = "Create a binding affinity value [0.0-1.0] and classify as weak/moderate/strong. Grounds to MAPPING (Interact trait). Cross-domain: drug-target binding, signal-receptor affinity."
    )]
    async fn stem_chem_affinity(
        &self,
        Parameters(params): Parameters<params::stem::StemChemAffinityParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::chem_affinity(params)
    }

    #[tool(
        description = "Create an amplitude value and optionally superpose (add) with another. Grounds to QUANTITY (Harmonics trait). Cross-domain: signal strength, effect magnitude."
    )]
    async fn stem_phys_amplitude(
        &self,
        Parameters(params): Parameters<params::stem::StemPhysAmplitudeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::phys_amplitude(params)
    }

    #[tool(
        description = "Apply a scale factor to a value (output = factor × input). Grounds to MAPPING (Scale trait). Cross-domain: dose scaling, proportional adjustment."
    )]
    async fn stem_phys_scale(
        &self,
        Parameters(params): Parameters<params::stem::StemPhysScaleParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::phys_scale(params)
    }

    #[tool(
        description = "Calculate resistance force from inertial mass and proposed change magnitude. Grounds to PERSISTENCE (Inertia trait). Cross-domain: organizational resistance, system momentum."
    )]
    async fn stem_phys_inertia(
        &self,
        Parameters(params): Parameters<params::stem::StemPhysInertiaParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::phys_inertia(params)
    }

    #[tool(
        description = "Construct a logical proof from premises and conclusion, marking it valid or invalid. Grounds to SEQUENCE+EXISTENCE (Prove trait). Cross-domain: causality assessment, regulatory justification."
    )]
    async fn stem_math_proof(
        &self,
        Parameters(params): Parameters<params::stem::StemMathProofParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::math_proof(params)
    }

    #[tool(
        description = "Check if a value is the identity element for addition (0) or multiplication (1). Grounds to STATE (Identify trait). Cross-domain: neutral element, baseline, no-op."
    )]
    async fn stem_math_identity(
        &self,
        Parameters(params): Parameters<params::stem::StemMathIdentityParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::math_identity(params)
    }

    #[tool(
        description = "Create a distance value and optionally compare two distances for approximate equality. Grounds to QUANTITY+MAPPING (metric space). Cross-domain: edit distance, concept similarity."
    )]
    async fn stem_spatial_distance(
        &self,
        Parameters(params): Parameters<params::stem::StemSpatialDistanceParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::spatial_distance(params)
    }

    #[tool(
        description = "Check the triangle inequality: d(a,c) ≤ d(a,b) + d(b,c). Validates metric consistency. Grounds to BOUNDARY+COMPARISON."
    )]
    async fn stem_spatial_triangle(
        &self,
        Parameters(params): Parameters<params::stem::StemSpatialTriangleParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::spatial_triangle(params)
    }

    #[tool(
        description = "Check if a point is within a neighborhood of given radius (open or closed boundary). Grounds to BOUNDARY+LOCATION. Cross-domain: threshold neighborhoods, fuzzy match radius."
    )]
    async fn stem_spatial_neighborhood(
        &self,
        Parameters(params): Parameters<params::stem::StemSpatialNeighborhoodParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::spatial_neighborhood(params)
    }

    #[tool(
        description = "Dimension rank and subspace checking. Returns codimension when comparing. Grounds to QUANTITY. Cross-domain: feature dimensionality, parameter space."
    )]
    async fn stem_spatial_dimension(
        &self,
        Parameters(params): Parameters<params::stem::StemSpatialDimensionParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::spatial_dimension(params)
    }

    #[tool(
        description = "Orientation composition (positive × negative = negative, etc.). Grounds to MAPPING+STATE. Cross-domain: signal direction, trend polarity."
    )]
    async fn stem_spatial_orientation(
        &self,
        Parameters(params): Parameters<params::stem::StemSpatialOrientationParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::stem::spatial_orientation(params)
    }

    // ========================================================================
    // Visualization Tools (6) - SVG diagrams for STEM concepts
    // ========================================================================

    #[tool(
        description = "Generate STEM taxonomy sunburst SVG showing all 32 traits across 4 domains (Science, Chemistry, Physics, Mathematics), color-coded by T1 grounding (MAPPING, SEQUENCE, RECURSION, STATE, PERSISTENCE, BOUNDARY, SUM). Returns self-contained SVG."
    )]
    async fn viz_stem_taxonomy(
        &self,
        Parameters(params): Parameters<params::viz::VizTaxonomyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::viz::taxonomy(params)
    }

    #[tool(
        description = "Generate type composition SVG showing how a type decomposes to T1 Lex Primitiva. Shows the type at center with T1 primitives radiating outward. Dominant primitive highlighted. Tier classification shown (T1/T2-P/T2-C/T3)."
    )]
    async fn viz_type_composition(
        &self,
        Parameters(params): Parameters<params::viz::VizCompositionParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::viz::composition(params)
    }

    #[tool(
        description = "Generate circular flow SVG for STEM method loops. Domain options: 'science' (8-step SCIENCE loop with Heisenberg/Shannon/Godel limits), 'chemistry' (9-step CHEMISTRY loop), 'math' (9-step MATHEMATICS loop)."
    )]
    async fn viz_method_loop(
        &self,
        Parameters(params): Parameters<params::viz::VizLoopParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::viz::method_loop(params)
    }

    #[tool(
        description = "Generate confidence propagation waterfall SVG. Shows how confidence flows through derivation chains. Rule: conf(child) <= min(conf(parents)). Claims as JSON array: [{\"text\":\"...\", \"confidence\":0.95, \"proof_type\":\"analytical\", \"parent\":null}]."
    )]
    async fn viz_confidence_chain(
        &self,
        Parameters(params): Parameters<params::viz::VizConfidenceParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::viz::confidence(params)
    }

    #[tool(
        description = "Generate bounds visualization SVG. Shows a value on a number line with optional lower/upper bounds, in-bounds/out-of-bounds status, and clamped position. Cross-domain: confidence intervals, price limits, range types."
    )]
    async fn viz_bounds(
        &self,
        Parameters(params): Parameters<params::viz::VizBoundsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::viz::bounds(params)
    }

    #[tool(
        description = "Generate DAG topology SVG with parallel execution levels. Edges as JSON array: [[\"from\",\"to\"], ...]. Nodes auto-discovered from edges. Shows topological ordering and which nodes can execute concurrently."
    )]
    async fn viz_dag(
        &self,
        Parameters(params): Parameters<params::viz::VizDagParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::viz::dag(params)
    }

    // ========================================================================
    // Watchtower Tools (5) - Session monitoring and primitive extraction
    // ========================================================================

    #[tool(
        description = "List saved Watchtower sessions. Returns session files with metadata (size, modified time) from ~/.claude/logs/sessions/."
    )]
    async fn watchtower_sessions_list(
        &self,
        Parameters(_params): Parameters<params::watchtower::WatchtowerSessionsListParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = tools::watchtower::watchtower_sessions_list();
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    }

    #[tool(
        description = "Get active Claude Code sessions from recent log entries. Returns session IDs and their working directories."
    )]
    async fn watchtower_active_sessions(
        &self,
        Parameters(_params): Parameters<params::watchtower::WatchtowerActiveSessionsParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = tools::watchtower::watchtower_active_sessions();
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    }

    #[tool(
        description = "Analyze a session log and extract behavioral primitives. Returns statistics, T1/T2/T3 primitives, anti-patterns, and transfer confidence scores."
    )]
    async fn watchtower_analyze(
        &self,
        Parameters(params): Parameters<params::watchtower::WatchtowerAnalyzeParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = tools::watchtower::watchtower_analyze(params.session_path.as_deref());
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    }

    #[tool(
        description = "Get hook telemetry statistics. Returns tool distribution, average hook timing, and session activity from hook_telemetry.jsonl."
    )]
    async fn watchtower_telemetry_stats(
        &self,
        Parameters(_params): Parameters<params::watchtower::WatchtowerTelemetryStatsParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = tools::watchtower::watchtower_telemetry_stats();
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    }

    #[tool(
        description = "Get recent log entries from commands.log. Optionally filter by session ID and limit count."
    )]
    async fn watchtower_recent(
        &self,
        Parameters(params): Parameters<params::watchtower::WatchtowerRecentParams>,
    ) -> Result<CallToolResult, McpError> {
        let result =
            tools::watchtower::watchtower_recent(params.count, params.session_filter.as_deref());
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    }

    #[tool(
        description = "Audit symbols in files for potential collisions. Scans for single-letter and short symbols, groups by context (definition/equation/reference), and flags where same symbol may have different meanings."
    )]
    async fn watchtower_symbol_audit(
        &self,
        Parameters(params): Parameters<params::watchtower::WatchtowerSymbolAuditParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = tools::watchtower::watchtower_symbol_audit(&params.path);
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    }

    #[tool(
        description = "Get Gemini API call statistics. Returns total calls, token usage, latency, and breakdown by session/flow from gemini_telemetry.jsonl."
    )]
    async fn watchtower_gemini_stats(
        &self,
        Parameters(_params): Parameters<params::watchtower::WatchtowerGeminiStatsParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = tools::watchtower::watchtower_gemini_stats();
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    }

    #[tool(
        description = "Get recent Gemini API calls. Returns the latest N entries with timestamps, models, tokens, and latency."
    )]
    async fn watchtower_gemini_recent(
        &self,
        Parameters(params): Parameters<params::watchtower::WatchtowerGeminiRecentParams>,
    ) -> Result<CallToolResult, McpError> {
        let result = tools::watchtower::watchtower_gemini_recent(params.count);
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    }

    #[tool(
        description = "Get unified Claude + Gemini telemetry view. Combines hook telemetry and Gemini API stats into a single dashboard."
    )]
    async fn watchtower_unified(
        &self,
        Parameters(params): Parameters<params::watchtower::WatchtowerUnifiedParams>,
    ) -> Result<CallToolResult, McpError> {
        let result =
            tools::watchtower::watchtower_unified(params.include_claude, params.include_gemini);
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            serde_json::to_string_pretty(&result).unwrap_or_else(|_| "{}".to_string()),
        )]))
    }

    // ========================================================================
    // Telemetry Intelligence Tools (6) - External source monitoring
    // ========================================================================

    #[tool(
        description = "List all discovered telemetry sources (sessions). Scans telemetry directories and returns session metadata including project hash, filename, and path."
    )]
    async fn telemetry_sources_list(
        &self,
        Parameters(params): Parameters<params::telemetry::TelemetrySourcesListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::telemetry_intel::sources_list(params)
    }

    #[tool(
        description = "Deep analysis of a specific telemetry source session. Returns operation counts, file access patterns, and token usage. Provide session_path or project_hash."
    )]
    async fn telemetry_source_analyze(
        &self,
        Parameters(params): Parameters<params::telemetry::TelemetrySourceAnalyzeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::telemetry_intel::source_analyze(params)
    }

    #[tool(
        description = "Cross-reference telemetry with governance module changes. Identifies file accesses to governance-related paths (primitives, governance, capabilities, constitutional) and tracks modifications."
    )]
    async fn telemetry_governance_crossref(
        &self,
        Parameters(params): Parameters<params::telemetry::TelemetryGovernanceCrossrefParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::telemetry_intel::governance_crossref(params)
    }

    #[tool(
        description = "Track snapshot/artifact version history. Discovers brain sessions and their artifacts, showing version evolution over time. Provide session_id for specific session or omit for overview."
    )]
    async fn telemetry_snapshot_evolution(
        &self,
        Parameters(params): Parameters<params::telemetry::TelemetrySnapshotEvolutionParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::telemetry_intel::snapshot_evolution(params)
    }

    #[tool(
        description = "Generate full intelligence report. Aggregates data from telemetry sources and brain snapshots to produce a comprehensive report with token usage, file patterns, and governance access."
    )]
    async fn telemetry_intel_report(
        &self,
        Parameters(params): Parameters<params::telemetry::TelemetryIntelReportParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::telemetry_intel::intel_report(params)
    }

    #[tool(
        description = "Real-time activity stream. Returns the most recent operations across all telemetry sources, sorted by timestamp descending."
    )]
    async fn telemetry_recent(
        &self,
        Parameters(params): Parameters<params::telemetry::TelemetryRecentParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::telemetry_intel::recent_activity(params)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // PRIMITIVE SCANNER (2 tools) - Domain primitive extraction
    // ═══════════════════════════════════════════════════════════════════════════

    #[tool(
        description = "Scan sources for primitives in a domain. Returns T1/T2/T3 tier classification."
    )]
    async fn primitive_scan(
        &self,
        Parameters(params): Parameters<params::primitive_scanner::PrimitiveScanParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::primitive_scanner::primitive_scan(params)
    }

    #[tool(
        description = "Batch test terms for primitiveness using 3-test protocol. Returns verdict and tier for each term."
    )]
    async fn primitive_batch_test(
        &self,
        Parameters(params): Parameters<params::primitive_scanner::PrimitiveBatchTestParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::primitive_scanner::primitive_batch_test(params)
    }

    // ═══════════════════════════════════════════════════════════════════════════
    // ALGOVIGILANCE (6 tools) - ICSR deduplication + signal triage
    // ═══════════════════════════════════════════════════════════════════════════

    #[tool(
        description = "Compare two ICSR narratives for similarity using Jaccard tokenization with medical stopword removal. Returns similarity score, threshold comparison, and duplicate verdict."
    )]
    async fn algovigil_dedup_pair(
        &self,
        Parameters(params): Parameters<params::algovigilance::AlgovigilDedupPairParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::algovigilance::dedup_pair(params)
    }

    #[tool(
        description = "Configure batch ICSR deduplication for a drug. Sets up DedupFunction with threshold and limit. Use algovigil_dedup_pair for pairwise comparison."
    )]
    async fn algovigil_dedup_batch(
        &self,
        Parameters(params): Parameters<params::algovigilance::AlgovigilDedupBatchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::algovigilance::dedup_batch(params)
    }

    #[tool(
        description = "Get signal with current decay-adjusted relevance. Looks up signal by drug+event in persisted queue, applies exponential decay (half-life configurable)."
    )]
    async fn algovigil_triage_decay(
        &self,
        Parameters(params): Parameters<params::algovigilance::AlgovigilTriageDecayParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::algovigilance::triage_decay(params)
    }

    #[tool(
        description = "Reinforce a signal with new case evidence. Restores confidence toward original level proportional to new cases."
    )]
    async fn algovigil_triage_reinforce(
        &self,
        Parameters(params): Parameters<params::algovigilance::AlgovigilTriageReinforceParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::algovigilance::triage_reinforce(params)
    }

    #[tool(
        description = "Get prioritized signal queue for a drug. Returns signals sorted by decay-adjusted relevance with configurable half-life and cutoff."
    )]
    async fn algovigil_triage_queue(
        &self,
        Parameters(params): Parameters<params::algovigilance::AlgovigilTriageQueueParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::algovigilance::triage_queue(params)
    }

    #[tool(
        description = "Algovigilance system status. Returns store health, synonym count, queue count, and registered function metadata."
    )]
    async fn algovigil_status(&self) -> Result<CallToolResult, McpError> {
        tools::algovigilance::status()
    }

    // ========================================================================
    // Edit Distance Framework (4 tools)
    // ========================================================================

    #[tool(
        description = "Compute edit distance between two strings. Supports algorithms: 'levenshtein' (default), 'damerau' (adds transposition), 'lcs' (indel-only). Returns distance, similarity, and string lengths."
    )]
    async fn edit_distance_compute(
        &self,
        Parameters(params): Parameters<params::edit_distance::EditDistanceParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::edit_distance::edit_distance_compute(params)
    }

    #[tool(
        description = "Compute string similarity with threshold check. Returns similarity score (0-1), distance, and whether similarity meets threshold (default 0.8). Useful for PV drug name matching."
    )]
    async fn edit_distance_similarity(
        &self,
        Parameters(params): Parameters<params::edit_distance::EditDistanceSimilarityParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::edit_distance::edit_distance_similarity(params)
    }

    #[tool(
        description = "Compute edit distance with full traceback: returns the sequence of insert/delete/substitute operations to transform source into target. Uses full-matrix DP solver."
    )]
    async fn edit_distance_traceback(
        &self,
        Parameters(params): Parameters<params::edit_distance::EditDistanceTracebackParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::edit_distance::edit_distance_traceback(params)
    }

    #[tool(
        description = "Look up cross-domain transfer confidence for edit distance concepts. Pre-computed maps: text/unicode ↔ bioinformatics/dna, spell-checking, nlp/tokens, pharmacovigilance; bioinformatics/dna ↔ music/melody. Returns structural/functional/contextual scores and caveats."
    )]
    async fn edit_distance_transfer(
        &self,
        Parameters(params): Parameters<params::edit_distance::EditDistanceTransferParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::edit_distance::edit_distance_transfer(params)
    }

    #[tool(
        description = "Batch edit distance: compare a query string against multiple candidates. Returns matches sorted by similarity (descending). Supports min_similarity filter, limit, and algorithm selection (levenshtein/damerau/lcs)."
    )]
    async fn edit_distance_batch(
        &self,
        Parameters(params): Parameters<params::edit_distance::EditDistanceBatchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::edit_distance::edit_distance_batch(params)
    }

    // ========================================================================
    // Integrity Assessment (3 tools)
    // ========================================================================

    #[tool(
        description = "Analyze text for AI-generation indicators using 5 statistical features (Zipf, entropy, burstiness, perplexity, TTR) aggregated via chemistry primitives (Beer-Lambert + Hill + Arrhenius). Optional Bloom taxonomy level (1-7) adapts threshold. Returns verdict (Human/Generated), probability, confidence, features."
    )]
    async fn integrity_analyze(
        &self,
        Parameters(params): Parameters<params::integrity::IntegrityAnalyzeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::integrity::integrity_analyze(params)
    }

    #[tool(
        description = "Assess KSB response integrity. Convenience endpoint requiring Bloom level (1-7). Higher Bloom = stricter threshold: L1 Remember=0.70, L4 Analyze=0.50, L7 Create=0.30. Returns verdict, probability, hill_score."
    )]
    async fn integrity_assess_ksb(
        &self,
        Parameters(params): Parameters<params::integrity::IntegrityAssessKsbParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::integrity::integrity_assess_ksb(params)
    }

    #[tool(
        description = "Get domain calibration profile with baseline feature expectations for PV domains (D02-D12). Returns domain baselines (zipf_alpha, entropy_std, burstiness, perplexity_var, ttr) and Bloom threshold presets."
    )]
    async fn integrity_calibration(
        &self,
        Parameters(params): Parameters<params::integrity::IntegrityCalibrationParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::integrity::integrity_calibration(params)
    }

    // ========================================================================
    // Decision Tree Engine (6 tools)
    // ========================================================================

    #[tool(
        description = "Train a CART decision tree on feature matrix and labels. Returns tree_id for subsequent predict/prune/export calls. Supports criteria: gini (default), entropy, gain_ratio, mse (regression). Configure max_depth, min_samples_split, min_samples_leaf for pre-pruning."
    )]
    async fn dtree_train(
        &self,
        Parameters(params): Parameters<params::dtree::DtreeTrainParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::dtree::dtree_train(params)
    }

    #[tool(
        description = "Predict class label (or regression value) for a single sample using a trained tree. Returns prediction, confidence, class distribution, and the explainable rule path from root to leaf."
    )]
    async fn dtree_predict(
        &self,
        Parameters(params): Parameters<params::dtree::DtreePredictParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::dtree::dtree_predict(params)
    }

    #[tool(
        description = "Get feature importance scores from a trained tree. Importance = sum of weighted impurity decreases across all splits using each feature. Normalized to sum to 1.0, sorted descending."
    )]
    async fn dtree_importance(
        &self,
        Parameters(params): Parameters<params::dtree::DtreeImportanceParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::dtree::dtree_importance(params)
    }

    #[tool(
        description = "Prune a trained tree using cost-complexity pruning (CCP). Alpha controls aggressiveness: 0.0 = no pruning, higher = more aggressive. Returns before/after depth and leaf counts."
    )]
    async fn dtree_prune(
        &self,
        Parameters(params): Parameters<params::dtree::DtreePruneParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::dtree::dtree_prune(params)
    }

    #[tool(
        description = "Export a trained tree in specified format. Formats: 'json' (serialized tree), 'rules' (human-readable if-then-else), 'summary' (statistics overview)."
    )]
    async fn dtree_export(
        &self,
        Parameters(params): Parameters<params::dtree::DtreeExportParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::dtree::dtree_export(params)
    }

    #[tool(
        description = "Get metadata and statistics for a trained tree: depth, leaf count, split count, feature count, class count, sample count, criterion, and feature names."
    )]
    async fn dtree_info(
        &self,
        Parameters(params): Parameters<params::dtree::DtreeInfoParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::dtree::dtree_info(params)
    }

    // ========================================================================
    // Cargo Toolchain Tools (6) — Structured build/check/test/clippy/fmt/tree
    // ========================================================================

    #[tool(
        description = "Run cargo check and return structured diagnostics (errors, warnings with file/line/column locations and error codes). Uses --message-format=json for parsed output."
    )]
    async fn cargo_check(
        &self,
        Parameters(params): Parameters<params::cargo::CargoCheckParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cargo::cargo_check(params)
    }

    #[tool(
        description = "Run cargo build and return structured diagnostics. Uses --message-format=json. Set release=true for optimized builds."
    )]
    async fn cargo_build(
        &self,
        Parameters(params): Parameters<params::cargo::CargoBuildParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cargo::cargo_build(params)
    }

    #[tool(
        description = "Run cargo test and return structured results: pass/fail/ignore counts per test, plus any compile diagnostics. Supports test_filter and skip parameters."
    )]
    async fn cargo_test(
        &self,
        Parameters(params): Parameters<params::cargo::CargoTestParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cargo::cargo_test(params)
    }

    #[tool(
        description = "Run cargo clippy and return structured lint diagnostics. deny_warnings defaults to true (-- -D warnings). Uses --message-format=json for parsed output."
    )]
    async fn cargo_clippy(
        &self,
        Parameters(params): Parameters<params::cargo::CargoClippyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cargo::cargo_clippy(params)
    }

    #[tool(
        description = "Run cargo fmt. Set check_only=true to verify formatting without modifying files. Returns list of unformatted files in check mode."
    )]
    async fn cargo_fmt(
        &self,
        Parameters(params): Parameters<params::cargo::CargoFmtParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cargo::cargo_fmt(params)
    }

    #[tool(
        description = "Run cargo tree and return the dependency tree. Supports package filter, depth limit, invert (reverse deps), and duplicates-only mode."
    )]
    async fn cargo_tree(
        &self,
        Parameters(params): Parameters<params::cargo::CargoTreeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::cargo::cargo_tree(params)
    }

    // ========================================================================
    // Git Tools (8) — Structured git CLI wrappers
    // ========================================================================

    #[tool(
        description = "Run git status and return structured output: branch name, staged/modified/untracked file counts, and per-file status."
    )]
    async fn git_status(
        &self,
        Parameters(params): Parameters<params::git::GitStatusParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::git::git_status(params)
    }

    #[tool(
        description = "Run git diff and return both stat summary and full diff text. Set staged=true for --staged. Supports file filter and ref_spec for comparing against branches/commits."
    )]
    async fn git_diff(
        &self,
        Parameters(params): Parameters<params::git::GitDiffParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::git::git_diff(params)
    }

    #[tool(
        description = "Run git log and return structured commit history: hash, author, email, date, subject. Set oneline=true for compact output. Default: 10 commits."
    )]
    async fn git_log(
        &self,
        Parameters(params): Parameters<params::git::GitLogParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::git::git_log(params)
    }

    #[tool(
        description = "Stage files and create a git commit. Safety: refuses to commit files matching sensitive patterns (.env, credentials, .key, .pem). If files is empty, commits currently staged changes."
    )]
    async fn git_commit(
        &self,
        Parameters(params): Parameters<params::git::GitCommitParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::git::git_commit(params)
    }

    #[tool(
        description = "List, create, or delete git branches. Default: lists local branches. Set list=true for all (local + remote). Use create/delete for branch management."
    )]
    async fn git_branch(
        &self,
        Parameters(params): Parameters<params::git::GitBranchParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::git::git_branch(params)
    }

    #[tool(
        description = "Checkout a git branch, tag, or commit. Set create=true to create a new branch (-b flag)."
    )]
    async fn git_checkout(
        &self,
        Parameters(params): Parameters<params::git::GitCheckoutParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::git::git_checkout(params)
    }

    #[tool(
        description = "Push commits to remote. Safety: blocks force push and direct push to main/master. Set set_upstream=true for -u flag."
    )]
    async fn git_push(
        &self,
        Parameters(params): Parameters<params::git::GitPushParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::git::git_push(params)
    }

    #[tool(
        description = "Git stash operations: push (save changes), pop (apply+remove), list (show stashes), drop (remove top stash). Optional message for push."
    )]
    async fn git_stash(
        &self,
        Parameters(params): Parameters<params::git::GitStashParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::git::git_stash(params)
    }

    // ========================================================================
    // GitHub CLI Tools (5) — PR and issue management via gh
    // ========================================================================

    #[tool(
        description = "Create a GitHub pull request. Returns the PR URL. Set draft=true for draft PRs. Optional base branch."
    )]
    async fn gh_pr_create(
        &self,
        Parameters(params): Parameters<params::gh::GhPrCreateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gh::gh_pr_create(params).await
    }

    #[tool(
        description = "View a GitHub pull request. Returns PR details (title, state, body, author, additions/deletions). If number not specified, views PR for current branch."
    )]
    async fn gh_pr_view(
        &self,
        Parameters(params): Parameters<params::gh::GhPrViewParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gh::gh_pr_view(params).await
    }

    #[tool(
        description = "List GitHub pull requests. Filter by state (open/closed/merged/all). Returns number, title, state, author, branch, date, URL."
    )]
    async fn gh_pr_list(
        &self,
        Parameters(params): Parameters<params::gh::GhPrListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gh::gh_pr_list(params).await
    }

    #[tool(
        description = "View a GitHub issue by number. Returns title, state, body, author, labels, assignees."
    )]
    async fn gh_issue_view(
        &self,
        Parameters(params): Parameters<params::gh::GhIssueViewParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gh::gh_issue_view(params).await
    }

    #[tool(
        description = "Call the GitHub REST API directly. Safety: blocks DELETE method by default (set allow_delete=true to proceed). Supports GET, POST, PUT, PATCH."
    )]
    async fn gh_api(
        &self,
        Parameters(params): Parameters<params::gh::GhApiParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::gh::gh_api(params).await
    }

    // ========================================================================
    // Systemctl Tools (4) — Service management (user scope by default)
    // ========================================================================

    #[tool(
        description = "Get systemctl status for a unit. Returns active state, sub-state, PID, and full status output. Defaults to --user scope."
    )]
    async fn systemctl_status(
        &self,
        Parameters(params): Parameters<params::service::SystemctlStatusParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::service::systemctl_status(params)
    }

    #[tool(
        description = "Restart a systemd unit. Safety: only allows --user scope. System-wide restart is blocked."
    )]
    async fn systemctl_restart(
        &self,
        Parameters(params): Parameters<params::service::SystemctlRestartParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::service::systemctl_restart(params)
    }

    #[tool(
        description = "Start a systemd unit. Safety: only allows --user scope. System-wide start is blocked."
    )]
    async fn systemctl_start(
        &self,
        Parameters(params): Parameters<params::service::SystemctlStartParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::service::systemctl_start(params)
    }

    #[tool(
        description = "List systemd units. Defaults to --user scope. Optional state filter (running, failed, active)."
    )]
    async fn systemctl_list(
        &self,
        Parameters(params): Parameters<params::service::SystemctlListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::service::systemctl_list(params)
    }

    // ========================================================================
    // NPM Tools (4) — Node.js package management
    // ========================================================================

    #[tool(
        description = "Run an npm script from package.json. Returns stdout/stderr with 120s timeout. Output truncated at 50KB."
    )]
    async fn npm_run(
        &self,
        Parameters(params): Parameters<params::npm::NpmRunParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::npm::npm_run(params).await
    }

    #[tool(
        description = "Install npm packages. If packages list is empty, runs npm install from package.json. Set dev=true for --save-dev."
    )]
    async fn npm_install(
        &self,
        Parameters(params): Parameters<params::npm::NpmInstallParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::npm::npm_install(params).await
    }

    #[tool(description = "List installed npm packages as JSON. depth=0 for top-level only.")]
    async fn npm_list(
        &self,
        Parameters(params): Parameters<params::npm::NpmListParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::npm::npm_list(params).await
    }

    #[tool(
        description = "Check for outdated npm packages. Returns JSON with current, wanted, and latest versions per package."
    )]
    async fn npm_outdated(
        &self,
        Parameters(params): Parameters<params::npm::NpmOutdatedParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::npm::npm_outdated(params).await
    }

    // ========================================================================
    // Filesystem Tools (4) — mkdir, copy, move, chmod
    // ========================================================================

    #[tool(
        description = "Create a directory. Set parents=true for recursive creation (-p). Uses Rust std::fs for speed."
    )]
    async fn fs_mkdir(
        &self,
        Parameters(params): Parameters<params::fs::FsMkdirParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::fs::fs_mkdir(params)
    }

    #[tool(
        description = "Copy a file or directory. Set recursive=true for directories (uses cp -a to preserve attributes)."
    )]
    async fn fs_copy(
        &self,
        Parameters(params): Parameters<params::fs::FsCopyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::fs::fs_copy(params)
    }

    #[tool(
        description = "Move a file or directory. Directories use safe move pattern (cp -a + verify + rm) to prevent data loss."
    )]
    async fn fs_move(
        &self,
        Parameters(params): Parameters<params::fs::FsMoveParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::fs::fs_move(params)
    }

    #[tool(
        description = "Change file permissions. Safety: blocks mode 777 (world-writable). Set recursive=true for -R flag."
    )]
    async fn fs_chmod(
        &self,
        Parameters(params): Parameters<params::fs::FsChmodParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::fs::fs_chmod(params)
    }

    // ========================================================================
    // Frontend & Accessibility Tools (6) — WCAG, touch targets, type/spacing scale
    // ========================================================================

    #[tool(
        description = "Compute WCAG 2.1 contrast ratio between foreground and background colors. Returns ratio, AA/AAA verdicts, and recommendations. Supports RGBA foreground (alpha-blended onto background). Provide colors as [r,g,b] or [r,g,b,a] (0-255 for RGB, 0.0-1.0 for alpha)."
    )]
    async fn frontend_wcag_contrast(
        &self,
        Parameters(params): Parameters<params::frontend::WcagContrastParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::frontend::wcag_contrast(params)
    }

    #[tool(
        description = "Blend an RGBA foreground color onto an opaque RGB background. Returns the effective RGB color after alpha compositing. Useful for computing actual rendered colors from CSS rgba() values."
    )]
    async fn frontend_color_blend(
        &self,
        Parameters(params): Parameters<params::frontend::ColorBlendParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::frontend::color_blend(params)
    }

    #[tool(
        description = "Check interactive element dimensions against WCAG 2.5.5 (AA: 44x44px) or 2.5.8 (AAA: 48x48px) touch target requirements. Returns pass/fail verdict and deficit in pixels."
    )]
    async fn frontend_touch_target(
        &self,
        Parameters(params): Parameters<params::frontend::TouchTargetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::frontend::touch_target(params)
    }

    #[tool(
        description = "Audit a set of font sizes for modular scale compliance. Detects gaps (missing intermediate sizes), clusters (nearly identical sizes), and ratio deviations. Default target: golden ratio (1.618). Returns per-step analysis and compliance score."
    )]
    async fn frontend_type_scale_audit(
        &self,
        Parameters(params): Parameters<params::frontend::TypeScaleAuditParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::frontend::type_scale_audit(params)
    }

    #[tool(
        description = "Audit spacing values against a modular scale (base * ratio^n). Reports which values are on-scale, off-scale, and the nearest scale step. Default: 8px base with golden ratio."
    )]
    async fn frontend_spacing_audit(
        &self,
        Parameters(params): Parameters<params::frontend::SpacingAuditParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::frontend::spacing_audit(params)
    }

    #[tool(
        description = "Combined accessibility audit in one call. Checks multiple contrast pairs (WCAG AA/AAA), touch target sizes (44px minimum), and heading hierarchy (no skips, single h1). Returns per-check results and weighted composite score (A/B/C/F grade)."
    )]
    async fn frontend_a11y_summary(
        &self,
        Parameters(params): Parameters<params::frontend::A11yAuditParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::frontend::a11y_summary(params)
    }

    // ========================================================================
    // Epidemiology Tools (11) — Domain 7, Cross-Domain Transfer to PV
    // ========================================================================

    #[tool(
        description = "Calculate Relative Risk (Risk Ratio). RR = [a/(a+b)] / [c/(c+d)]. Maps to PRR in PV. Transfer confidence: 0.95. Returns RR with 95% CI and interpretation."
    )]
    async fn epi_relative_risk(
        &self,
        Parameters(params): Parameters<params::EpiRelativeRiskParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::epidemiology::relative_risk(params)
    }

    #[tool(
        description = "Calculate Odds Ratio. OR = (a×d)/(b×c). Identical to ROR in PV. Transfer confidence: 0.98. Returns OR with 95% CI (Woolf method) and interpretation."
    )]
    async fn epi_odds_ratio(
        &self,
        Parameters(params): Parameters<params::EpiOddsRatioParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::epidemiology::odds_ratio(params)
    }

    #[tool(
        description = "Calculate Attributable Risk (Risk Difference). AR = Ie - Io = a/(a+b) - c/(c+d). Maps to excess signal rate in PV. Transfer confidence: 0.90."
    )]
    async fn epi_attributable_risk(
        &self,
        Parameters(params): Parameters<params::EpiAttributableRiskParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::epidemiology::attributable_risk(params)
    }

    #[tool(
        description = "Calculate NNT (Number Needed to Treat) or NNH (Number Needed to Harm). NNT = 1/ARR when protective, NNH = 1/ARI when harmful. Maps to benefit-risk ratio in PV. Transfer confidence: 0.85."
    )]
    async fn epi_nnt_nnh(
        &self,
        Parameters(params): Parameters<params::EpiNntNnhParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::epidemiology::nnt_nnh(params)
    }

    #[tool(
        description = "Calculate Attributable Fraction among exposed. AF = (RR-1)/RR. Maps to signal contribution fraction in PV. Transfer confidence: 0.88."
    )]
    async fn epi_attributable_fraction(
        &self,
        Parameters(params): Parameters<params::EpiAttributableFractionParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::epidemiology::attributable_fraction(params)
    }

    #[tool(
        description = "Calculate Population Attributable Fraction. PAF = Pe(RR-1)/[1+Pe(RR-1)]. Maps to population signal burden in PV. Transfer confidence: 0.85. Quantifies population-level impact."
    )]
    async fn epi_population_af(
        &self,
        Parameters(params): Parameters<params::EpiPopulationAFParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::epidemiology::population_attributable_fraction(params)
    }

    #[tool(
        description = "Calculate Incidence Rate. IR = events/person-time × multiplier. Maps to reporting rate in PV. Transfer confidence: 0.92. Includes Poisson CI."
    )]
    async fn epi_incidence_rate(
        &self,
        Parameters(params): Parameters<params::EpiIncidenceRateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::epidemiology::incidence_rate(params)
    }

    #[tool(
        description = "Calculate Point Prevalence. P = cases/population × multiplier. Maps to background rate in PV. Transfer confidence: 0.90. Includes Wilson score CI."
    )]
    async fn epi_prevalence(
        &self,
        Parameters(params): Parameters<params::EpiPrevalenceParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::epidemiology::prevalence(params)
    }

    #[tool(
        description = "Kaplan-Meier product-limit survival estimator. S(t) = Π[1-d_i/n_i]. Maps to time-to-onset survival (Weibull TTO) in PV. Transfer confidence: 0.82. Handles censoring, computes median survival, Greenwood SE."
    )]
    async fn epi_kaplan_meier(
        &self,
        Parameters(params): Parameters<params::EpiKaplanMeierParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::epidemiology::kaplan_meier(params)
    }

    #[tool(
        description = "Calculate Standardized Mortality/Morbidity Ratio. SMR = observed/expected. Maps to O/E ratio (EBGM) in PV. Transfer confidence: 0.93. Includes Byar CI."
    )]
    async fn epi_smr(
        &self,
        Parameters(params): Parameters<params::EpiSmrParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::epidemiology::smr(params)
    }

    #[tool(
        description = "Get all epidemiology → PV transfer mappings. Shows 10 epidemiology measures mapped to their PV equivalents with transfer confidence scores. Overall transfer confidence: 0.95."
    )]
    async fn epi_pv_mappings(
        &self,
        Parameters(params): Parameters<params::EpiPvMappingsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::epidemiology::epi_pv_mappings(params)
    }

    #[tool(
        description = "Make an HTTP request to any URL (curl replacement). Supports GET, POST, PUT, DELETE, PATCH, HEAD, OPTIONS. Works with localhost. Returns status, headers, and body as JSON. Use body_only=true for raw response body."
    )]
    async fn http_request(
        &self,
        Parameters(params): Parameters<params::http::HttpRequestParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::http::http_request(params).await
    }

    // ========================================================================
    // Observatory Phase 9 (3 tools) — Career, Learning DAG, Graph Layout
    // ========================================================================

    #[tool(
        description = "Compute career transition graph from KSB corpus. Uses cosine similarity over 1,286 KSB component vectors to determine transition probability between PV career roles. Returns nodes (roles with salary data) and edges (transitions with probability/difficulty). Set include_salary=true for value-mining salary signals."
    )]
    async fn career_transitions(
        &self,
        Parameters(params): Parameters<params::career::CareerTransitionsParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::career::transitions(params)
    }

    #[tool(
        description = "Resolve a learning progression DAG with completion state. Builds DAG from pathway structure, runs topological sort for levels, propagates completion state (completed/unlocked/locked), and computes height values for terrain mesh rendering. Pass user_id for personalized view."
    )]
    async fn learning_dag_resolve(
        &self,
        Parameters(params): Parameters<params::learning_dag::LearningDagResolveParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::learning_dag::resolve(params)
    }

    #[tool(
        description = "Pre-compute converged force-directed graph layout positions. Fruchterman-Reingold algorithm in 2D or 3D. Returns positioned nodes normalized to [-1,1]. Supports configurable iterations and convergence detection. Performance: <50ms for 100 nodes, <500ms for 1000 nodes."
    )]
    async fn graph_layout_converge(
        &self,
        Parameters(params): Parameters<params::graph_layout::GraphLayoutConvergeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::graph_layout::converge(params)
    }

    // ========================================================================
    // Observatory Personalization (4 tools) — detect, get, set, validate
    // ========================================================================

    #[tool(
        description = "Auto-detect optimal Observatory rendering settings from device capabilities: GPU renderer, memory, CPU cores, and accessibility media queries (prefers-reduced-motion, prefers-contrast). Returns recommended quality, theme, post-processing, and worker layout settings."
    )]
    async fn observatory_personalize_detect(
        &self,
        Parameters(params): Parameters<params::observatory::PersonalizeDetectParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::observatory::personalize_detect(params)
    }

    #[tool(
        description = "Get default Observatory personalization preferences for a named user profile. Profiles: 'default' (balanced), 'power-user' (cinematic quality, all effects), 'accessibility' (high-contrast, CVD-safe, reduced motion), 'mobile' (low quality, no effects)."
    )]
    async fn observatory_personalize_get(
        &self,
        Parameters(params): Parameters<params::observatory::PersonalizeGetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::observatory::personalize_get(params)
    }

    #[tool(
        description = "Validate and normalize an Observatory preferences object. Checks quality (low/medium/high/cinematic), theme (default/warm/clinical/high-contrast), CVD mode (normal/deuteranopia/protanopia/tritanopia), explorer, layout, and post-processing effects. Returns normalized values or validation errors."
    )]
    async fn observatory_personalize_set(
        &self,
        Parameters(params): Parameters<params::observatory::PersonalizeSetParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::observatory::personalize_set(params)
    }

    #[tool(
        description = "Cross-validate an Observatory configuration against explorer capability constraints. Detects incompatibilities (e.g. CVD mode unsupported in 'state' explorer, layout ignored in 'math' explorer) and suggests improvements (e.g. bloom with cinematic quality). Returns errors, warnings, and suggestions."
    )]
    async fn observatory_personalize_validate(
        &self,
        Parameters(params): Parameters<params::observatory::PersonalizeValidateParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::observatory::personalize_validate(params)
    }

    // ========================================================================
    // SOP-Anatomy-Code Tools (4) — Triple mapping, reactor, transfer protocol
    // ========================================================================

    #[tool(
        description = "Look up the SOP-Anatomy-Code triple mapping. Maps 18 SOP governance sections through biological anatomy to software code structures. Provide a section number (1-18) or omit for all 18."
    )]
    async fn sop_anatomy_map(
        &self,
        Parameters(params): Parameters<params::sop_anatomy::SopAnatomyMapParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::sop_anatomy::sop_anatomy_map(params)
    }

    #[tool(
        description = "Cross-domain transfer using the Capability Transfer Protocol (FISSION->CHIRALITY->FUSION->TITRATION). Translates a concept from one domain (sop/anatomy/code) to another with confidence scoring and chirality warnings."
    )]
    async fn sop_anatomy_bridge(
        &self,
        Parameters(params): Parameters<params::sop_anatomy::SopAnatomyBridgeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::sop_anatomy::sop_anatomy_bridge(params)
    }

    #[tool(
        description = "Audit a crate or project directory against all 18 SOP governance sections. Detects structural code patterns (file presence, directory existence) and scores with critical-section 2x weighting (max 25)."
    )]
    async fn sop_anatomy_audit(
        &self,
        Parameters(params): Parameters<params::sop_anatomy::SopAnatomyAuditParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::sop_anatomy::sop_anatomy_audit(params)
    }

    #[tool(
        description = "Full 18-section SOP-Anatomy-Code coverage report. Shows all sections with priority tier, weight, and bio-crate wiring status (which nexcore bio-crates implement each governance function)."
    )]
    async fn sop_anatomy_coverage(
        &self,
        Parameters(params): Parameters<params::sop_anatomy::SopAnatomyCoverageParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::sop_anatomy::sop_anatomy_coverage(params)
    }
}

// ============================================================================
// Unified mode stateful helpers (outside #[tool_router] block)
// ============================================================================
impl NexCoreMcpServer {
    /// Health check for unified dispatcher.
    pub fn unified_health(&self) -> Result<CallToolResult, McpError> {
        let tool_count = self.tool_router.list_all().len();
        let health = serde_json::json!({
            "status": "healthy",
            "server": "nexcore-mcp",
            "version": env!("CARGO_PKG_VERSION"),
            "tool_count": tool_count,
            "domains": ["foundation", "pv", "vigilance", "skills", "guidelines", "faers", "gcloud", "wolfram", "principles", "hooks"]
        });
        unified_text_result(&health)
    }

    /// Config validate for unified dispatcher.
    pub fn unified_config_validate(&self) -> Result<CallToolResult, McpError> {
        use nexcore_config::Validate;
        let msg = match &self.config {
            Some(cfg) =>
                match cfg.validate() {
                    Ok(_) =>
                        "Configuration valid: no security issues, all paths validated, MCP servers configured".to_string(),
                    Err(e) => format!("Configuration warnings:\n{e}"),
                }
            None => "No configuration loaded".to_string(),
        };
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            msg,
        )]))
    }

    /// MCP servers list for unified dispatcher.
    pub fn unified_mcp_servers_list(
        &self,
        p: params::system::McpServersListParams,
    ) -> Result<CallToolResult, McpError> {
        let servers = unified_collect_servers(&self.config, p.include_projects);
        let result = serde_json::json!({"total": servers.len(), "servers": servers});
        unified_text_result(&result)
    }

    /// MCP server get for unified dispatcher.
    pub fn unified_mcp_server_get(
        &self,
        p: params::system::McpServerGetParams,
    ) -> Result<CallToolResult, McpError> {
        let msg = unified_get_server(&self.config, &p.name);
        Ok(CallToolResult::success(vec![rmcp::model::Content::text(
            msg,
        )]))
    }
}

fn unified_text_result(v: &serde_json::Value) -> Result<CallToolResult, McpError> {
    Ok(CallToolResult::success(vec![rmcp::model::Content::text(
        serde_json::to_string_pretty(v).unwrap_or_default(),
    )]))
}

fn unified_collect_servers(
    config: &Option<nexcore_config::ClaudeConfig>,
    include_projects: bool,
) -> Vec<serde_json::Value> {
    let cfg = match config {
        Some(c) => c,
        None => {
            return Vec::new();
        }
    };
    let mut servers: Vec<serde_json::Value> = cfg
        .mcp_servers
        .iter()
        .map(|(name, sc)| unified_server_json(name, sc, "global", None))
        .collect();
    if include_projects {
        for (path, project) in &cfg.projects {
            for (name, sc) in &project.mcp_servers {
                servers.push(unified_server_json(
                    name,
                    sc,
                    "project",
                    Some(&path.display().to_string()),
                ));
            }
        }
    }
    servers
}

fn unified_server_json(
    name: &str,
    sc: &nexcore_config::McpServerConfig,
    scope: &str,
    project: Option<&str>,
) -> serde_json::Value {
    let (command, args) = match sc {
        nexcore_config::McpServerConfig::Stdio { command, args, .. } => {
            (command.clone(), args.clone())
        }
    };
    let mut v = serde_json::json!({"name": name, "scope": scope, "command": command, "args": args});
    if let Some(p) = project {
        v["project"] = serde_json::Value::String(p.to_string());
    }
    v
}

fn unified_get_server(config: &Option<nexcore_config::ClaudeConfig>, name: &str) -> String {
    let cfg = match config {
        Some(c) => c,
        None => {
            return "No configuration loaded".to_string();
        }
    };
    match cfg.mcp_servers.get(name) {
        Some(nexcore_config::McpServerConfig::Stdio { command, args, env }) => {
            let result = serde_json::json!({"name": name, "type": "stdio", "command": command, "args": args, "env": env});
            serde_json::to_string_pretty(&result).unwrap_or_default()
        }
        None => format!("MCP server '{name}' not found"),
    }
}

impl Default for NexCoreMcpServer {
    fn default() -> Self {
        Self::new()
    }
}

impl ServerHandler for NexCoreMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some(
                r#"nexcore MCP Server — 380+ Rust-powered tools via unified dispatcher.

USAGE: nexcore(command="CMD", params={...})
DISCOVER: nexcore(command="help") for full categorized catalog
SEARCH: nexcore(command="toolbox", params={query: "keyword"}) for tool lookup with parameter schemas

Domains: Foundation, PV Signal Detection, Vigilance, Guardian, Vigil, Skills, Validation, Guidelines, FAERS, GCloud, Wolfram, Principles, Brain, Hooks, Regulatory, Chemistry, STEM, Algovigilance, EditDistance, Cargo, Epidemiology, Compliance, HUD, Immunity, Watchtower, Telemetry"#
                    .into(),
            ),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "nexcore-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: Some("NexCore MCP Server".into()),
                icons: None,
                website_url: None,
            },
            ..Default::default()
        }
    }

    fn call_tool(
        &self,
        request: CallToolRequestParams,
        context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<CallToolResult, McpError>> + Send + '_ {
        async move {
            let tool_name: String = request.name.to_string();
            if let Err(err) = crate::tooling::tool_gate().check(&tool_name) {
                return Ok(crate::tooling::gated_result(&tool_name, err));
            }
            let tcc = ToolCallContext::new(self, request, context);
            let result = self.tool_router.call(tcc).await?;
            Ok(crate::tooling::wrap_result(result))
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        // Toolbox architecture: expose only the unified dispatcher + meta-tools.
        // All 380+ commands remain accessible via nexcore(command="CMD", params={...}).
        // Use nexcore(command="toolbox", params={query:"keyword"}) for discovery.
        // Use nexcore(command="help") for full catalog.
        //
        // To restore individual tool listing, set NEXCORE_MCP_ALL_TOOLS=true.
        let expose_all = std::env::var("NEXCORE_MCP_ALL_TOOLS")
            .map(|v| v == "true" || v == "1")
            .unwrap_or(false);

        std::future::ready(Ok(ListToolsResult {
            tools: if expose_all {
                self.tool_router.list_all()
            } else {
                // Toolbox mode (default): only meta-tools
                self.tool_router
                    .list_all()
                    .into_iter()
                    .filter(|t| {
                        matches!(
                            t.name.as_ref(),
                            "nexcore" | "nexcore_health_probe" | "nexcore_assist"
                        )
                    })
                    .collect()
            },
            meta: None,
            next_cursor: None,
        }))
    }
}
