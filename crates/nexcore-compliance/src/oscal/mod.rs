//! OSCAL-Inspired Compliance Types
//!
//! Type-safe control definitions based on NIST OSCAL patterns.
//! Adapted for pharmacovigilance regulatory compliance.

use serde::{Deserialize, Serialize};

/// A regulatory control from a catalog.
///
/// T2-P: Cross-domain primitive for compliance controls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Control {
    /// Unique identifier (e.g., "ICH-E2A-3.1").
    pub id: String,
    /// Human-readable title.
    pub title: String,
    /// Control description.
    pub description: String,
    /// Source catalog (FDA, ICH, CIOMS).
    pub catalog: String,
    /// Current implementation status.
    pub status: ControlStatus,
}

/// Implementation status of a control.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ControlStatus {
    /// Not yet implemented.
    NotImplemented,
    /// Partially implemented.
    Partial,
    /// Fully implemented.
    Implemented,
    /// Not applicable to this system.
    NotApplicable,
}

/// A catalog of regulatory controls.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ControlCatalog {
    /// Catalog name.
    pub name: String,
    /// Controls in this catalog.
    pub controls: Vec<Control>,
}

impl ControlCatalog {
    /// Create a new empty catalog.
    #[must_use]
    pub fn new(name: impl Into<String>) -> Self {
        Self {
            name: name.into(),
            controls: Vec::new(),
        }
    }

    /// Add a control to the catalog.
    pub fn add_control(&mut self, control: Control) {
        self.controls.push(control);
    }

    /// Get compliance percentage (implemented / total).
    #[must_use]
    pub fn compliance_percentage(&self) -> f64 {
        if self.controls.is_empty() {
            return 100.0;
        }
        let implemented = self
            .controls
            .iter()
            .filter(|c| {
                matches!(
                    c.status,
                    ControlStatus::Implemented | ControlStatus::NotApplicable
                )
            })
            .count();
        (implemented as f64 / self.controls.len() as f64) * 100.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_control(id: &str, status: ControlStatus) -> Control {
        Control {
            id: id.to_string(),
            title: format!("Control {id}"),
            description: format!("Description for {id}"),
            catalog: "ICH".to_string(),
            status,
        }
    }

    #[test]
    fn empty_catalog_is_100_percent() {
        let catalog = ControlCatalog::new("empty");
        assert!((catalog.compliance_percentage() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn all_implemented_is_100_percent() {
        let mut catalog = ControlCatalog::new("ICH");
        catalog.add_control(make_control("ICH-01", ControlStatus::Implemented));
        catalog.add_control(make_control("ICH-02", ControlStatus::Implemented));
        assert!((catalog.compliance_percentage() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn not_applicable_counts_as_compliant() {
        let mut catalog = ControlCatalog::new("ICH");
        catalog.add_control(make_control("ICH-01", ControlStatus::Implemented));
        catalog.add_control(make_control("ICH-02", ControlStatus::NotApplicable));
        assert!((catalog.compliance_percentage() - 100.0).abs() < f64::EPSILON);
    }

    #[test]
    fn partial_compliance_calculates_correctly() {
        let mut catalog = ControlCatalog::new("FDA");
        catalog.add_control(make_control("FDA-01", ControlStatus::Implemented));
        catalog.add_control(make_control("FDA-02", ControlStatus::NotImplemented));
        assert!((catalog.compliance_percentage() - 50.0).abs() < f64::EPSILON);
    }

    #[test]
    fn zero_compliance_when_none_implemented() {
        let mut catalog = ControlCatalog::new("CIOMS");
        catalog.add_control(make_control("C-01", ControlStatus::NotImplemented));
        catalog.add_control(make_control("C-02", ControlStatus::Partial));
        assert!((catalog.compliance_percentage() - 0.0).abs() < f64::EPSILON);
    }

    #[test]
    fn catalog_name_preserved() {
        let catalog = ControlCatalog::new("Test Catalog");
        assert_eq!(catalog.name, "Test Catalog");
    }

    #[test]
    fn control_serialization_roundtrip() {
        let control = make_control("ICH-E2A-3.1", ControlStatus::Implemented);
        let json = serde_json::to_string(&control).unwrap_or_default();
        assert!(!json.is_empty());
        let deserialized: Result<Control, _> = serde_json::from_str(&json);
        assert!(deserialized.is_ok());
    }
}
