//! # Regulatory Compliance & ICH Glossary (Phase 9C)
//!
//! ## Compliance Bridge
//!
//! Maps regulatory guidelines (EMA, FDA, WHO) to Conservation Laws.
//!
//! Regulatory requirements can be formalized as constraints on the 11 Conservation Laws.
//! This module provides:
//! 1. Authority-specific guideline definitions
//! 2. Mapping from guidelines to conservation law validations
//! 3. Compliance reporting with violation details
//!
//! ## Supported Authorities
//!
//! - **EMA GVP-IX**: European Medicines Agency Good Pharmacovigilance Practice Module IX
//! - **FDA 21 CFR**: US FDA Code of Federal Regulations Title 21
//! - **WHO-UMC**: World Health Organization Uppsala Monitoring Centre
//! - **ICH E2B**: International Council for Harmonisation E2B(R3) format
//!
//! ## Effectiveness Endpoints
//!
//! FDA clinical trial effectiveness endpoints for proving efficacy.
//! Based on "Multiple Endpoints in Clinical Trials" and "Accelerated Approval" guidance.
//! See [`effectiveness`] module for details.
//!
//! ## ICH Glossary
//!
//! High-performance O(1) lookup for 894+ ICH/CIOMS pharmacovigilance terms.
//! See [`ich_glossary`] module for details.

pub mod effectiveness;
pub mod ich_glossary;
pub mod reportability;

use crate::comppv::types::ConservationLaw;
use crate::comppv::validators::{FullSystemState, SteadyStateState};
use serde::{Deserialize, Serialize};

// ═══════════════════════════════════════════════════════════════════════════
// REGULATORY AUTHORITY
// ═══════════════════════════════════════════════════════════════════════════

/// Regulatory authority enumeration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum RegulatoryAuthority {
    /// European Medicines Agency Good Pharmacovigilance Practice Module IX
    EmaGvpIx,
    /// US FDA Code of Federal Regulations Title 21
    FdaCfr21,
    /// World Health Organization Uppsala Monitoring Centre
    WhoUmc,
    /// International Council for Harmonisation E2B(R3)
    IchE2b,
}

impl RegulatoryAuthority {
    /// Get the human-readable name of the authority.
    #[must_use]
    pub const fn name(&self) -> &'static str {
        match self {
            Self::EmaGvpIx => "EMA GVP Module IX",
            Self::FdaCfr21 => "FDA 21 CFR",
            Self::WhoUmc => "WHO-UMC",
            Self::IchE2b => "ICH E2B(R3)",
        }
    }

    /// Get the authority's focus area.
    #[must_use]
    pub const fn focus(&self) -> &'static str {
        match self {
            Self::EmaGvpIx => "Signal management",
            Self::FdaCfr21 => "Postmarket safety reporting",
            Self::WhoUmc => "International signal detection",
            Self::IchE2b => "Individual case safety reports",
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// GUIDELINE REQUIREMENT
// ═══════════════════════════════════════════════════════════════════════════

/// A specific regulatory requirement with conservation law mapping.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GuidelineRequirement {
    /// Regulatory authority source
    pub authority: RegulatoryAuthority,
    /// Section reference (e.g., "GVP-IX.B.4")
    pub section: String,
    /// Human-readable requirement description
    pub requirement: String,
    /// Conservation laws that validate this requirement
    pub conservation_laws: Vec<ConservationLaw>,
    /// Quantitative threshold if applicable
    pub threshold: Option<f64>,
    /// Whether this requirement is mandatory
    pub mandatory: bool,
}

// ═══════════════════════════════════════════════════════════════════════════
// VIOLATION
// ═══════════════════════════════════════════════════════════════════════════

/// A compliance violation detected during validation.
///
/// Tier: T2-C (∂ + → — boundary violation with causal requirement)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RegulatoryViolation {
    /// The requirement that was violated
    pub requirement: GuidelineRequirement,
    /// The conservation law that failed
    pub failed_law: ConservationLaw,
    /// Actual value observed
    pub actual_value: Option<f64>,
    /// Expected threshold or range
    pub expected: String,
    /// Severity of the violation (1-5, 5 = critical)
    pub severity: u8,
}

/// Backward-compatible alias.
#[deprecated(note = "use RegulatoryViolation — F2 equivocation fix")]
pub type Violation = RegulatoryViolation;

// ═══════════════════════════════════════════════════════════════════════════
// COMPLIANCE REPORT
// ═══════════════════════════════════════════════════════════════════════════

/// Complete compliance assessment report.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ComplianceReport {
    /// Authority assessed against
    pub authority: RegulatoryAuthority,
    /// All requirements checked
    pub requirements_checked: Vec<GuidelineRequirement>,
    /// Violations found
    pub violations: Vec<RegulatoryViolation>,
    /// Overall compliance percentage (0-100)
    pub compliance_percentage: f64,
    /// Whether the system passes all mandatory requirements
    pub passes_mandatory: bool,
}

impl ComplianceReport {
    /// Check if fully compliant (no violations).
    #[must_use]
    pub fn is_compliant(&self) -> bool {
        self.violations.is_empty()
    }

    /// Get count of critical violations (severity 5).
    #[must_use]
    pub fn critical_violations(&self) -> usize {
        self.violations.iter().filter(|v| v.severity == 5).count()
    }

    /// Generate a summary string.
    #[must_use]
    pub fn summary(&self) -> String {
        format!(
            "{}: {:.1}% compliant, {} requirements, {} violations ({} critical)",
            self.authority.name(),
            self.compliance_percentage,
            self.requirements_checked.len(),
            self.violations.len(),
            self.critical_violations()
        )
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// COMPLIANCE BRIDGE
// ═══════════════════════════════════════════════════════════════════════════

/// Bridge between regulatory guidelines and Conservation Law validation.
#[derive(Debug, Default)]
pub struct ComplianceBridge {
    /// Cached guideline requirements by authority
    guidelines: std::collections::HashMap<RegulatoryAuthority, Vec<GuidelineRequirement>>,
}

impl ComplianceBridge {
    /// Create a new compliance bridge with default guidelines.
    #[must_use]
    pub fn new() -> Self {
        let mut bridge = Self {
            guidelines: std::collections::HashMap::new(),
        };

        // Load default guidelines for each authority
        bridge
            .guidelines
            .insert(RegulatoryAuthority::EmaGvpIx, Self::ema_gvp_guidelines());
        bridge
            .guidelines
            .insert(RegulatoryAuthority::FdaCfr21, Self::fda_cfr_guidelines());
        bridge
            .guidelines
            .insert(RegulatoryAuthority::WhoUmc, Self::who_umc_guidelines());
        bridge
            .guidelines
            .insert(RegulatoryAuthority::IchE2b, Self::ich_e2b_guidelines());

        bridge
    }

    /// Load guideline requirements for a specific authority.
    #[must_use]
    pub fn load_guidelines(&self, authority: RegulatoryAuthority) -> Vec<GuidelineRequirement> {
        self.guidelines.get(&authority).cloned().unwrap_or_default()
    }

    /// Validate system state against regulatory requirements.
    #[must_use]
    pub fn validate_compliance(
        &self,
        state: &FullSystemState,
        authority: RegulatoryAuthority,
    ) -> ComplianceReport {
        let requirements = self.load_guidelines(authority);
        let mut violations = Vec::new();

        // Validate each requirement
        for req in &requirements {
            for law in &req.conservation_laws {
                if let Some(violation) = self.check_law(state, &req, *law) {
                    violations.push(violation);
                }
            }
        }

        // Calculate compliance percentage
        let total_checks = requirements
            .iter()
            .map(|r| r.conservation_laws.len())
            .sum::<usize>();
        let passed = total_checks.saturating_sub(violations.len());
        let compliance_percentage = if total_checks > 0 {
            (passed as f64 / total_checks as f64) * 100.0
        } else {
            100.0
        };

        // Check mandatory requirements
        let passes_mandatory = violations
            .iter()
            .filter(|v| v.requirement.mandatory)
            .count()
            == 0;

        ComplianceReport {
            authority,
            requirements_checked: requirements,
            violations,
            compliance_percentage,
            passes_mandatory,
        }
    }

    /// Check a specific conservation law against state.
    fn check_law(
        &self,
        state: &FullSystemState,
        req: &GuidelineRequirement,
        law: ConservationLaw,
    ) -> Option<RegulatoryViolation> {
        // Perform law-specific validation
        match law {
            ConservationLaw::DrugMassBalance => {
                self.validate_law(self.check_mass_balance(state), req, law)
            }
            ConservationLaw::ThermodynamicBinding => {
                self.validate_law(self.check_binding(state), req, law)
            }
            ConservationLaw::SteadyState => {
                self.validate_law(self.check_steady_state(state, req.threshold), req, law)
            }
            // Fail-safe: unimplemented laws generate low-severity warning
            _ => Some(RegulatoryViolation {
                requirement: req.clone(),
                failed_law: law,
                actual_value: None,
                expected: "Validation not implemented".to_string(),
                severity: 1, // Informational - flags unimplemented laws
            }),
        }
    }

    /// Convert check result to optional violation.
    fn validate_law(
        &self,
        result: (bool, Option<f64>),
        req: &GuidelineRequirement,
        law: ConservationLaw,
    ) -> Option<RegulatoryViolation> {
        let (passes, actual) = result;
        if passes {
            None
        } else {
            Some(RegulatoryViolation {
                requirement: req.clone(),
                failed_law: law,
                actual_value: actual,
                expected: req
                    .threshold
                    .map_or("N/A".to_string(), |t| format!("≤ {t}")),
                severity: if req.mandatory { 5 } else { 3 },
            })
        }
    }

    /// Check mass balance conservation.
    fn check_mass_balance(&self, state: &FullSystemState) -> (bool, Option<f64>) {
        if let Some(mb) = &state.mass_balance {
            let error =
                (mb.initial_dose - mb.current_amount_in_body - mb.cumulative_eliminated).abs();
            let tolerance = mb.initial_dose * 0.01; // 1% tolerance
            (error <= tolerance, Some(error))
        } else {
            (true, None) // No data = passes by default
        }
    }

    /// Check thermodynamic binding.
    fn check_binding(&self, state: &FullSystemState) -> (bool, Option<f64>) {
        if let Some(binding) = &state.binding {
            // Ka must be positive for valid binding
            (
                binding.association_constant_m_inv > 0.0,
                Some(binding.association_constant_m_inv),
            )
        } else {
            (true, None)
        }
    }

    /// Check steady-state PK constraints.
    fn check_steady_state(
        &self,
        state: &FullSystemState,
        threshold: Option<f64>,
    ) -> (bool, Option<f64>) {
        if let Some(ss) = &state.steady_state {
            // Calculate expected Css
            if ss.clearance_l_h > 0.0 && ss.dosing_interval_h > 0.0 {
                let expected_css =
                    (ss.bioavailability * ss.dose) / (ss.clearance_l_h * ss.dosing_interval_h);
                let error_pct =
                    ((ss.measured_concentration - expected_css) / expected_css).abs() * 100.0;

                // Check against threshold (default 20% error allowed)
                let max_error = threshold.unwrap_or(20.0);
                (error_pct <= max_error, Some(error_pct))
            } else {
                (false, None)
            }
        } else {
            (true, None)
        }
    }

    /// Check PK parameters against regulatory thresholds.
    #[must_use]
    pub fn check_pk_limits(
        &self,
        pk_state: &SteadyStateState,
        authority: RegulatoryAuthority,
    ) -> bool {
        // Get authority-specific limits
        let (min_bioavail, max_clearance) = match authority {
            RegulatoryAuthority::EmaGvpIx => (0.1, 100.0), // L/h
            RegulatoryAuthority::FdaCfr21 => (0.05, 150.0),
            RegulatoryAuthority::WhoUmc => (0.1, 100.0),
            RegulatoryAuthority::IchE2b => (0.1, 100.0),
        };

        pk_state.bioavailability >= min_bioavail && pk_state.clearance_l_h <= max_clearance
    }

    // ═══════════════════════════════════════════════════════════════════════
    // GUIDELINE DEFINITIONS
    // ═══════════════════════════════════════════════════════════════════════

    fn ema_gvp_guidelines() -> Vec<GuidelineRequirement> {
        vec![
            GuidelineRequirement {
                authority: RegulatoryAuthority::EmaGvpIx,
                section: "GVP-IX.B.2".to_string(),
                requirement: "Signal detection shall use validated statistical methods".to_string(),
                conservation_laws: vec![ConservationLaw::DrugMassBalance],
                threshold: None,
                mandatory: true,
            },
            GuidelineRequirement {
                authority: RegulatoryAuthority::EmaGvpIx,
                section: "GVP-IX.B.4".to_string(),
                requirement: "PK parameters must be within therapeutic range".to_string(),
                conservation_laws: vec![ConservationLaw::SteadyState],
                threshold: Some(20.0), // 20% error tolerance
                mandatory: true,
            },
            GuidelineRequirement {
                authority: RegulatoryAuthority::EmaGvpIx,
                section: "GVP-IX.C.1".to_string(),
                requirement: "Drug-receptor binding must be thermodynamically favorable"
                    .to_string(),
                conservation_laws: vec![ConservationLaw::ThermodynamicBinding],
                threshold: None,
                mandatory: false,
            },
        ]
    }

    fn fda_cfr_guidelines() -> Vec<GuidelineRequirement> {
        vec![
            GuidelineRequirement {
                authority: RegulatoryAuthority::FdaCfr21,
                section: "21 CFR 314.80".to_string(),
                requirement: "Postmarket adverse experience reporting".to_string(),
                conservation_laws: vec![ConservationLaw::DrugMassBalance],
                threshold: None,
                mandatory: true,
            },
            GuidelineRequirement {
                authority: RegulatoryAuthority::FdaCfr21,
                section: "21 CFR 312.32".to_string(),
                requirement: "IND safety reports".to_string(),
                conservation_laws: vec![
                    ConservationLaw::SteadyState,
                    ConservationLaw::ReceptorState,
                ],
                threshold: Some(25.0),
                mandatory: true,
            },
        ]
    }

    fn who_umc_guidelines() -> Vec<GuidelineRequirement> {
        vec![GuidelineRequirement {
            authority: RegulatoryAuthority::WhoUmc,
            section: "WHO-UMC-001".to_string(),
            requirement: "Causality assessment using WHO-UMC criteria".to_string(),
            conservation_laws: vec![
                ConservationLaw::DrugMassBalance,
                ConservationLaw::EnzymeRegeneration,
            ],
            threshold: None,
            mandatory: true,
        }]
    }

    fn ich_e2b_guidelines() -> Vec<GuidelineRequirement> {
        vec![
            GuidelineRequirement {
                authority: RegulatoryAuthority::IchE2b,
                section: "E2B(R3)-A.1".to_string(),
                requirement: "Individual case safety report structure".to_string(),
                conservation_laws: vec![ConservationLaw::DrugMassBalance],
                threshold: None,
                mandatory: true,
            },
            GuidelineRequirement {
                authority: RegulatoryAuthority::IchE2b,
                section: "E2B(R3)-B.4".to_string(),
                requirement: "Drug characterization including PK data".to_string(),
                conservation_laws: vec![
                    ConservationLaw::SteadyState,
                    ConservationLaw::IonizationState,
                ],
                threshold: Some(15.0),
                mandatory: false,
            },
        ]
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::comppv::validators::MassBalanceState;

    #[test]
    fn test_load_guidelines() {
        let bridge = ComplianceBridge::new();
        let ema = bridge.load_guidelines(RegulatoryAuthority::EmaGvpIx);
        assert!(!ema.is_empty());
        assert!(ema.iter().any(|r| r.section.contains("GVP-IX")));
    }

    #[test]
    fn test_compliance_check_pass() {
        let bridge = ComplianceBridge::new();

        // Valid mass balance state
        let state = FullSystemState {
            mass_balance: Some(MassBalanceState {
                initial_dose: 100.0,
                current_amount_in_body: 60.0,
                cumulative_eliminated: 40.0,
            }),
            ..Default::default()
        };

        let report = bridge.validate_compliance(&state, RegulatoryAuthority::EmaGvpIx);
        assert!(report.passes_mandatory);
    }

    #[test]
    fn test_compliance_check_violation() {
        let bridge = ComplianceBridge::new();

        // Invalid mass balance (doesn't sum correctly)
        let state = FullSystemState {
            mass_balance: Some(MassBalanceState {
                initial_dose: 100.0,
                current_amount_in_body: 60.0,
                cumulative_eliminated: 20.0, // Missing 20!
            }),
            ..Default::default()
        };

        let report = bridge.validate_compliance(&state, RegulatoryAuthority::EmaGvpIx);
        assert!(!report.violations.is_empty());
    }

    #[test]
    fn test_pk_limits() {
        let bridge = ComplianceBridge::new();

        let valid_pk = SteadyStateState {
            bioavailability: 0.8,
            dose: 100.0,
            clearance_l_h: 50.0,
            dosing_interval_h: 8.0,
            measured_concentration: 0.25,
        };

        assert!(bridge.check_pk_limits(&valid_pk, RegulatoryAuthority::EmaGvpIx));

        let invalid_pk = SteadyStateState {
            bioavailability: 0.01, // Too low
            dose: 100.0,
            clearance_l_h: 200.0, // Too high
            dosing_interval_h: 8.0,
            measured_concentration: 0.0,
        };

        assert!(!bridge.check_pk_limits(&invalid_pk, RegulatoryAuthority::EmaGvpIx));
    }

    #[test]
    fn test_report_summary() {
        let report = ComplianceReport {
            authority: RegulatoryAuthority::EmaGvpIx,
            requirements_checked: vec![],
            violations: vec![],
            compliance_percentage: 100.0,
            passes_mandatory: true,
        };

        assert!(report.is_compliant());
        assert!(report.summary().contains("100.0%"));
    }

    #[test]
    fn test_unimplemented_law_generates_warning() {
        // Gemini-identified test: verify unimplemented laws generate warnings
        let bridge = ComplianceBridge::new();

        // WHO-UMC guidelines include EnzymeRegeneration law which is not yet implemented
        let state = FullSystemState::default();
        let report = bridge.validate_compliance(&state, RegulatoryAuthority::WhoUmc);

        // Should have at least one low-severity violation for unimplemented law
        let unimplemented_warnings: Vec<_> = report
            .violations
            .iter()
            .filter(|v| v.severity == 1 && v.expected.contains("not implemented"))
            .collect();

        assert!(
            !unimplemented_warnings.is_empty(),
            "Unimplemented laws should generate severity-1 warnings, not silently pass"
        );
    }
}
