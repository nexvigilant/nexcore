//! # Detection/Counter-Detection Effectiveness Matrix
//!
//! An 8×8 matrix M where M\[i\]\[j\] = effectiveness of counter-primitive j
//! against sensing primitive i. Values in \[0.0, 1.0\].
//!
//! ## Mathematical Formalization
//!
//! ```text
//! M : SensingPrimitive × CounterPrimitive → \[0, 1\]
//! M\[i\]\[j\] = effectiveness of counter j at negating sensing primitive i
//!
//! Diagonal elements are primary counters (highest effectiveness).
//! Off-diagonal elements capture cross-effects.
//! ```
//!
//! ## Lex Primitiva Grounding
//! `EffectivenessMatrix` → μ (Mapping) — maps primitive pairs to effectiveness scores

use serde::{Deserialize, Serialize};

use crate::primitives::{CounterPrimitive, SensingPrimitive};

/// The 8×8 counter-effectiveness matrix.
///
/// `data[sensing_index][counter_index]` = effectiveness ∈ [0.0, 1.0]
///
/// Tier: T2-C (composed of μ + N + κ)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EffectivenessMatrix {
    /// 8×8 matrix: rows = sensing primitives, cols = counter-primitives
    data: [[f64; 8]; 8],
}

impl EffectivenessMatrix {
    /// Create a new matrix with all zeros (no countermeasures effective).
    pub fn zeros() -> Self {
        Self {
            data: [[0.0; 8]; 8],
        }
    }

    /// Create the default physics-grounded effectiveness matrix.
    ///
    /// Values derived from electromagnetic theory and material science:
    /// - Diagonal: primary counter-primitive effectiveness (0.7–0.9)
    /// - Off-diagonal: cross-effects where physics links primitives
    /// - Zero: no physical mechanism connects the pair
    pub fn default_physics() -> Self {
        // Row order: Reflection, Emission, Contrast, Boundary, Intensity, Frequency, Distance, Resolution
        // Col order: Absorption, ThermalEq, Homogenization, Diffusion, Attenuation, BandDenial, RangeDenial, SubResolution
        Self {
            data: [
                //  Abs   ThEq  Homo  Diff  Attn  Band  Range SubR
                [0.85, 0.00, 0.10, 0.05, 0.30, 0.20, 0.00, 0.00], // Reflection
                [0.05, 0.80, 0.15, 0.00, 0.10, 0.10, 0.00, 0.00], // Emission
                [0.20, 0.15, 0.85, 0.30, 0.20, 0.05, 0.00, 0.10], // Contrast
                [0.00, 0.00, 0.15, 0.80, 0.00, 0.00, 0.00, 0.25], // Boundary
                [0.40, 0.10, 0.15, 0.00, 0.85, 0.15, 0.30, 0.20], // Intensity
                [0.00, 0.00, 0.00, 0.00, 0.10, 0.90, 0.00, 0.00], // Frequency
                [0.00, 0.00, 0.00, 0.00, 0.15, 0.00, 0.85, 0.10], // Distance
                [0.00, 0.00, 0.05, 0.10, 0.00, 0.00, 0.10, 0.90], // Resolution
            ],
        }
    }

    /// Get effectiveness of a counter-primitive against a sensing primitive.
    pub fn get(&self, sensing: SensingPrimitive, counter: CounterPrimitive) -> f64 {
        // SAFETY: SensingPrimitive::index() and CounterPrimitive::index() both return
        // values in 0..8 via exhaustive match arms — every variant is enumerated
        // and maps to a distinct value in that range. The array size is 8×8.
        self.data[sensing.index()][counter.index()]
    }

    /// Set effectiveness of a counter-primitive against a sensing primitive.
    /// Clamps to [0.0, 1.0].
    pub fn set(&mut self, sensing: SensingPrimitive, counter: CounterPrimitive, value: f64) {
        // SAFETY: see get() — both index() methods return 0..8.
        self.data[sensing.index()][counter.index()] = value.clamp(0.0, 1.0);
    }

    /// Get the full row for a sensing primitive (all counter-effectiveness values).
    pub fn row(&self, sensing: SensingPrimitive) -> &[f64; 8] {
        // SAFETY: see get() — sensing.index() returns 0..8.
        &self.data[sensing.index()]
    }

    /// Get the full column for a counter-primitive (effectiveness against all sensing primitives).
    pub fn column(&self, counter: CounterPrimitive) -> [f64; 8] {
        // SAFETY: counter.index() returns 0..8 via exhaustive match on 8 variants.
        // col[i] is safe because enumerate() over self.data (8 rows) yields i ∈ 0..8,
        // and col is [f64; 8]. row[ci] is safe by the same argument as get().
        let ci = counter.index();
        let mut col = [0.0; 8];
        for (i, row) in self.data.iter().enumerate() {
            col[i] = row[ci];
        }
        col
    }

    /// Compute total counter-effectiveness of a set of countermeasures
    /// against a specific sensing primitive.
    ///
    /// Uses multiplicative composition:
    /// ```text
    /// E_total(sensing, counters) = 1 - ∏(1 - M[sensing][counter_i])
    /// ```
    ///
    /// This models diminishing returns: two 50% effective countermeasures
    /// yield 75% total, not 100%.
    pub fn combined_effectiveness(
        &self,
        sensing: SensingPrimitive,
        counters: &[CounterPrimitive],
    ) -> f64 {
        let pass_through: f64 = counters
            .iter()
            .map(|c| 1.0 - self.get(sensing, *c))
            .product();

        1.0 - pass_through
    }

    /// Compute the residual signature fraction after applying countermeasures.
    ///
    /// ```text
    /// S_residual / S_raw = ∏(1 - M[sensing][counter_i])
    /// ```
    ///
    /// Returns value in [0.0, 1.0]:
    /// - 1.0 = no reduction (countermeasures ineffective)
    /// - 0.0 = complete elimination (perfect stealth on this primitive)
    pub fn residual_fraction(
        &self,
        sensing: SensingPrimitive,
        counters: &[CounterPrimitive],
    ) -> f64 {
        counters
            .iter()
            .map(|c| 1.0 - self.get(sensing, *c))
            .product()
    }
}

impl Default for EffectivenessMatrix {
    fn default() -> Self {
        Self::default_physics()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn diagonal_dominance() {
        let m = EffectivenessMatrix::default_physics();
        // Each sensing primitive's primary counter should be the most effective
        for sp in SensingPrimitive::all() {
            let primary = sp.primary_counter();
            let primary_eff = m.get(*sp, primary);
            for cp in CounterPrimitive::all() {
                if *cp != primary {
                    assert!(
                        primary_eff >= m.get(*sp, *cp),
                        "Primary counter {:?} for {:?} should dominate, but {:?} is higher",
                        primary,
                        sp,
                        cp,
                    );
                }
            }
        }
    }

    #[test]
    fn values_in_range() {
        let m = EffectivenessMatrix::default_physics();
        for sp in SensingPrimitive::all() {
            for cp in CounterPrimitive::all() {
                let v = m.get(*sp, *cp);
                assert!((0.0..=1.0).contains(&v), "Value out of range: {v}");
            }
        }
    }

    #[test]
    fn combined_effectiveness_diminishing_returns() {
        let m = EffectivenessMatrix::default_physics();
        // Two countermeasures should be better than one, but less than sum
        let single = m.combined_effectiveness(
            SensingPrimitive::Reflection,
            &[CounterPrimitive::Absorption],
        );
        let double = m.combined_effectiveness(
            SensingPrimitive::Reflection,
            &[CounterPrimitive::Absorption, CounterPrimitive::Attenuation],
        );
        assert!(double > single, "Two counters should be better than one");
        assert!(
            double < single + m.get(SensingPrimitive::Reflection, CounterPrimitive::Attenuation),
            "Diminishing returns should apply"
        );
    }

    #[test]
    fn residual_plus_effectiveness_equals_one() {
        let m = EffectivenessMatrix::default_physics();
        let counters = &[
            CounterPrimitive::Absorption,
            CounterPrimitive::Homogenization,
        ];
        let sp = SensingPrimitive::Contrast;
        let eff = m.combined_effectiveness(sp, counters);
        let res = m.residual_fraction(sp, counters);
        let sum = eff + res;
        assert!(
            (sum - 1.0).abs() < 1e-10,
            "Effectiveness + residual should equal 1.0, got {sum}"
        );
    }
}
