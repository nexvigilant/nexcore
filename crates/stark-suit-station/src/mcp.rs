//! MCP server — exposes the 4 compound snapshots as tools.
//!
//! Tools:
//!   - `stark_suit_status`           — whole-station snapshot
//!   - `stark_suit_perception_world` — perception compound
//!   - `stark_suit_power_status`     — power compound
//!   - `stark_suit_control_command`  — control compound
//!   - `stark_suit_human_interface`  — HI compound

use crate::state::StationState;
use rmcp::handler::server::router::tool::ToolRouter;
use rmcp::handler::server::wrapper::Parameters;
use rmcp::model::{CallToolResult, Content, Implementation, ServerCapabilities, ServerInfo};
use rmcp::{ErrorData as McpError, ServerHandler, tool, tool_router};
use std::sync::Arc;

#[derive(serde::Deserialize, schemars::JsonSchema, Default)]
#[serde(crate = "rmcp::serde")]
pub struct EmptyParams {}

#[derive(Clone)]
pub struct StarkSuitMcpServer {
    tool_router: ToolRouter<Self>,
    state: Arc<StationState>,
}

#[tool_router]
impl StarkSuitMcpServer {
    /// Construct the server bound to a shared StationState.
    pub fn new(state: Arc<StationState>) -> Self {
        Self {
            tool_router: Self::tool_router(),
            state,
        }
    }

    /// Whole-station snapshot across all 4 compounds.
    #[tool(description = "Returns the latest snapshot of all 4 stark-suit compounds (perception, power, control, human_interface) plus aggregate tick count.")]
    async fn stark_suit_status(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let snap = self.state.snapshot().await;
        json_result(&snap)
    }

    /// Perception compound — world state.
    #[tool(description = "Returns the latest perception snapshot: tick, heading_rad, altitude_m, classified intent label.")]
    async fn stark_suit_perception_world(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let snap = self.state.perception.read().await.clone();
        json_result(&snap)
    }

    /// Power compound — SOC + load tier + degradation state.
    #[tool(description = "Returns the latest power snapshot: tick, soc_pct, health, current_tier, power_state.")]
    async fn stark_suit_power_status(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let snap = self.state.power.read().await.clone();
        json_result(&snap)
    }

    /// Control compound — last flight command.
    #[tool(description = "Returns the latest control snapshot: tick, target_vector [x,y,z].")]
    async fn stark_suit_control_command(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let snap = self.state.control.read().await.clone();
        json_result(&snap)
    }

    /// Human-interface compound — safety + thermal.
    #[tool(description = "Returns the latest human-interface snapshot: tick, estop_status, thermal_action, watchdog_kicks.")]
    async fn stark_suit_human_interface(
        &self,
        _params: Parameters<EmptyParams>,
    ) -> Result<CallToolResult, McpError> {
        let snap = self.state.human_interface.read().await.clone();
        json_result(&snap)
    }
}

impl ServerHandler for StarkSuitMcpServer {
    fn get_info(&self) -> ServerInfo {
        ServerInfo {
            protocol_version: rmcp::model::ProtocolVersion::default(),
            capabilities: ServerCapabilities::builder().enable_tools().build(),
            server_info: Implementation {
                name: "stark-suit-station".into(),
                title: Some("Stark Suit Station".into()),
                version: env!("CARGO_PKG_VERSION").into(),
                website_url: None,
                icons: None,
            },
            instructions: Some(
                "Iron Vigil Stark Suit station — 4 compound control loops (perception/power/control/human_interface). Use stark_suit_status for the whole-station snapshot or per-compound tools for targeted reads.".into(),
            ),
        }
    }
}

fn json_result<T: serde::Serialize>(value: &T) -> Result<CallToolResult, McpError> {
    let s = serde_json::to_string(value)
        .map_err(|e| McpError::internal_error(format!("serialize: {e}"), None))?;
    Ok(CallToolResult::success(vec![Content::text(s)]))
}
