//! Token-as-energy MCP tools — ATP/ADP biochemistry for token budget management.
//!
//! # T1 Grounding
//! - N (quantity): Token pools are countable resources
//! - κ (comparison): Regime classification, strategy selection
//! - ∝ (proportionality): Energy charge, coupling ratio

use nexcore_energy::{EnergySystem, Operation, Regime, TokenPool, decide, snapshot};
use rmcp::ErrorData as McpError;
use rmcp::model::{CallToolResult, Content};

use crate::params::{EnergyChargeParams, EnergyDecideParams};

/// Compute energy charge and full state snapshot for a token pool.
pub fn energy_charge(params: EnergyChargeParams) -> Result<CallToolResult, McpError> {
    let mut pool = TokenPool::new(params.budget);

    if let Some(productive) = params.productive_spent {
        pool.spend_productive(productive);
    }
    if let Some(wasted) = params.wasted {
        pool.spend_waste(wasted);
    }

    let total_value = params.total_value.unwrap_or(0.0);
    let state = snapshot(&pool, total_value);

    let response = serde_json::json!({
        "energy_charge": state.energy_charge,
        "regime": format!("{}", state.regime),
        "recommended_strategy": format!("{}", state.recommended_strategy),
        "energy_system": format!("{}", state.energy_system),
        "pool": {
            "t_atp": pool.t_atp,
            "t_adp": pool.t_adp,
            "t_amp": pool.t_amp,
            "total": pool.total(),
        },
        "metrics": {
            "waste_ratio": state.waste_ratio,
            "burn_rate": state.burn_rate,
            "coupling_efficiency": state.coupling_efficiency,
        },
        "display": format!("{state}"),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}

/// Decide the optimal strategy for a specific operation given current energy state.
pub fn energy_decide(params: EnergyDecideParams) -> Result<CallToolResult, McpError> {
    let mut pool = TokenPool::new(params.budget);

    if let Some(productive) = params.productive_spent {
        pool.spend_productive(productive);
    }
    if let Some(wasted) = params.wasted {
        pool.spend_waste(wasted);
    }

    let op = Operation::builder(&params.operation_label)
        .cost(params.estimated_cost)
        .value(params.estimated_value);
    let op = if params.cache_possible.unwrap_or(false) {
        op.cacheable()
    } else {
        op
    };
    let op = op.build();

    let strategy = decide(&pool, &op);
    let system = EnergySystem::for_strategy(strategy);
    let regime = Regime::from_ec(pool.energy_charge());

    let response = serde_json::json!({
        "strategy": format!("{strategy}"),
        "energy_system": format!("{system}"),
        "regime": format!("{regime}"),
        "energy_charge": pool.energy_charge(),
        "operation": {
            "label": op.label,
            "estimated_cost": op.estimated_cost,
            "estimated_value": op.estimated_value,
            "coupling_ratio": op.coupling_ratio(),
            "cache_possible": op.cache_possible,
        },
        "cost_multiplier": strategy.cost_multiplier(),
        "allows_expensive": regime.allows_expensive(),
    });
    Ok(CallToolResult::success(vec![Content::text(
        response.to_string(),
    )]))
}
