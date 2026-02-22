// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Antipattern Immunity System
//!
//! Self-healing code through learned antibodies. Detects antipatterns (PAMPs/DAMPs),
//! applies corrections, and prevents recurrence.
//!
//! ## Primitive Grounding
//!
//! The immunity system is grounded in T1 primitives:
//!
//! | Concept | T1 Primitive | Symbol |
//! |---------|--------------|--------|
//! | Threat Sensing | Existence | ∃ |
//! | Pattern Validation | Comparison | κ |
//! | Fix Transformation | Mapping | μ |
//! | Antibody Storage | Persistence | π |
//! | Homeostasis Loop | Recursion | ρ |
//! | Detection Rate | Frequency | ν |
//!
//! ## Homeostasis Loop
//!
//! ```text
//! ┌─────────────────────────────────────────────────────────────┐
//! │                   HOMEOSTASIS LOOP                          │
//! │                                                             │
//! │   SENSE ──► DECIDE ──► RESPOND ──► LEARN ──► SENSE...     │
//! │     │         │          │          │                       │
//! │   [PAMP]   [Match]    [Block/     [Store                   │
//! │   [DAMP]   [Pattern]   Fix]       Antibody]                │
//! └─────────────────────────────────────────────────────────────┘
//! ```
//!
//! ## Quick Start
//!
//! ```rust,no_run
//! use nexcore_immunity::{load_default_registry, ImmunityScanner};
//!
//! // Load antibodies from ~/.claude/immunity/antibodies.yaml
//! let registry = load_default_registry()?;
//!
//! // Create scanner
//! let scanner = ImmunityScanner::new(&registry)?;
//!
//! // Scan code for threats
//! let result = scanner.scan("let x = foo.unwrap();", Some("test.rs"));
//!
//! if !result.clean {
//!     for threat in &result.threats {
//!         println!("[{}] {} at line {:?}",
//!             threat.severity,
//!             threat.antibody_name,
//!             threat.location
//!         );
//!     }
//! }
//! # Ok::<(), nexcore_immunity::ImmunityError>(())
//! ```
//!
//! ## Threat Classification
//!
//! - **PAMPs** (Pathogen-Associated Molecular Patterns): External threats from user input,
//!   templates, or dependencies.
//! - **DAMPs** (Damage-Associated Molecular Patterns): Internal damage signals like
//!   compilation errors, test failures, or structural issues.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic, missing_docs)]

pub mod adaptive;
pub mod co_translational;
pub mod error;
pub mod grounding;
pub mod loader;
pub mod scanner;
pub mod smg;
pub mod thymic;
pub mod types;

// Re-exports for convenience
pub use error::{ImmunityError, ImmunityResult};
pub use loader::{DEFAULT_REGISTRY_PATH, load_default_registry, load_from_str, load_registry};
pub use scanner::ImmunityScanner;
pub use co_translational::{
    CheckpointObservation, UpfAnomaly, UpfChannel, UpfComplex, UpfConfig, UpfVerdict,
};
// Re-export spliceosome types used in NMD pipeline
pub use nexcore_spliceosome::{EjcMarker, TaskCategory};
pub use adaptive::{NmdAdaptiveEngine, NmdLearningEvent, ThresholdAdjustment};
pub use smg::{SmgAction, SmgComplex, SmgConfig};
pub use thymic::{CategoryObservation, ThymicConfig, ThymicGate};
pub use types::{
    Antibody, AntibodyRegistry, AutoimmuneDiagnosis, AutoimmuneReport, AutoimmuneStatus,
    CodePattern, Detection, Response, ResponseStrategy, ScanMetrics, ScanResult, ThreatLevel,
    ThreatMatch, ThreatType,
};

/// Prelude for common imports.
pub mod prelude {
    //! Common imports for immunity system usage.
    pub use crate::error::{ImmunityError, ImmunityResult};
    pub use crate::loader::{load_default_registry, load_from_str, load_registry};
    pub use crate::scanner::ImmunityScanner;
    pub use crate::types::{
        Antibody, AntibodyRegistry, ResponseStrategy, ScanResult, ThreatLevel, ThreatMatch,
        ThreatType,
    };
}

// ═══════════════════════════════════════════════════════════════════════════════
// INTEGRATION TESTS
// ═══════════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod integration_tests {
    use super::*;

    #[test]
    fn test_full_workflow() {
        // Create a minimal registry
        let yaml = r#"
version: "1.0"
antibodies:
  - id: TEST-001
    name: test-unwrap
    threat_type: DAMP
    severity: critical
    description: Detects unwrap
    detection:
      code_patterns:
        - pattern: '\.unwrap\(\)'
    response:
      strategy: suggest_safe_alternative
    confidence: 0.9
"#;

        let registry = load_from_str(yaml);
        assert!(registry.is_ok());
        let registry = registry.ok().unwrap_or_default();

        let scanner = ImmunityScanner::new(&registry);
        assert!(scanner.is_ok());
        let scanner = scanner.ok().unwrap_or_else(|| {
            ImmunityScanner::new(&AntibodyRegistry::new())
                .ok()
                .unwrap_or_else(|| panic!("Failed to create scanner"))
        });

        // Test clean code
        let result = scanner.scan("let x = foo?;", None);
        assert!(result.clean);

        // Test infected code
        let result = scanner.scan("let x = foo.unwrap();", None);
        assert!(!result.clean);
        assert_eq!(result.threats.len(), 1);
        assert_eq!(result.threats[0].severity, ThreatLevel::Critical);
    }

    #[test]
    fn test_threat_types() {
        assert_eq!(ThreatType::Pamp.to_string(), "PAMP");
        assert_eq!(ThreatType::Damp.to_string(), "DAMP");
    }

    #[test]
    fn test_severity_ordering() {
        assert!(ThreatLevel::Low < ThreatLevel::Medium);
        assert!(ThreatLevel::Medium < ThreatLevel::High);
        assert!(ThreatLevel::High < ThreatLevel::Critical);
    }

    #[test]
    fn test_empty_registry() {
        let registry = AntibodyRegistry::new();
        let scanner = ImmunityScanner::new(&registry);
        assert!(scanner.is_ok());

        let scanner = scanner
            .ok()
            .unwrap_or_else(|| panic!("Failed to create scanner"));
        let result = scanner.scan("let x = foo.unwrap();", None);
        assert!(result.clean); // No antibodies = no threats detected
    }
}
