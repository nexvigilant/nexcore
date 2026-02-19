//! FAERS Bridge: Connects real-world FDA signal detection with DNA-based PV theory.
//!
//! This module provides tools to bridge traditional disproportionality metrics
//! (PRR, ROR) from FAERS with the DNA-based proximity and resonance metrics
//! from `pv_theory`.

use crate::error::Result;
use crate::pv_theory::{self, CausalityScore, DrugEventSignal};
use std::collections::HashMap;

/// Traditional disproportionality metrics for a drug-event pair.
#[derive(Debug, Clone, Default)]
pub struct Disproportionality {
    /// Proportional Reporting Ratio.
    pub prr: f64,
    /// Reporting Odds Ratio.
    pub ror: f64,
    /// Chi-square statistic (with Yates' correction).
    pub chi_square: f64,
    /// Case count (cell 'a' in 2x2).
    pub count: u32,
}

/// A 2x2 contingency table for signal detection.
///
/// |             | Event | Not Event |
/// |-------------|-------|-----------|
/// | Drug        |   a   |     b     |
/// | Not Drug    |   c   |     d     |
#[derive(Debug, Clone, Copy)]
pub struct ContingencyTable {
    pub a: u32,
    pub b: u32,
    pub c: u32,
    pub d: u32,
}

impl ContingencyTable {
    pub fn calculate_metrics(&self) -> Disproportionality {
        let a = self.a as f64;
        let b = self.b as f64;
        let c = self.c as f64;
        let d = self.d as f64;

        let prr = (a / (a + b)) / (c / (c + d));
        let ror = (a / b) / (c / d);

        // Chi-square with Yates correction
        let n = a + b + c + d;
        let numerator = n * ((a * d - b * c).abs() - n / 2.0).powi(2);
        let denominator = (a + b) * (c + d) * (a + c) * (b + d);
        let chi_square = if denominator > 0.0 {
            numerator / denominator
        } else {
            0.0
        };

        Disproportionality {
            prr: if prr.is_finite() { prr } else { 0.0 },
            ror: if ror.is_finite() { ror } else { 0.0 },
            chi_square,
            count: self.a,
        }
    }
}

/// Bridged signal detection result combining FAERS data and DNA math.
pub struct BridgedSignal {
    pub drug: String,
    pub event: String,
    pub traditional: Disproportionality,
    pub dna_based: DrugEventSignal,
    pub bridged_score: f64,
}

pub struct FaersSignalDetector;

impl FaersSignalDetector {
    /// Bridge traditional FAERS metrics with DNA-based PV theory.
    ///
    /// The bridged score weights traditional disproportionality (PRR)
    /// with DNA-based proximity and resonance.
    pub fn detect(drug: &str, event: &str, table: ContingencyTable) -> BridgedSignal {
        let traditional = table.calculate_metrics();
        let dna_based = pv_theory::detect_signal(drug, event);

        // Bridged score formula:
        // Combine normalized PRR (log scale) with DNA combined score.
        let norm_prr = (traditional.prr.ln().max(0.0) / 5.0).min(1.0);
        let bridged_score = 0.5 * norm_prr + 0.5 * dna_based.combined_score;

        BridgedSignal {
            drug: drug.to_string(),
            event: event.to_string(),
            traditional,
            dna_based,
            bridged_score,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_contingency_metrics() {
        // Example: a=10, b=90, c=5, d=995
        let table = ContingencyTable {
            a: 10,
            b: 90,
            c: 5,
            d: 995,
        };
        let metrics = table.calculate_metrics();

        assert!(metrics.prr > 1.0);
        assert!(metrics.ror > 1.0);
        assert!(metrics.chi_square > 0.0);
    }

    #[test]
    fn test_bridged_detection() {
        let drug = "WARFARIN";
        let event = "BLEEDING";
        // High disproportionality
        let table = ContingencyTable {
            a: 100,
            b: 1000,
            c: 10,
            d: 10000,
        };

        let bridged = FaersSignalDetector::detect(drug, event, table);

        assert!(bridged.traditional.prr > 1.0);
        assert!(bridged.dna_based.detected);
        assert!(bridged.bridged_score > 0.0);

        println!(
            "Bridged score for {}->{}: {:.4}",
            drug, event, bridged.bridged_score
        );
    }
}
