//! Transfer mappings for nexcore-mcp
//!
//! Cross-domain transfer between AI Service, Pharmacovigilance, and Biology.

use serde::{Deserialize, Serialize};

/// A cross-domain transfer mapping for nexcore-mcp concepts.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TransferMapping {
    /// Source type name
    pub source_type: &'static str,
    /// Target domain
    pub domain: &'static str,
    /// Analogous concept in target domain
    pub analog: &'static str,
    /// Transfer confidence (0.0-1.0)
    pub confidence: f64,
}

/// All cross-domain transfer mappings for nexcore-mcp.
pub fn transfer_mappings() -> Vec<TransferMapping> {
    vec![
        // ========== NexCoreMcpServer (ς+μ+π) ==========
        TransferMapping {
            source_type: "NexCoreMcpServer",
            domain: "PV",
            analog: "VigilanceGateway (sensory-actuator interface)",
            confidence: 0.85,
        },
        TransferMapping {
            source_type: "NexCoreMcpServer",
            domain: "Biology",
            analog: "CellMembrane (signal regulation and homeostasis)",
            confidence: 0.75,
        },
        TransferMapping {
            source_type: "NexCoreMcpServer",
            domain: "Economics",
            analog: "ClearingHouse (routing and resolving requests)",
            confidence: 0.70,
        },
        // ========== UnifiedCommand (μ+×) ==========
        TransferMapping {
            source_type: "UnifiedCommand",
            domain: "PV",
            analog: "SignalTriageRequest",
            confidence: 0.80,
        },
        TransferMapping {
            source_type: "UnifiedCommand",
            domain: "Biology",
            analog: "HormoneSignal (targeted message and payload)",
            confidence: 0.70,
        },
        // ========== McpServerStatus (ς+N) ==========
        TransferMapping {
            source_type: "McpServerStatus",
            domain: "Biology",
            analog: "OrganHealth (functional status and capacity)",
            confidence: 0.75,
        },
        // ========== AgentLock (∂+λ+ς) ==========
        TransferMapping {
            source_type: "AgentLock",
            domain: "Biology",
            analog: "EnzymeInhibition (reversible binding to active site)",
            confidence: 0.65,
        },
    ]
}
