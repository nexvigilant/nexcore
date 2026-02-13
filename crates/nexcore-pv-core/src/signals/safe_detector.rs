//! # Safe Signal Detector (L2 Molecule)
//!
//! Composes L1 atoms to provide axiomatically coupled signal detection.

use super::atoms::{calculate_safety_distance, score_signal_trust};
use super::evaluate_signal_complete;
use crate::foundation_compat::traits::{SafeCalculable, VigilantResult};
use crate::thresholds::SignalCriteria;
use crate::types::{CompleteSignalResult, ContingencyTable};

/// Safe signal detector that couples signal detection with axiomatic safety margins.
pub struct SafeSignalDetector {
    /// The signal detection criteria to use.
    pub criteria: SignalCriteria,
}

impl SafeSignalDetector {
    /// Create a new safe signal detector with the given criteria.
    #[must_use]
    pub fn new(criteria: SignalCriteria) -> Self {
        Self { criteria }
    }
}

impl SafeCalculable for SafeSignalDetector {
    type Input = ContingencyTable;
    type Output = CompleteSignalResult;

    fn calculate_safe(&self, input: Self::Input) -> VigilantResult<Self::Output> {
        let res = evaluate_signal_complete(&input, &self.criteria);

        // Extract values before moving res
        let safety_margin = calculate_safety_distance(
            res.prr.point_estimate,
            res.ror.lower_ci,
            res.ic.lower_ci,
            res.ebgm.lower_ci,
            res.n,
        );
        let trust_score = score_signal_trust(res.n);

        VigilantResult {
            data: res,
            safety_margin,
            trust_score,
        }
    }
}
