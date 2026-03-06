//! Station MCP tools — WebMCP Hub config rail management.
//!
//! Build, manage, and export NexVigilant configs for the MoltBrowser hub.
//! Each config is a rail that routes AI agent traffic through NexVigilant tooling.
//!
//! ## Tools
//! - `station_build_config`: Create a config for a PV vertical
//! - `station_add_tool`: Add a tool to a config
//! - `station_list`: List all configs in the registry
//! - `station_export`: Export config as MoltBrowser JSON payloads
//! - `station_coverage`: Show vertical coverage report
//! - `station_verticals`: List all PV verticals and their domains
//!
//! ## Primitive Foundation
//! - N (Quantity): Coverage metrics, tool counts
//! - μ (Mapping): Vertical → domain → config
//! - → (Transition): Config → MoltBrowser payload
//! - ∂ (Boundary): Access tier gating

use crate::params::station::{
    StationAddToolParams, StationBuildConfigParams, StationCoverageParams, StationExportParams,
    StationListParams, StationResolveParams,
};
use nexcore_station::{
    AccessTier, ExecutionType, PvVertical, ResolutionRequest, StationBuilder, StationClient,
    StationConfig, StationRegistry, StationTool, StubObservatoryFeed,
};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};
use serde_json::json;
use std::sync::Mutex;

// In-memory registry for the session. Persisted via brain artifacts.
static REGISTRY: Mutex<Option<StationRegistry>> = Mutex::new(None);

fn get_registry() -> StationRegistry {
    let guard = REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    guard.clone().unwrap_or_default()
}

fn set_registry(reg: StationRegistry) {
    let mut guard = REGISTRY.lock().unwrap_or_else(|e| e.into_inner());
    *guard = Some(reg);
}

fn parse_vertical(name: &str) -> Result<PvVertical, McpError> {
    match name.to_lowercase().as_str() {
        "platform" => Ok(PvVertical::Platform),
        "faers" => Ok(PvVertical::Faers),
        "dailymed" | "daily_med" => Ok(PvVertical::DailyMed),
        "eudravigilance" | "eudra_vigilance" => Ok(PvVertical::EudraVigilance),
        "vigibase" | "vigi_base" | "vigiaccess" => Ok(PvVertical::VigiBase),
        "meddra" | "med_dra" => Ok(PvVertical::MedDra),
        "clinical_trials" | "clinicaltrials" => Ok(PvVertical::ClinicalTrials),
        "pubmed" | "pub_med" => Ok(PvVertical::PubMed),
        "ich" => Ok(PvVertical::Ich),
        "ema" => Ok(PvVertical::Ema),
        "fda" => Ok(PvVertical::Fda),
        _ => Err(McpError::invalid_params(
            format!(
                "Unknown vertical: '{name}'. Valid: platform, faers, dailymed, eudravigilance, vigibase, meddra, clinical_trials, pubmed, ich, ema, fda"
            ),
            None,
        )),
    }
}

fn parse_execution_type(s: &str) -> Result<ExecutionType, McpError> {
    match s.to_lowercase().as_str() {
        "extract" => Ok(ExecutionType::Extract),
        "navigate" => Ok(ExecutionType::Navigate),
        "fill" => Ok(ExecutionType::Fill),
        "click" => Ok(ExecutionType::Click),
        _ => Err(McpError::invalid_params(
            format!("Unknown execution type: '{s}'. Valid: extract, navigate, fill, click"),
            None,
        )),
    }
}

/// Build a new station config for a PV vertical.
pub fn station_build_config(params: StationBuildConfigParams) -> Result<CallToolResult, McpError> {
    let vertical = parse_vertical(&params.vertical)?;
    let config = StationBuilder::new(vertical, &params.title)
        .description(&params.description)
        .build();

    let domain = config.domain.clone();
    let mut reg = get_registry();
    reg.add(config);
    set_registry(reg);

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "status": "created",
            "vertical": params.vertical,
            "domain": domain,
            "title": params.title,
            "message": format!("Station config created for {} ({}). Add tools with station_add_tool.", params.vertical, domain)
        })
        .to_string(),
    )]))
}

/// Add a tool to an existing station config.
pub fn station_add_tool(params: StationAddToolParams) -> Result<CallToolResult, McpError> {
    let vertical = parse_vertical(&params.vertical)?;
    let exec_type = parse_execution_type(&params.execution_type)?;

    let mut reg = get_registry();
    let config = reg
        .configs
        .iter_mut()
        .find(|c| c.vertical == vertical)
        .ok_or_else(|| {
            McpError::invalid_params(
                format!(
                    "No config for vertical '{}'. Create one first with station_build_config.",
                    params.vertical
                ),
                None,
            )
        })?;

    config.tools.push(StationTool {
        name: params.name.clone(),
        description: params.description.clone(),
        route: params.route.clone(),
        execution_type: exec_type,
        access_tier: AccessTier::Public,
        input_schema: None,
        tags: Vec::new(),
    });

    let tool_count = config.total_tools();
    set_registry(reg);

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "status": "added",
            "tool": params.name,
            "vertical": params.vertical,
            "total_tools": tool_count,
        })
        .to_string(),
    )]))
}

/// List all station configs in the registry.
pub fn station_list(params: StationListParams) -> Result<CallToolResult, McpError> {
    let reg = get_registry();

    let configs: Vec<&StationConfig> = if let Some(ref v) = params.vertical {
        let vertical = parse_vertical(v)?;
        reg.by_vertical(vertical)
    } else {
        reg.configs.iter().collect()
    };

    let result: Vec<serde_json::Value> = configs
        .iter()
        .map(|c| {
            json!({
                "vertical": format!("{:?}", c.vertical),
                "domain": c.domain,
                "title": c.title,
                "total_tools": c.total_tools(),
                "public_tools": c.public_tool_count(),
                "gated_tools": c.gated_tool_count(),
                "premium_tools": c.premium_tool_count(),
                "tools": c.tools.iter().map(|t| json!({
                    "name": t.name,
                    "type": format!("{:?}", t.execution_type),
                    "route": t.route,
                })).collect::<Vec<_>>(),
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "configs": result,
            "total_configs": configs.len(),
            "total_tools": configs.iter().map(|c| c.total_tools()).sum::<usize>(),
        })
        .to_string(),
    )]))
}

/// Export config as MoltBrowser contribute payloads.
pub fn station_export(params: StationExportParams) -> Result<CallToolResult, McpError> {
    let vertical = parse_vertical(&params.vertical)?;
    let reg = get_registry();

    let config = reg.by_domain(vertical.domain()).ok_or_else(|| {
        McpError::invalid_params(
            format!(
                "No config for vertical '{}'. Create one first.",
                params.vertical
            ),
            None,
        )
    })?;

    let builder = StationBuilder::new(vertical, &config.title).description(&config.description);

    let create_payload = builder.to_moltbrowser_create();
    let config_id = params.config_id.as_deref().unwrap_or("<CONFIG_ID>");
    let tool_payloads = config
        .tools
        .iter()
        .map(|t| {
            json!({
                "configId": config_id,
                "name": t.name,
                "description": t.description,
            })
        })
        .collect::<Vec<_>>();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "create_config_payload": create_payload,
            "add_tool_payloads": tool_payloads,
            "instructions": format!(
                "1. Call contribute_create-config with create_config_payload\n\
                 2. Get configId from response\n\
                 3. Call contribute_add-tool for each tool payload (replace <CONFIG_ID>)\n\
                 4. Total tools to publish: {}",
                tool_payloads.len()
            ),
        })
        .to_string(),
    )]))
}

/// Show vertical coverage report — what's owned, what's uncovered.
pub fn station_coverage(_params: StationCoverageParams) -> Result<CallToolResult, McpError> {
    let reg = get_registry();
    let covered = reg.covered_verticals();
    let uncovered = reg.uncovered_verticals();

    let verticals: Vec<serde_json::Value> = PvVertical::all()
        .iter()
        .map(|v| {
            let is_covered = covered.contains(v);
            let tool_count = if is_covered {
                reg.by_vertical(*v)
                    .iter()
                    .map(|c| c.total_tools())
                    .sum::<usize>()
            } else {
                0
            };
            json!({
                "vertical": format!("{v:?}"),
                "domain": v.domain(),
                "covered": is_covered,
                "tools": tool_count,
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({
            "coverage_ratio": format!("{:.0}%", reg.coverage_ratio() * 100.0),
            "covered": covered.len(),
            "uncovered": uncovered.len(),
            "total_verticals": PvVertical::all().len(),
            "total_configs": reg.config_count(),
            "total_tools": reg.total_tools(),
            "verticals": verticals,
        })
        .to_string(),
    )]))
}

/// List all available PV verticals and their canonical domains.
pub fn station_verticals() -> Result<CallToolResult, McpError> {
    let verticals: Vec<serde_json::Value> = PvVertical::all()
        .iter()
        .map(|v| {
            json!({
                "name": format!("{v:?}"),
                "domain": v.domain(),
            })
        })
        .collect();

    Ok(CallToolResult::success(vec![Content::text(
        json!({ "verticals": verticals }).to_string(),
    )]))
}

/// Resolve the best tool for a domain via StationClient.
///
/// Instantiates StationClient with StubObservatoryFeed (production-safe default),
/// queries the registry for the domain, computes confidence via Observatory metrics,
/// and returns a JSON-serialized ResolutionResponse.
pub fn station_resolve(params: StationResolveParams) -> Result<CallToolResult, McpError> {
    let registry = get_registry();
    let feed = StubObservatoryFeed;
    let client = StationClient::new(registry, feed);

    let request = ResolutionRequest {
        domain: params.domain.clone(),
        task_hint: params.task_hint,
    };

    match client.resolve(&request) {
        Ok(response) => {
            let result = json!({
                "success": true,
                "domain": params.domain,
                "tool_name": response.tool_name,
                "config": response.config,
                "confidence": response.confidence.value,
                "trust_tier": format!("{:?}", response.trust_tier),
                "verified_at": response.verified_at,
                "gap": response.gap.map(|g| json!({
                    "domain": g.domain,
                    "priority": g.priority,
                    "reason": g.reason,
                })),
            });
            Ok(CallToolResult::success(vec![Content::text(
                result.to_string(),
            )]))
        }
        Err(e) => {
            let result = json!({
                "success": false,
                "domain": params.domain,
                "error": e.to_string(),
            });
            Ok(CallToolResult::error(vec![Content::text(
                result.to_string(),
            )]))
        }
    }
}
