//! # Mitotic Auto-Repair
//!
//! Implements code "healing" through the replication and replacement of faulty "Cells".

use serde::{Deserialize, Serialize};

/// Represents a "failing cell" (a crate or module with errors).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FailingCell {
    pub name: String,
    pub error_type: String,
    pub severity: f64,
}

/// The state of a mitotic repair cycle.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MitosisState {
    /// Identifying the failure.
    Detection,
    /// Loading a known-good "Stem Cell" snapshot.
    StemCellLoading,
    /// Spawning a specialized agent for repair.
    Regeneration,
    /// Birth of the repaired cell.
    Verification,
    /// Cell successfully integrated.
    Integrated,
}

/// A mitotic repair event.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MitoticRepair {
    pub cell: FailingCell,
    pub state: MitosisState,
    pub attempts: usize,
}

impl MitoticRepair {
    pub fn new(cell: FailingCell) -> Self {
        Self {
            cell,
            state: MitosisState::Detection,
            attempts: 0,
        }
    }

    /// Advances the repair cycle.
    pub fn advance(&mut self) {
        self.state = match self.state {
            MitosisState::Detection => MitosisState::StemCellLoading,
            MitosisState::StemCellLoading => MitosisState::Regeneration,
            MitosisState::Regeneration => MitosisState::Verification,
            MitosisState::Verification => MitosisState::Integrated,
            MitosisState::Integrated => MitosisState::Integrated,
        };
    }
}
