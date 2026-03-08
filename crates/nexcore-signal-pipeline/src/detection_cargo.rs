//! # Detection Cargo — Pipeline Results with Transport Metadata
//!
//! Wraps `DetectionResult` in typed cargo for the signal detection pipeline.
//! Each pipeline stage stamps the custody chain with its fidelity, producing
//! a complete provenance trail by the time results reach consumers.
//!
//! ## Cold-Chain Behavior
//!
//! Perishability derives from signal strength (same mapping as `SignalCargo`
//! in `nexcore-pv-core`):
//! - **None** → `Periodic` (no urgency)
//! - **Weak/Moderate** → `Prompt(90)` (routine monitoring window)
//! - **Strong/Critical** → `Expedited(15)` (regulatory urgency)
//!
//! ## Grounding
//!
//! `→(Causality) + ∂(Boundary) + ς(State) + π(Persistence)`
//! Each stage stamps → the custody chain preserves ∂ between stages →
//! ς accumulates fidelity → π persists the audit trail.

use crate::core::{DetectionResult, SignalStrength};
use crate::relay::stage_fidelity;
use nexcore_cargo::{
    Cargo, CustodyChain, DataSource, Destination, Perishability, Provenance, QueryParams,
    StationStamp,
};

/// Pipeline detection result enriched with cargo transport metadata.
///
/// Created by `Pipeline::run_with_cargo()`. Carries the full provenance trail
/// from ingestion through detection, with each stage's fidelity stamped
/// into the custody chain.
pub struct DetectionCargo {
    result: DetectionResult,
    provenance: Provenance,
    destination: Destination,
    perishability: Perishability,
    custody: CustodyChain,
}

impl DetectionCargo {
    /// Create cargo from a detection result, stamping the pipeline stages
    /// that produced it.
    ///
    /// `loaded_at` should be the pipeline start timestamp for consistency.
    #[must_use]
    pub fn from_pipeline_result(
        result: DetectionResult,
        source: DataSource,
        loaded_at: i64,
    ) -> Self {
        let mut query = QueryParams::empty();
        query.insert("drug", &result.pair.drug);
        query.insert("event", &result.pair.event);

        let source_confidence = strength_to_confidence(result.strength);
        let provenance = Provenance::new(source, query, loaded_at, source_confidence);
        let destination = Destination::SignalDetection;
        let perishability = strength_to_perishability(result.strength);

        let mut custody = CustodyChain::new();

        // Stamp the pipeline stages that produced this result.
        // Each stage's fidelity comes from the relay constants —
        // the same values used for axiom verification in relay.rs.
        custody.stamp(StationStamp::new(
            "signal-pipeline",
            "ingest",
            loaded_at,
            stage_fidelity::INGEST,
        ));
        custody.stamp(StationStamp::new(
            "signal-pipeline",
            "normalize",
            loaded_at,
            stage_fidelity::NORMALIZE,
        ));
        custody.stamp(StationStamp::new(
            "signal-pipeline",
            "detect",
            loaded_at,
            stage_fidelity::DETECT,
        ));
        custody.stamp(StationStamp::new(
            "signal-pipeline",
            "threshold",
            loaded_at,
            stage_fidelity::THRESHOLD,
        ));
        custody.stamp(StationStamp::new(
            "signal-pipeline",
            "store",
            loaded_at,
            stage_fidelity::STORE,
        ));

        Self {
            result,
            provenance,
            destination,
            perishability,
            custody,
        }
    }

    /// Number of algorithms that detected a signal (0-4).
    #[must_use]
    pub fn signal_count(&self) -> usize {
        let mut count = 0;
        if self.result.prr.is_some_and(|p| p.0 >= 2.0) {
            count += 1;
        }
        if self.result.ror.is_some_and(|r| r.0 >= 1.0) {
            count += 1;
        }
        if self.result.ic.is_some_and(|i| i.0 >= 0.0) {
            count += 1;
        }
        if self.result.ebgm.is_some_and(|e| e.0 >= 2.0) {
            count += 1;
        }
        count
    }
}

impl Cargo for DetectionCargo {
    type Payload = DetectionResult;

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

    fn payload(&self) -> &DetectionResult {
        &self.result
    }

    fn stamp(&mut self, stamp: StationStamp) {
        self.custody.stamp(stamp);
    }

    fn upgrade_perishability(&mut self, new: Perishability) {
        if new > self.perishability {
            self.perishability = new;
        }
    }
}

/// Map signal strength to source confidence for provenance.
#[allow(
    unreachable_patterns,
    reason = "SignalStrength is #[non_exhaustive] — wildcard required for forward compat"
)]
fn strength_to_confidence(strength: SignalStrength) -> f64 {
    match strength {
        SignalStrength::Critical => 0.98,
        SignalStrength::Strong => 0.90,
        SignalStrength::Moderate => 0.80,
        SignalStrength::Weak => 0.65,
        SignalStrength::None | _ => 0.50,
    }
}

/// Map signal strength to perishability (ICH E2D alignment).
#[allow(
    unreachable_patterns,
    reason = "SignalStrength is #[non_exhaustive] — wildcard required for forward compat"
)]
fn strength_to_perishability(strength: SignalStrength) -> Perishability {
    match strength {
        SignalStrength::Critical | SignalStrength::Strong => Perishability::EXPEDITED_15,
        SignalStrength::Moderate | SignalStrength::Weak => Perishability::PROMPT_90,
        SignalStrength::None | _ => Perishability::Periodic,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::core::{ChiSquare, ContingencyTable, DrugEventPair, Prr, Ror};
    use nexcore_chrono::DateTime;

    fn test_result(strength: SignalStrength) -> DetectionResult {
        DetectionResult::new(
            DrugEventPair::new("aspirin", "bleeding"),
            ContingencyTable::new(15, 100, 20, 10_000),
            Some(Prr(4.5)),
            Some(Ror(7.0)),
            None,
            None,
            ChiSquare(25.0),
            strength,
            DateTime::now(),
        )
    }

    #[test]
    fn cargo_stamps_five_pipeline_stages() {
        let result = test_result(SignalStrength::Strong);
        let cargo = DetectionCargo::from_pipeline_result(result, DataSource::Faers, 1709856000);

        assert_eq!(cargo.custody_chain().hop_count(), 5);
        // 5-stage fidelity: 0.98 * 0.96 * 0.93 * 0.97 * 0.99 ≈ 0.840
        let f = cargo.custody_chain().cumulative_fidelity();
        assert!(f > 0.83 && f < 0.85, "5-stage fidelity {f} should be ~0.84");
    }

    #[test]
    fn strong_signal_gets_expedited_perishability() {
        let result = test_result(SignalStrength::Strong);
        let cargo = DetectionCargo::from_pipeline_result(result, DataSource::Faers, 1709856000);

        assert_eq!(cargo.perishability(), Perishability::EXPEDITED_15);
        assert_eq!(cargo.destination(), Destination::SignalDetection);
    }

    #[test]
    fn weak_signal_gets_prompt_perishability() {
        let result = test_result(SignalStrength::Weak);
        let cargo = DetectionCargo::from_pipeline_result(result, DataSource::Faers, 1709856000);

        assert_eq!(cargo.perishability(), Perishability::PROMPT_90);
    }

    #[test]
    fn no_signal_gets_periodic() {
        let result = test_result(SignalStrength::None);
        let cargo = DetectionCargo::from_pipeline_result(result, DataSource::Faers, 1709856000);

        assert_eq!(cargo.perishability(), Perishability::Periodic);
    }

    #[test]
    fn provenance_carries_drug_event_params() {
        let result = test_result(SignalStrength::Moderate);
        let cargo = DetectionCargo::from_pipeline_result(result, DataSource::Faers, 1709856000);

        assert_eq!(
            cargo
                .provenance()
                .query
                .params
                .get("drug")
                .map(String::as_str),
            Some("aspirin")
        );
        assert_eq!(
            cargo
                .provenance()
                .query
                .params
                .get("event")
                .map(String::as_str),
            Some("bleeding")
        );
    }

    #[test]
    fn perishability_upgrade_works() {
        let result = test_result(SignalStrength::None);
        let mut cargo = DetectionCargo::from_pipeline_result(result, DataSource::Faers, 1709856000);

        assert_eq!(cargo.perishability(), Perishability::Periodic);

        // Simulates signal detection upgrading perishability mid-transit
        cargo.upgrade_perishability(Perishability::EXPEDITED_15);
        assert_eq!(cargo.perishability(), Perishability::EXPEDITED_15);
    }

    #[test]
    fn custody_meets_safety_threshold() {
        let result = test_result(SignalStrength::Strong);
        let cargo = DetectionCargo::from_pipeline_result(result, DataSource::Faers, 1709856000);

        // 5-stage fidelity ≈ 0.84 > 0.80 safety threshold
        assert!(cargo.custody_chain().meets_safety_threshold());
    }
}
