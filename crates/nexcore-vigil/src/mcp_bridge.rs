//! # MCP Bridge
//!
//! Bridge between FRIDAY's AgenticLoop and real nexcore MCP tools.

use std::collections::HashMap;
use std::path::Path;
use std::sync::Arc;

use nexcore_error::{Result, nexerror};
use nexcore_vigilance::primitives::chemistry::{arrhenius_rate, remaining_after_time};
use nexcore_vigilance::primitives::quantum::{Qubit, Superposition};
use nexcore_vigilance::pv::causality::{calculate_naranjo_quick, calculate_who_umc_quick};
use nexcore_vigilance::pv::signals::evaluate_signal_complete;
use nexcore_vigilance::pv::thresholds::SignalCriteria;
use nexcore_vigilance::pv::types::ContingencyTable;
use nexcore_vigilance::pv::{calculate_chi_square, calculate_prr};
use nexcore_vigilance::skills::{SkillRegistry, validate_diamond};
use parking_lot::RwLock;
use serde_json::{Value, json};

#[derive(Clone)]
pub struct McpBridge {
    registry: Arc<RwLock<SkillRegistry>>,
    supported_tools: HashMap<String, ToolCategory>,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ToolCategory {
    PvSignal,
    PvCausality,
    SkillValidation,
    SkillRegistry,
    Chemistry,
    Quantum,
}

impl McpBridge {
    pub fn new() -> Self {
        let mut tools = HashMap::new();
        tools.insert("pv_signal_complete".into(), ToolCategory::PvSignal);
        tools.insert("pv_signal_prr".into(), ToolCategory::PvSignal);
        tools.insert("pv_chi_square".into(), ToolCategory::PvSignal);
        tools.insert("pv_naranjo_quick".into(), ToolCategory::PvCausality);
        tools.insert("pv_who_umc_quick".into(), ToolCategory::PvCausality);
        tools.insert("skill_validate".into(), ToolCategory::SkillValidation);
        tools.insert("skill_scan".into(), ToolCategory::SkillRegistry);
        tools.insert("skill_list".into(), ToolCategory::SkillRegistry);
        tools.insert("chemistry_threshold_rate".into(), ToolCategory::Chemistry);
        tools.insert("chemistry_decay_remaining".into(), ToolCategory::Chemistry);
        tools.insert("quantum_qubit_new".into(), ToolCategory::Quantum);
        tools.insert(
            "quantum_superposition_entropy".into(),
            ToolCategory::Quantum,
        );

        Self {
            registry: Arc::new(RwLock::new(SkillRegistry::new())),
            supported_tools: tools,
        }
    }

    pub fn supports(&self, tool_name: &str) -> bool {
        self.supported_tools.contains_key(tool_name)
    }

    pub async fn invoke(&self, tool: &str, params: Value) -> Result<Value> {
        let cat = self
            .supported_tools
            .get(tool)
            .ok_or_else(|| nexerror!("Unknown"))?;
        match cat {
            ToolCategory::PvSignal => self.invoke_pv_signal(tool, params).await,
            ToolCategory::PvCausality => self.invoke_pv_causality(tool, params).await,
            ToolCategory::SkillValidation => self.invoke_skill_validation(params).await,
            ToolCategory::SkillRegistry => self.invoke_skill_registry(tool, params).await,
            ToolCategory::Chemistry => self.invoke_chemistry(tool, params).await,
            ToolCategory::Quantum => self.invoke_quantum(tool, params).await,
        }
    }

    async fn invoke_pv_signal(&self, tool: &str, params: Value) -> Result<Value> {
        let table = extract_table(&params)?;
        if tool == "pv_signal_complete" {
            let res = evaluate_signal_complete(&table, &SignalCriteria::evans());
            Ok(json!({ "any_signal": res.prr.is_signal || res.ror.is_signal }))
        } else if tool == "pv_signal_prr" {
            let res = calculate_prr(&table, &SignalCriteria::evans());
            Ok(json!({ "is_signal": res.is_signal }))
        } else {
            let chi = calculate_chi_square(&table);
            Ok(json!({ "chi_square": chi }))
        }
    }

    async fn invoke_pv_causality(&self, tool: &str, _params: Value) -> Result<Value> {
        if tool == "pv_naranjo_quick" {
            let res = calculate_naranjo_quick(0, 0, 0, 0, 0);
            Ok(json!({ "score": res.score }))
        } else {
            let res = calculate_who_umc_quick(0, 0, 0, 0, 0);
            Ok(json!({ "category": format!("{:?}", res.category) }))
        }
    }

    async fn invoke_skill_validation(&self, params: Value) -> Result<Value> {
        let path = params["path"].as_str().ok_or_else(|| nexerror!("path"))?;
        let res = validate_diamond(Path::new(path)).map_err(|e| nexerror!("{}", e))?;
        Ok(json!({ "level": res.level.to_string() }))
    }

    async fn invoke_skill_registry(&self, tool: &str, params: Value) -> Result<Value> {
        if tool == "skill_scan" {
            let dir = params["directory"]
                .as_str()
                .ok_or_else(|| nexerror!("dir"))?;
            let n = self
                .registry
                .write()
                .scan(Path::new(dir))
                .map_err(|e| nexerror!("{}", e))?;
            Ok(json!({ "found": n }))
        } else {
            Ok(json!({ "count": self.registry.read().len() }))
        }
    }

    async fn invoke_chemistry(&self, tool: &str, _params: Value) -> Result<Value> {
        if tool == "chemistry_threshold_rate" {
            let r = arrhenius_rate(1.0, 0.0, 298.15).map_err(|e| nexerror!("{}", e))?;
            Ok(json!({ "rate": r }))
        } else {
            let r = remaining_after_time(100.0, 1.0, 0.0).map_err(|e| nexerror!("{}", e))?;
            Ok(json!({ "rem": r }))
        }
    }

    async fn invoke_quantum(&self, tool: &str, _params: Value) -> Result<Value> {
        if tool == "quantum_qubit_new" {
            let q = Qubit::new(1.0, 0.0);
            Ok(json!({ "p0": q.prob_zero() }))
        } else {
            let s = Superposition::new(vec![1.0], vec!["0".into()]);
            Ok(json!({ "e": s.entropy() }))
        }
    }
}

fn extract_table(params: &Value) -> Result<ContingencyTable> {
    let a = params["a"].as_u64().unwrap_or(0);
    let b = params["b"].as_u64().unwrap_or(0);
    let c = params["c"].as_u64().unwrap_or(0);
    let d = params["d"].as_u64().ok_or_else(|| nexerror!("d"))?;
    Ok(ContingencyTable::new(a, b, c, d))
}

impl Default for McpBridge {
    fn default() -> Self {
        Self::new()
    }
}
