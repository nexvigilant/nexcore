//! Differentiation - irreversible state specialization.
//!
//! ## T1 Grounding
//!
//! - `CellType` → Σ (sum type) - categorical specialization
//! - `DifferentiationPath` → ∂ (boundary) - point of no return
//! - `Stimulus → CellType` → μ (mapping) - signal determines specialization
//!
//! ## Biological Analog
//!
//! Stem cells differentiate into specialized cell types:
//! 1. **Totipotent**: Can become anything (embryonic stem cell)
//! 2. **Multipotent**: Can become several types (hematopoietic stem cell)
//! 3. **Specialized**: Terminal differentiation (T-cell, B-cell, macrophage)
//! 4. **Irreversible**: Once specialized, cannot de-differentiate (normally)
//!
//! ## Claude Code Analog
//!
//! Component specialization from generic to specific:
//! - **Undifferentiated**: Generic agent (can handle any task)
//! - **Stimulus**: Task characteristics determine specialization
//! - **Differentiation**: Agent configures for specific role
//! - **Committed**: Specialized agent cannot switch roles mid-task

use serde::{Deserialize, Serialize};

/// Potency level — how many specializations are still possible.
///
/// # Tier: T2-P
/// Grounds to: N (quantity — number of possible fates)
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Potency {
    /// Can become any cell type
    Totipotent = 4,
    /// Can become several cell types
    Pluripotent = 3,
    /// Can become a few related types
    Multipotent = 2,
    /// Can become one specific type
    Unipotent = 1,
    /// Fully specialized, no further differentiation
    Terminal = 0,
}

impl Potency {
    /// Check if further differentiation is possible.
    pub fn can_differentiate(&self) -> bool {
        *self != Self::Terminal
    }
}

/// A specialization that a cell can differentiate into.
///
/// # Tier: T2-P
/// Grounds to: μ (mapping — name to capability set)
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub struct Specialization {
    /// Name of the specialization
    pub name: String,
    /// Capabilities granted by this specialization
    pub capabilities: Vec<String>,
    /// Potency after taking this path
    pub resulting_potency: Potency,
}

impl Specialization {
    /// Create a new specialization.
    pub fn new(
        name: impl Into<String>,
        capabilities: Vec<String>,
        resulting_potency: Potency,
    ) -> Self {
        Self {
            name: name.into(),
            capabilities,
            resulting_potency,
        }
    }

    /// Create a terminal specialization (no further differentiation).
    pub fn terminal(name: impl Into<String>, capabilities: Vec<String>) -> Self {
        Self::new(name, capabilities, Potency::Terminal)
    }
}

/// A cell that can undergo differentiation.
///
/// # Tier: T2-C (Composite)
/// Grounds to: ∂ (boundary — irreversible transitions) + μ (mapping — stimulus to type)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DifferentiableCell {
    /// Cell identifier
    pub id: String,
    /// Current potency
    pub potency: Potency,
    /// Current specialization (None if undifferentiated)
    pub specialization: Option<String>,
    /// Accumulated capabilities from all differentiations
    pub capabilities: Vec<String>,
    /// History of differentiation steps (irreversible record)
    pub differentiation_history: Vec<String>,
    /// Available specialization paths from current state
    available_paths: Vec<Specialization>,
}

impl DifferentiableCell {
    /// Create a new totipotent cell.
    pub fn totipotent(id: impl Into<String>) -> Self {
        Self {
            id: id.into(),
            potency: Potency::Totipotent,
            specialization: None,
            capabilities: Vec::new(),
            differentiation_history: Vec::new(),
            available_paths: Vec::new(),
        }
    }

    /// Create a multipotent cell with specific paths.
    pub fn multipotent(id: impl Into<String>, paths: Vec<Specialization>) -> Self {
        Self {
            id: id.into(),
            potency: Potency::Multipotent,
            specialization: None,
            capabilities: Vec::new(),
            differentiation_history: Vec::new(),
            available_paths: paths,
        }
    }

    /// Add available differentiation paths.
    #[must_use]
    pub fn with_paths(mut self, paths: Vec<Specialization>) -> Self {
        self.available_paths = paths;
        self
    }

    /// Get available specialization paths.
    pub fn available_paths(&self) -> &[Specialization] {
        &self.available_paths
    }

    /// Check if differentiation is possible.
    pub fn can_differentiate(&self) -> bool {
        self.potency.can_differentiate() && !self.available_paths.is_empty()
    }

    /// Attempt to differentiate along a named path.
    ///
    /// This is **irreversible** — the cell gains capabilities but loses potency.
    pub fn differentiate(&mut self, path_name: &str) -> Result<(), DifferentiationError> {
        if !self.potency.can_differentiate() {
            return Err(DifferentiationError::AlreadyTerminal);
        }

        let path = self
            .available_paths
            .iter()
            .find(|p| p.name == path_name)
            .cloned();

        match path {
            Some(spec) => {
                // Apply specialization (irreversible)
                self.capabilities.extend(spec.capabilities.clone());
                self.specialization = Some(spec.name.clone());
                self.potency = spec.resulting_potency;
                self.differentiation_history.push(spec.name);

                // Remove paths that are no longer available
                // (differentiation narrows future options)
                self.available_paths
                    .retain(|p| p.resulting_potency <= self.potency);

                Ok(())
            }
            None => Err(DifferentiationError::PathNotAvailable(
                path_name.to_string(),
            )),
        }
    }

    /// Check if the cell has a specific capability.
    pub fn has_capability(&self, capability: &str) -> bool {
        self.capabilities.iter().any(|c| c == capability)
    }

    /// Get differentiation depth (number of irreversible steps taken).
    pub fn depth(&self) -> usize {
        self.differentiation_history.len()
    }
}

/// Error during differentiation.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum DifferentiationError {
    /// Cell has already reached terminal differentiation
    #[error("cell is already terminally differentiated")]
    AlreadyTerminal,
    /// Requested path is not available from current state
    #[error("differentiation path not available: {0}")]
    PathNotAvailable(String),
}

/// Pre-built differentiation trees for common agent types.
pub mod lineages {
    use super::{Potency, Specialization};

    /// Immune cell lineage: Stem → {T-cell, B-cell, Macrophage}.
    pub fn immune_cells() -> Vec<Specialization> {
        vec![
            Specialization::new(
                "t_cell",
                vec!["detect_threat".into(), "kill_target".into()],
                Potency::Unipotent,
            ),
            Specialization::new(
                "b_cell",
                vec!["produce_antibody".into(), "memory_response".into()],
                Potency::Unipotent,
            ),
            Specialization::terminal(
                "macrophage",
                vec!["phagocytosis".into(), "antigen_presentation".into()],
            ),
        ]
    }

    /// Agent specialization: Generic → {Coder, Reviewer, Researcher}.
    pub fn agent_roles() -> Vec<Specialization> {
        vec![
            Specialization::terminal(
                "coder",
                vec!["write_code".into(), "edit_files".into(), "run_tests".into()],
            ),
            Specialization::terminal(
                "reviewer",
                vec![
                    "read_code".into(),
                    "detect_issues".into(),
                    "suggest_fixes".into(),
                ],
            ),
            Specialization::terminal(
                "researcher",
                vec!["search_web".into(), "read_docs".into(), "summarize".into()],
            ),
        ]
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_potency_ordering() {
        assert!(Potency::Totipotent > Potency::Terminal);
        assert!(Potency::Multipotent > Potency::Unipotent);
    }

    #[test]
    fn test_potency_can_differentiate() {
        assert!(Potency::Totipotent.can_differentiate());
        assert!(Potency::Multipotent.can_differentiate());
        assert!(!Potency::Terminal.can_differentiate());
    }

    #[test]
    fn test_differentiation_lifecycle() {
        let mut cell = DifferentiableCell::multipotent("stem_1", lineages::immune_cells());

        assert!(cell.can_differentiate());
        assert_eq!(cell.potency, Potency::Multipotent);

        // Differentiate into T-cell
        let result = cell.differentiate("t_cell");
        assert!(result.is_ok());
        assert_eq!(cell.specialization, Some("t_cell".to_string()));
        assert!(cell.has_capability("detect_threat"));
        assert!(cell.has_capability("kill_target"));
        assert_eq!(cell.potency, Potency::Unipotent);
    }

    #[test]
    fn test_terminal_differentiation() {
        let mut cell = DifferentiableCell::multipotent("agent_1", lineages::agent_roles());

        let result = cell.differentiate("coder");
        assert!(result.is_ok());
        assert_eq!(cell.potency, Potency::Terminal);
        assert!(!cell.can_differentiate());

        // Cannot differentiate further
        let result = cell.differentiate("reviewer");
        assert_eq!(result, Err(DifferentiationError::AlreadyTerminal));
    }

    #[test]
    fn test_irreversible_history() {
        let mut cell = DifferentiableCell::multipotent("test", lineages::immune_cells());

        cell.differentiate("macrophage").ok();
        assert_eq!(cell.depth(), 1);
        assert_eq!(cell.differentiation_history, vec!["macrophage"]);
    }

    #[test]
    fn test_path_not_available() {
        let mut cell = DifferentiableCell::multipotent("test", lineages::immune_cells());

        let result = cell.differentiate("neuron");
        assert_eq!(
            result,
            Err(DifferentiationError::PathNotAvailable("neuron".to_string()))
        );
    }

    #[test]
    fn test_totipotent_cell() {
        let cell = DifferentiableCell::totipotent("embryo");
        assert_eq!(cell.potency, Potency::Totipotent);
        assert!(cell.specialization.is_none());
        assert!(cell.capabilities.is_empty());
    }

    #[test]
    fn test_immune_cell_lineages() {
        let lineages = lineages::immune_cells();
        assert_eq!(lineages.len(), 3);
    }

    #[test]
    fn test_agent_role_lineages() {
        let roles = lineages::agent_roles();
        assert_eq!(roles.len(), 3);
        assert!(
            roles
                .iter()
                .all(|r| r.resulting_potency == Potency::Terminal)
        );
    }
}
