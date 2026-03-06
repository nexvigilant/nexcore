#![forbid(unsafe_code)]
#![warn(missing_docs)]
#![cfg_attr(
    not(test),
    deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)
)]

//! Skills MCP Server — vocabulary programs as MCP tools.
//!
//! Native Rust implementations of LEARN, PROVE, VITALS programs
//! plus a generic runner for all other vocabulary programs.

pub mod tools;
pub mod types;

use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::tool::ToolCallContext;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{
    CallToolRequestParams, CallToolResult, Implementation, ListToolsResult, PaginatedRequestParams,
    ServerCapabilities, ServerInfo,
};
use rmcp::service::{RequestContext, RoleServer};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_router};

use crate::types::*;

#[derive(Clone)]
pub struct SkillsMcpServer {
    tool_router: ToolRouter<Self>,
}

#[tool_router]
impl SkillsMcpServer {
    pub fn new() -> Result<Self, nexcore_error::NexError> {
        Ok(Self {
            tool_router: Self::tool_router(),
        })
    }

    // =====================================================================
    // VITALS — Biological Infrastructure Health (6 tools + pipeline)
    // =====================================================================

    #[tool(
        description = "VITALS [V] Vigor: Check hormone state (cortisol, dopamine, serotonin, adrenaline, oxytocin, melatonin). Returns levels, staleness, and health assessment."
    )]
    async fn vitals_vigor(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vitals::vigor()
    }

    #[tool(
        description = "VITALS [I] Immunity: Query antibody registry from brain.db. Returns PAMP/DAMP counts, severity distribution, and coverage assessment."
    )]
    async fn vitals_immunity(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vitals::immunity()
    }

    #[tool(
        description = "VITALS [T] Telemetry: Check signal processing health. Returns signal count, type distribution, and receiver status."
    )]
    async fn vitals_telemetry(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vitals::telemetry()
    }

    #[tool(
        description = "VITALS [A] Antibodies: Deep-dive into antibody detection and response patterns. Returns detailed roster with threat types and confidence."
    )]
    async fn vitals_antibodies(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vitals::antibodies()
    }

    #[tool(
        description = "VITALS [L] Lifespan: Brain session persistence check. Returns session count, DB table sizes, code tracker, and implicit knowledge stats."
    )]
    async fn vitals_lifespan(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vitals::lifespan()
    }

    #[tool(
        description = "VITALS [S] Synapse: Connection health and biological score (0-100). Computes overall health grade from all subsystems."
    )]
    async fn vitals_synapse(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vitals::synapse()
    }

    #[tool(
        description = "VITALS full pipeline: Run all 6 phases (Vigor→Immunity→Telemetry→Antibodies→Lifespan→Synapse) and return combined health report with score and grade."
    )]
    async fn vitals_pipeline(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::vitals::pipeline()
    }

    // =====================================================================
    // LEARN — Knowledge Feedback Loop (5 tools + pipeline)
    // =====================================================================

    #[tool(
        description = "LEARN [L] Landscape: Survey all data sources (signals, sessions, brain.db tables, implicit knowledge, hormones). Returns counts and freshness."
    )]
    async fn learn_landscape(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::learn::landscape()
    }

    #[tool(
        description = "LEARN [E] Extract: Mine patterns from signals (blocked tools, failures), vocabulary counters (hit rate, dead weight), and brain DB. Returns extracted patterns."
    )]
    async fn learn_extract(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::learn::extract()
    }

    #[tool(
        description = "LEARN [A] Assimilate: Write extracted patterns to implicit/patterns.json and brain.db patterns table. Deduplicates by type+source."
    )]
    async fn learn_assimilate(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::learn::assimilate()
    }

    #[tool(
        description = "LEARN [R] Recall: Verify knowledge loads correctly. Tests MEMORY.md, preferences, patterns, sessions, antibodies, and hook-knowledge bridge. Returns recall score (N/6)."
    )]
    async fn learn_recall(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::learn::recall()
    }

    #[tool(
        description = "LEARN [N] Normalize: Prune dead weight vocabulary, deduplicate patterns, rotate large signal files. Returns items pruned."
    )]
    async fn learn_normalize(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::learn::normalize()
    }

    #[tool(
        description = "LEARN full pipeline: Run all 5 phases (Landscape→Extract→Assimilate→Recall→Normalize) and return combined learning report."
    )]
    async fn learn_pipeline(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::learn::pipeline()
    }

    // =====================================================================
    // PROVE — Self-Verification (4 tools + pipeline, excludes R=Run)
    // =====================================================================

    #[tool(
        description = "PROVE [P] Prepare: Check Claude CLI, MCP binary, brain DB, settings.json. Capture baseline snapshot."
    )]
    async fn prove_prepare(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::prove::prepare()
    }

    #[tool(
        description = "PROVE [O] Observe: Parse the most recent sub-Claude run output into structured findings (brain, immunity, hormones status)."
    )]
    async fn prove_observe(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::prove::observe()
    }

    #[tool(
        description = "PROVE [V] Validate: Compare observed findings against baselines. Returns pass/fail for each check."
    )]
    async fn prove_validate(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::prove::validate()
    }

    #[tool(
        description = "PROVE [E] Evaluate: Score and track improvement over time. Appends to history, computes trend (IMPROVING/STABLE/REGRESSING)."
    )]
    async fn prove_evaluate(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::prove::evaluate()
    }

    // =====================================================================
    // Generic Skill Runner — Execute any vocabulary program script
    // =====================================================================

    #[tool(
        description = "Run any vocabulary program or individual phase. Programs: SMART, BRAIN, GUARD, PULSE, FORGE, SCOPE, CLEAN, AUDIT, CRAFT, TRACE. Use program='clean' and phase='ALL' for full pipeline, or phase='C' for a single letter."
    )]
    async fn skill_run(
        &self,
        Parameters(params): Parameters<SkillRunParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::runner::run_skill(params).await
    }

    #[tool(
        description = "List all available vocabulary programs with their phases and descriptions."
    )]
    async fn skill_list(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::runner::list_skills()
    }

    // =====================================================================
    // PRIMITIVES — T1 Lex Primitiva Cognitive Tools
    // =====================================================================

    #[tool(
        description = "Decompose a concept, problem, or code pattern into its T1 Lex Primitiva components. Returns identified primitives with relevance scores, dominant primitive, tier classification, and composition formula. Examples: 'rate limiter', 'cache invalidation', 'login authentication', 'HashMap<K,V>'."
    )]
    async fn primitive_decompose(
        &self,
        Parameters(params): Parameters<PrimitiveDecomposeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::primitives::decompose(params)
    }

    #[tool(
        description = "Given T1 primitive names or symbols, describe what they compose into. Returns known patterns, tier classification, and Rust implementation suggestions. Examples: ['Sequence', 'Mapping', 'Boundary'] or ['σ', 'μ', '∂']."
    )]
    async fn primitive_compose(
        &self,
        Parameters(params): Parameters<PrimitiveComposeParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::primitives::compose(params)
    }

    #[tool(
        description = "List all 16 T1 Lex Primitiva symbols with their meanings and Rust manifestations. The irreducible universal primitives that ground all higher-tier types."
    )]
    async fn primitive_list(
        &self,
        #[allow(unused)] Parameters(params): Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        tools::primitives::list_all()
    }
}

impl ServerHandler for SkillsMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            instructions: Some("Skills MCP: vocabulary programs as tools. LEARN (feedback loop), PROVE (self-verification), VITALS (biological health), plus generic runner for SMART/BRAIN/GUARD/PULSE/FORGE/SCOPE/CLEAN/AUDIT/CRAFT/TRACE.".into()),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "skills-mcp".into(),
                version: env!("CARGO_PKG_VERSION").into(),
                title: Some("Skills MCP Server".into()),
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
            let tcc = ToolCallContext::new(self, request, context);
            let result = self.tool_router.call(tcc).await?;
            Ok(result)
        }
    }

    fn list_tools(
        &self,
        _request: Option<PaginatedRequestParams>,
        _context: RequestContext<RoleServer>,
    ) -> impl std::future::Future<Output = Result<ListToolsResult, McpError>> + Send + '_ {
        std::future::ready(Ok(ListToolsResult {
            tools: self.tool_router.list_all(),
            meta: None,
            next_cursor: None,
        }))
    }
}
