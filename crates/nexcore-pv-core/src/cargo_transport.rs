//! # Cargo Transport — ICSR and Signal Types as Typed Cargo
//!
//! Implements the `nexcore_cargo::Cargo` trait for PV domain types,
//! making them first-class participants in the typed transport system.
//!
//! ## Seriousness → Perishability Mapping (ICH E2D)
//!
//! | Seriousness | Perishability | Regulatory Basis |
//! |-------------|--------------|------------------|
//! | Death | Expedited(7) | IND safety report (FDA 21 CFR 312.32) |
//! | Life-threatening | Expedited(15) | ICH E2D §III.A — 15 calendar days |
//! | Hospitalization/Disability | Prompt(90) | ICH E2D §III.B — 90 calendar days |
//! | Other medically important | Prompt(90) | ICH E2D §III.B |
//! | Non-serious | Periodic | PSUR/PBRER reporting cycle |
//!
//! ## Cold-Chain Principle
//!
//! Perishability can UPGRADE during transit but never downgrade. A case
//! starts as Periodic when first ingested from FAERS. Signal detection
//! finding a strong PRR upgrades to Prompt(90). Causality assessment
//! finding "Certain" + death upgrades to Expedited(7). The urgency is
//! DISCOVERED during transit, not known at loading.

use nexcore_cargo::{
    Cargo, CustodyChain, DataSource, Destination, Perishability, Provenance, QueryParams,
    StationStamp,
};

use crate::icsr::{
    CausalityResult, Icsr, ReportSource, ReportType, Seriousness, SeriousnessCriterion,
};

// ═══════════════════════════════════════════════════════════════════════════
// ICSR CARGO — The atomic PV unit in transit
// ═══════════════════════════════════════════════════════════════════════════

/// An ICSR wrapped as typed cargo for transport through the PV pipeline.
///
/// Adds provenance tracking, destination routing, perishability management,
/// and chain of custody to the raw ICSR. The ICSR itself is the payload;
/// the cargo wrapper is the shipping label.
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct IcsrCargo {
    /// The ICSR payload
    icsr: Icsr,
    /// Where this ICSR originated
    provenance: Provenance,
    /// Current destination (may change during transit)
    destination: Destination,
    /// Regulatory reporting deadline
    perishability: Perishability,
    /// Chain of custody through processing stations
    custody: CustodyChain,
}

impl IcsrCargo {
    /// Create ICSR cargo from a raw ICSR.
    ///
    /// Automatically derives:
    /// - `provenance` from report source and type
    /// - `destination` from seriousness criteria
    /// - `perishability` from seriousness → ICH E2D deadline mapping
    #[must_use]
    pub fn from_icsr(icsr: Icsr, loaded_at: i64, source_confidence: f64) -> Self {
        let provenance = derive_provenance(&icsr, loaded_at, source_confidence);
        let destination = derive_destination(&icsr.seriousness);
        let perishability = derive_perishability(&icsr.seriousness);

        Self {
            icsr,
            provenance,
            destination,
            perishability,
            custody: CustodyChain::new(),
        }
    }

    /// Create ICSR cargo with explicit provenance (e.g., from a specific FAERS query).
    #[must_use]
    pub fn with_provenance(icsr: Icsr, provenance: Provenance) -> Self {
        let destination = derive_destination(&icsr.seriousness);
        let perishability = derive_perishability(&icsr.seriousness);

        Self {
            icsr,
            provenance,
            destination,
            perishability,
            custody: CustodyChain::new(),
        }
    }

    /// Access the raw ICSR.
    #[must_use]
    pub fn icsr(&self) -> &Icsr {
        &self.icsr
    }

    /// The highest causality result across all assessments.
    #[must_use]
    pub fn highest_causality(&self) -> Option<CausalityResult> {
        self.icsr.causality.iter().map(|c| c.result).max()
    }

    /// Upgrade perishability based on causality assessment findings.
    ///
    /// Called when a causality assessment station processes this cargo.
    /// WHO-UMC "Certain" + death → Expedited(7).
    /// Naranjo "Probable" or higher → at least Prompt(90).
    pub fn apply_causality_upgrade(&mut self) {
        if let Some(highest) = self.highest_causality() {
            let new_perishability = match highest {
                CausalityResult::Certain if self.icsr.seriousness.death => {
                    Perishability::Expedited { deadline_days: 7 }
                }
                CausalityResult::Certain => Perishability::EXPEDITED_15,
                CausalityResult::Probable => Perishability::PROMPT_90,
                _ => return, // No upgrade for Possible/Unlikely/Unassessable
            };
            self.perishability = self.perishability.upgrade(new_perishability);
        }
    }
}

impl Cargo for IcsrCargo {
    type Payload = Icsr;

    fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    fn destination(&self) -> Destination {
        self.destination
    }

    fn perishability(&self) -> Perishability {
        self.perishability
    }

    fn custody_chain(&self) -> &CustodyChain {
        &self.custody
    }

    fn payload(&self) -> &Icsr {
        &self.icsr
    }

    fn stamp(&mut self, stamp: StationStamp) {
        self.custody.stamp(stamp);
    }

    fn upgrade_perishability(&mut self, new: Perishability) {
        self.perishability = self.perishability.upgrade(new);
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// DERIVATION FUNCTIONS
// ═══════════════════════════════════════════════════════════════════════════

/// Derive provenance from ICSR report metadata.
fn derive_provenance(icsr: &Icsr, loaded_at: i64, source_confidence: f64) -> Provenance {
    let source = match icsr.report.report_type {
        ReportType::Spontaneous => match icsr.report.source {
            ReportSource::HealthcareProfessional | ReportSource::Consumer => DataSource::Faers,
            _ => DataSource::Faers,
        },
        ReportType::StudyReport => DataSource::ClinicalTrials,
        ReportType::Literature => DataSource::Literature,
        ReportType::Other => DataSource::Other("unknown".to_string()),
    };

    let mut query = QueryParams::empty();
    query.insert("case_id", icsr.case_id.as_str());
    if let Some(ref country) = icsr.report.country {
        query.insert("country", country);
    }
    if let Some(drug) = icsr.drugs.first() {
        query.insert("drug", &drug.name);
    }

    Provenance::new(source, query, loaded_at, source_confidence)
}

/// Derive destination from seriousness criteria.
///
/// The destination determines where this cargo is heading:
/// - Serious cases with death/life-threatening → RegulatoryReporting (expedited)
/// - Serious cases → CausalityAssessment (needs evaluation)
/// - Non-serious → AggregateAnalysis (periodic review)
fn derive_destination(seriousness: &Seriousness) -> Destination {
    match seriousness.most_severe() {
        Some(SeriousnessCriterion::Death | SeriousnessCriterion::LifeThreatening) => {
            Destination::RegulatoryReporting
        }
        Some(_) => Destination::CausalityAssessment,
        None => Destination::AggregateAnalysis,
    }
}

/// Derive perishability from seriousness criteria per ICH E2D.
///
/// This is the cold-chain clock. Once set, it can only upgrade (get more urgent).
fn derive_perishability(seriousness: &Seriousness) -> Perishability {
    match seriousness.most_severe() {
        Some(SeriousnessCriterion::Death) => {
            // IND safety report: 7 calendar days (FDA 21 CFR 312.32)
            Perishability::Expedited { deadline_days: 7 }
        }
        Some(SeriousnessCriterion::LifeThreatening) => {
            // ICH E2D §III.A: 15 calendar days
            Perishability::EXPEDITED_15
        }
        Some(
            SeriousnessCriterion::Hospitalization
            | SeriousnessCriterion::Disability
            | SeriousnessCriterion::CongenitalAnomaly
            | SeriousnessCriterion::OtherMedicallyImportant,
        ) => {
            // ICH E2D §III.B: 90 calendar days
            Perishability::PROMPT_90
        }
        None => {
            // Non-serious: periodic reporting (PSUR/PBRER cycle)
            Perishability::Periodic
        }
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// SIGNAL CARGO — Disproportionality results in transit
// ═══════════════════════════════════════════════════════════════════════════

use crate::types::CompleteSignalResult;

/// A complete signal detection result wrapped as typed cargo.
///
/// When a drug-event pair undergoes disproportionality analysis, the result
/// becomes cargo that transits toward causality assessment or regulatory
/// reporting. The `is_signal` determination drives perishability:
///
/// - **Any algorithm detects signal** → `Prompt(90)` (needs causality review)
/// - **Multiple algorithms agree** → `Expedited(15)` (convergent evidence)
/// - **No signal** → `Periodic` (aggregate surveillance)
#[derive(Debug, Clone, serde::Serialize, serde::Deserialize)]
pub struct SignalCargo {
    /// The signal detection payload
    result: CompleteSignalResult,
    /// Drug name for provenance tracking
    drug: String,
    /// Event/reaction name for provenance tracking
    event: String,
    /// Where this result originated
    provenance: Provenance,
    /// Current destination
    destination: Destination,
    /// Regulatory urgency
    perishability: Perishability,
    /// Chain of custody
    custody: CustodyChain,
}

impl SignalCargo {
    /// Create signal cargo from a complete signal result.
    ///
    /// Automatically derives perishability from signal detection outcomes:
    /// - 2+ algorithms agree on signal → Expedited(15)
    /// - 1 algorithm detects signal → Prompt(90)
    /// - No signal detected → Periodic
    #[must_use]
    pub fn from_result(
        result: CompleteSignalResult,
        drug: String,
        event: String,
        source: DataSource,
        loaded_at: i64,
        source_confidence: f64,
    ) -> Self {
        let signal_count = count_signals(&result);
        let perishability = signal_count_to_perishability(signal_count);
        let destination = if signal_count >= 2 {
            Destination::RegulatoryReporting
        } else if signal_count == 1 {
            Destination::CausalityAssessment
        } else {
            Destination::AggregateAnalysis
        };

        let mut query = QueryParams::empty();
        query.insert("drug", &drug);
        query.insert("event", &event);
        query.insert("n", &result.n.to_string());

        let provenance = Provenance::new(source, query, loaded_at, source_confidence);

        Self {
            result,
            drug,
            event,
            provenance,
            destination,
            perishability,
            custody: CustodyChain::new(),
        }
    }

    /// Access the signal detection result.
    #[must_use]
    pub fn result(&self) -> &CompleteSignalResult {
        &self.result
    }

    /// Drug name.
    #[must_use]
    pub fn drug(&self) -> &str {
        &self.drug
    }

    /// Event name.
    #[must_use]
    pub fn event(&self) -> &str {
        &self.event
    }

    /// Count how many of the 4 algorithms detected a signal.
    #[must_use]
    pub fn signal_count(&self) -> usize {
        count_signals(&self.result)
    }

    /// Whether any algorithm detected a signal.
    #[must_use]
    pub fn is_signal(&self) -> bool {
        count_signals(&self.result) > 0
    }
}

impl Cargo for SignalCargo {
    type Payload = CompleteSignalResult;

    fn provenance(&self) -> &Provenance {
        &self.provenance
    }

    fn destination(&self) -> Destination {
        self.destination
    }

    fn perishability(&self) -> Perishability {
        self.perishability
    }

    fn custody_chain(&self) -> &CustodyChain {
        &self.custody
    }

    fn payload(&self) -> &CompleteSignalResult {
        &self.result
    }

    fn stamp(&mut self, stamp: StationStamp) {
        self.custody.stamp(stamp);
    }

    fn upgrade_perishability(&mut self, new: Perishability) {
        self.perishability = self.perishability.upgrade(new);
    }
}

/// Count algorithms that detected a signal (out of 4: PRR, ROR, IC, EBGM).
fn count_signals(result: &CompleteSignalResult) -> usize {
    [
        result.prr.is_signal,
        result.ror.is_signal,
        result.ic.is_signal,
        result.ebgm.is_signal,
    ]
    .iter()
    .filter(|&&s| s)
    .count()
}

/// Map signal algorithm agreement count to perishability.
///
/// Convergent evidence from multiple algorithms increases urgency:
/// - 2+ algorithms → Expedited(15): strong convergent signal
/// - 1 algorithm → Prompt(90): needs further investigation
/// - 0 algorithms → Periodic: routine surveillance
fn signal_count_to_perishability(count: usize) -> Perishability {
    match count {
        0 => Perishability::Periodic,
        1 => Perishability::PROMPT_90,
        _ => Perishability::EXPEDITED_15,
    }
}

// ═══════════════════════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;
    use crate::icsr::{
        Assessor, CausalityAssessment, CausalityMethod, IcsrBuilder, ReactionOutcome,
    };

    fn fatal_icsr() -> Icsr {
        IcsrBuilder::new("FATAL-001")
            .suspect_drug("ROFECOXIB")
            .reaction("Myocardial infarction", ReactionOutcome::Fatal)
            .seriousness(Seriousness {
                death: true,
                ..Default::default()
            })
            .build()
    }

    fn serious_icsr() -> Icsr {
        IcsrBuilder::new("SERIOUS-001")
            .suspect_drug("METFORMIN")
            .reaction("Lactic acidosis", ReactionOutcome::Recovered)
            .seriousness(Seriousness {
                hospitalization: true,
                ..Default::default()
            })
            .build()
    }

    fn nonserious_icsr() -> Icsr {
        IcsrBuilder::new("NONSER-001")
            .suspect_drug("ASPIRIN")
            .reaction("Headache", ReactionOutcome::Recovered)
            .build()
    }

    #[test]
    fn fatal_case_gets_expedited_7() {
        let cargo = IcsrCargo::from_icsr(fatal_icsr(), 1709856000, 0.98);

        assert_eq!(
            cargo.perishability(),
            Perishability::Expedited { deadline_days: 7 }
        );
        assert_eq!(cargo.destination(), Destination::RegulatoryReporting);
    }

    #[test]
    fn serious_case_gets_prompt_90() {
        let cargo = IcsrCargo::from_icsr(serious_icsr(), 1709856000, 0.95);

        assert_eq!(cargo.perishability(), Perishability::PROMPT_90);
        assert_eq!(cargo.destination(), Destination::CausalityAssessment);
    }

    #[test]
    fn nonserious_case_gets_periodic() {
        let cargo = IcsrCargo::from_icsr(nonserious_icsr(), 1709856000, 0.90);

        assert_eq!(cargo.perishability(), Perishability::Periodic);
        assert_eq!(cargo.destination(), Destination::AggregateAnalysis);
    }

    #[test]
    fn provenance_captures_case_metadata() {
        let icsr = IcsrBuilder::new("US-FDA-2024-12345")
            .suspect_drug("METFORMIN")
            .build();
        let cargo = IcsrCargo::from_icsr(icsr, 1709856000, 0.98);

        assert_eq!(cargo.provenance().source, DataSource::Faers);
        assert_eq!(
            cargo
                .provenance()
                .query
                .params
                .get("case_id")
                .map(String::as_str),
            Some("US-FDA-2024-12345")
        );
        assert_eq!(
            cargo
                .provenance()
                .query
                .params
                .get("drug")
                .map(String::as_str),
            Some("METFORMIN")
        );
    }

    #[test]
    fn custody_chain_grows_during_transit() {
        let mut cargo = IcsrCargo::from_icsr(nonserious_icsr(), 1709856000, 0.90);

        cargo.stamp(StationStamp::new("openfda", "ingest", 1709856100, 0.98));
        cargo.stamp(StationStamp::new("signal-detect", "prr", 1709856200, 0.93));

        assert_eq!(cargo.custody_chain().hop_count(), 2);
        let expected_f = 0.98 * 0.93;
        assert!((cargo.custody_chain().cumulative_fidelity() - expected_f).abs() < 1e-10);
    }

    #[test]
    fn cold_chain_upgrade_never_downgrades() {
        let mut cargo = IcsrCargo::from_icsr(nonserious_icsr(), 1709856000, 0.90);
        assert_eq!(cargo.perishability(), Perishability::Periodic);

        // Signal detected — upgrade to Prompt
        cargo.upgrade_perishability(Perishability::PROMPT_90);
        assert_eq!(cargo.perishability(), Perishability::PROMPT_90);

        // Try to downgrade — should be rejected
        cargo.upgrade_perishability(Perishability::Periodic);
        assert_eq!(cargo.perishability(), Perishability::PROMPT_90);

        // Further upgrade to Expedited — should succeed
        cargo.upgrade_perishability(Perishability::EXPEDITED_15);
        assert_eq!(cargo.perishability(), Perishability::EXPEDITED_15);
    }

    #[test]
    fn causality_upgrade_certain_death() {
        let icsr = IcsrBuilder::new("CAUSAL-001")
            .suspect_drug("DRUG_X")
            .reaction("Hepatic failure", ReactionOutcome::Fatal)
            .seriousness(Seriousness {
                death: true,
                ..Default::default()
            })
            .causality(CausalityAssessment {
                drug_index: 0,
                reaction_index: 0,
                method: CausalityMethod::WhoUmc,
                result: CausalityResult::Certain,
                assessor: Assessor::Algorithm,
            })
            .build();

        let mut cargo = IcsrCargo::from_icsr(icsr, 1709856000, 0.95);
        // Already Expedited(7) from seriousness, but apply_causality_upgrade
        // confirms it stays at 7 (Certain + death)
        cargo.apply_causality_upgrade();
        assert_eq!(
            cargo.perishability(),
            Perishability::Expedited { deadline_days: 7 }
        );
    }

    #[test]
    fn causality_upgrade_probable_nonserious() {
        let icsr = IcsrBuilder::new("CAUSAL-002")
            .suspect_drug("DRUG_Y")
            .reaction("Rash", ReactionOutcome::Recovered)
            .causality(CausalityAssessment {
                drug_index: 0,
                reaction_index: 0,
                method: CausalityMethod::Naranjo,
                result: CausalityResult::Probable,
                assessor: Assessor::Algorithm,
            })
            .build();

        let mut cargo = IcsrCargo::from_icsr(icsr, 1709856000, 0.90);
        assert_eq!(cargo.perishability(), Perishability::Periodic);

        // Causality assessment finds Probable — upgrade to Prompt(90)
        cargo.apply_causality_upgrade();
        assert_eq!(cargo.perishability(), Perishability::PROMPT_90);
    }

    #[test]
    fn full_transit_scenario() {
        // Simulate: FAERS ingest → signal detection → causality → routing
        let icsr = IcsrBuilder::new("TRANSIT-001")
            .suspect_drug("METFORMIN")
            .reaction("Lactic acidosis", ReactionOutcome::Recovered)
            .seriousness(Seriousness {
                hospitalization: true,
                ..Default::default()
            })
            .build();

        let mut cargo = IcsrCargo::from_icsr(icsr, 1709856000, 0.98);

        // Station 1: openFDA ingest
        cargo.stamp(StationStamp::new(
            "nexvigilant-station::openfda",
            "search_adverse_events",
            1709856100,
            0.98,
        ));

        // Station 2: Signal detection — finds strong signal
        cargo.stamp(StationStamp::new(
            "signal-pipeline::detect",
            "prr_compute",
            1709856200,
            0.93,
        ));
        // Signal confirmed — already Prompt(90) from seriousness, stays

        // Station 3: Causality assessment
        cargo.stamp(StationStamp::new(
            "microgram::naranjo-quick",
            "naranjo_score",
            1709856300,
            0.95,
        ));

        // Verify chain
        assert_eq!(cargo.custody_chain().hop_count(), 3);
        assert_eq!(cargo.perishability(), Perishability::PROMPT_90);
        assert_eq!(cargo.destination(), Destination::CausalityAssessment);
        assert!(cargo.custody_chain().meets_safety_threshold());

        // Verify fidelity: 0.98 * 0.93 * 0.95 ≈ 0.866
        let f_total = cargo.custody_chain().cumulative_fidelity();
        let expected = 0.98 * 0.93 * 0.95;
        assert!((f_total - expected).abs() < 1e-10);
    }

    #[test]
    fn literature_report_provenance() {
        use crate::icsr::{ReportInfo, ReportType};

        let icsr = IcsrBuilder::new("LIT-001")
            .suspect_drug("THALIDOMIDE")
            .reaction("Phocomelia", ReactionOutcome::NotRecovered)
            .report(ReportInfo {
                report_type: ReportType::Literature,
                source: ReportSource::HealthcareProfessional,
                country: Some("DE".to_string()),
                receipt_date: None,
                latest_date: None,
            })
            .seriousness(Seriousness {
                congenital_anomaly: true,
                ..Default::default()
            })
            .build();

        let cargo = IcsrCargo::from_icsr(icsr, 1709856000, 0.85);

        assert_eq!(cargo.provenance().source, DataSource::Literature);
        assert_eq!(cargo.perishability(), Perishability::PROMPT_90);
        assert_eq!(
            cargo
                .provenance()
                .query
                .params
                .get("country")
                .map(String::as_str),
            Some("DE")
        );
    }

    // ── Signal Cargo Tests ───────────────────────────────────────────

    use crate::types::SignalResult;

    fn no_signal_result() -> CompleteSignalResult {
        CompleteSignalResult {
            prr: SignalResult::new(1.2, 0.8, 1.5, false),
            ror: SignalResult::new(1.1, 0.7, 1.4, false),
            ic: SignalResult::new(-0.2, -0.5, 0.1, false),
            ebgm: SignalResult::new(1.0, 0.8, 1.3, false),
            chi_square: 1.2,
            n: 15,
        }
    }

    fn single_signal_result() -> CompleteSignalResult {
        CompleteSignalResult {
            prr: SignalResult::new(3.5, 2.1, 5.8, true), // PRR signals
            ror: SignalResult::new(1.3, 0.9, 1.8, false),
            ic: SignalResult::new(0.1, -0.2, 0.4, false),
            ebgm: SignalResult::new(1.1, 0.9, 1.4, false),
            chi_square: 8.5,
            n: 42,
        }
    }

    fn convergent_signal_result() -> CompleteSignalResult {
        CompleteSignalResult {
            prr: SignalResult::new(4.2, 2.8, 6.3, true),
            ror: SignalResult::new(3.8, 2.5, 5.7, true),
            ic: SignalResult::new(1.5, 0.8, 2.2, true),
            ebgm: SignalResult::new(2.1, 1.5, 2.9, false),
            chi_square: 24.7,
            n: 156,
        }
    }

    #[test]
    fn no_signal_gets_periodic() {
        let cargo = SignalCargo::from_result(
            no_signal_result(),
            "ASPIRIN".into(),
            "Headache".into(),
            DataSource::Faers,
            1709856000,
            0.95,
        );

        assert_eq!(cargo.perishability(), Perishability::Periodic);
        assert_eq!(cargo.destination(), Destination::AggregateAnalysis);
        assert!(!cargo.is_signal());
        assert_eq!(cargo.signal_count(), 0);
    }

    #[test]
    fn single_signal_gets_prompt() {
        let cargo = SignalCargo::from_result(
            single_signal_result(),
            "ROFECOXIB".into(),
            "Myocardial infarction".into(),
            DataSource::Faers,
            1709856000,
            0.98,
        );

        assert_eq!(cargo.perishability(), Perishability::PROMPT_90);
        assert_eq!(cargo.destination(), Destination::CausalityAssessment);
        assert!(cargo.is_signal());
        assert_eq!(cargo.signal_count(), 1);
    }

    #[test]
    fn convergent_signal_gets_expedited() {
        let cargo = SignalCargo::from_result(
            convergent_signal_result(),
            "METFORMIN".into(),
            "Lactic acidosis".into(),
            DataSource::Faers,
            1709856000,
            0.98,
        );

        assert_eq!(cargo.perishability(), Perishability::EXPEDITED_15);
        assert_eq!(cargo.destination(), Destination::RegulatoryReporting);
        assert!(cargo.is_signal());
        assert_eq!(cargo.signal_count(), 3);
    }

    #[test]
    fn signal_cargo_provenance_tracks_drug_event() {
        let cargo = SignalCargo::from_result(
            single_signal_result(),
            "METFORMIN".into(),
            "Lactic acidosis".into(),
            DataSource::Faers,
            1709856000,
            0.98,
        );

        assert_eq!(cargo.drug(), "METFORMIN");
        assert_eq!(cargo.event(), "Lactic acidosis");
        assert_eq!(cargo.provenance().source, DataSource::Faers);
        assert_eq!(
            cargo
                .provenance()
                .query
                .params
                .get("drug")
                .map(String::as_str),
            Some("METFORMIN")
        );
        assert_eq!(
            cargo
                .provenance()
                .query
                .params
                .get("event")
                .map(String::as_str),
            Some("Lactic acidosis")
        );
    }

    #[test]
    fn signal_cargo_custody_chain() {
        let mut cargo = SignalCargo::from_result(
            convergent_signal_result(),
            "METFORMIN".into(),
            "Lactic acidosis".into(),
            DataSource::Faers,
            1709856000,
            0.98,
        );

        cargo.stamp(StationStamp::new(
            "signal-pipeline",
            "detect",
            1709856100,
            0.93,
        ));
        cargo.stamp(StationStamp::new("causality", "naranjo", 1709856200, 0.95));

        assert_eq!(cargo.custody_chain().hop_count(), 2);
        let expected = 0.93 * 0.95;
        assert!((cargo.custody_chain().cumulative_fidelity() - expected).abs() < 1e-10);
    }

    #[test]
    fn signal_cargo_cold_chain_upgrade() {
        let mut cargo = SignalCargo::from_result(
            single_signal_result(),
            "DRUG_X".into(),
            "Hepatotoxicity".into(),
            DataSource::Faers,
            1709856000,
            0.95,
        );

        assert_eq!(cargo.perishability(), Perishability::PROMPT_90);

        // Causality confirms — upgrade to Expedited
        cargo.upgrade_perishability(Perishability::EXPEDITED_15);
        assert_eq!(cargo.perishability(), Perishability::EXPEDITED_15);

        // Try downgrade — rejected
        cargo.upgrade_perishability(Perishability::PROMPT_90);
        assert_eq!(cargo.perishability(), Perishability::EXPEDITED_15);
    }
}
