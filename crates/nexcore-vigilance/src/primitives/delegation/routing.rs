//! Task Routing - T1 Mapping: TaskCharacteristics → Model

use super::{Model, confidence::DelegationConfidence};
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ErrorCost {
    Low,
    Medium,
    High,
    Critical,
}

impl ErrorCost {
    pub fn tolerance(&self) -> f64 {
        match self {
            Self::Low => 0.9,
            Self::Medium => 0.5,
            Self::High => 0.2,
            Self::Critical => 0.0,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskCharacteristics {
    pub item_count: usize,
    pub is_repetitive: bool,
    pub has_structure: bool,
    pub needs_reasoning: bool,
    pub is_novel: bool,
    pub is_sensitive: bool,
    pub is_multimodal: bool,
    pub error_cost: ErrorCost,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RoutingDecision {
    pub model: Model,
    pub confidence: f64,
    pub rationale: &'static str,
}

pub struct DelegationRouter;

impl DelegationRouter {
    /// Route task to optimal model (T1: Sequence of Mappings)
    pub fn route(task: &TaskCharacteristics) -> RoutingDecision {
        // Priority-ordered classification
        let (model, rationale) = if task.is_sensitive || task.error_cost == ErrorCost::Critical {
            (Model::ClaudeOpus, "High-stakes requires Opus")
        } else if task.item_count > 10 && task.is_repetitive && task.has_structure {
            (Model::GeminiFlash, "Bulk structured → Flash")
        } else if task.is_novel && task.needs_reasoning {
            (Model::ClaudeOpus, "Novel reasoning → Opus")
        } else if task.is_multimodal {
            (Model::GeminiPro, "Multimodal → Pro")
        } else if task.item_count > 50 {
            (Model::GeminiFlash, "High volume → Flash")
        } else {
            (Model::ClaudeSonnet, "Balanced → Sonnet")
        };

        let patterns = [task.is_repetitive, task.has_structure, task.item_count > 10]
            .iter()
            .filter(|&&b| b)
            .count();
        let confidence =
            DelegationConfidence::new(patterns, task.item_count, task.error_cost.tolerance())
                .compute()
                .total;

        RoutingDecision {
            model,
            confidence,
            rationale,
        }
    }
}
