//! Mass state functions — thermodynamic analogs for primitive distributions.
//!
//! Computes entropy, Gibbs free energy, equilibrium status, and identifies
//! depleted/saturated primitives in an equation's inventory.

use crate::equation::BalancedEquation;
use crate::inventory::PrimitiveInventory;
use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use nexcore_lex_primitiva::tier::Tier;
use serde::{Deserialize, Serialize};
use std::collections::HashSet;

/// Number of operational primitive slots.
const SLOT_COUNT: usize = 15;

/// Thermodynamic state of a primitive inventory.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassState {
    inventory: PrimitiveInventory,
}

impl MassState {
    /// Create a mass state from an inventory.
    #[must_use]
    pub fn new(inventory: PrimitiveInventory) -> Self {
        Self { inventory }
    }

    /// Create a mass state from a balanced equation's product side.
    #[must_use]
    pub fn from_equation(eq: &BalancedEquation) -> Self {
        let inv = PrimitiveInventory::from_primitives(eq.concept.formula.primitives());
        Self::new(inv)
    }

    /// Total mass in daltons (sum of count * atomic_mass for each primitive).
    #[must_use]
    pub fn total_mass(&self) -> f64 {
        self.inventory.total_mass()
    }

    /// Shannon entropy of the primitive distribution.
    ///
    /// H = -sum(p_i * log2(p_i)) over all 15 slots where p_i > 0.
    #[must_use]
    pub fn entropy(&self) -> f64 {
        let total = self.inventory.total_count();
        if total == 0 {
            return 0.0;
        }
        let total_f = f64::from(total);
        let counts = self.inventory.counts();

        let mut h = 0.0_f64;
        for &c in counts {
            if c > 0 {
                let p = f64::from(c) / total_f;
                h -= p * p.log2();
            }
        }
        h
    }

    /// Maximum possible entropy: log2(15) — when all 15 slots are equal.
    #[must_use]
    pub fn max_entropy() -> f64 {
        (SLOT_COUNT as f64).log2()
    }

    /// Gibbs free energy analog: G = total_mass - (tier_diversity * entropy).
    ///
    /// `tier_diversity` = number of distinct tiers represented in the inventory.
    /// Lower G means more thermodynamically "favorable" (balanced) composition.
    #[must_use]
    pub fn gibbs_free_energy(&self) -> f64 {
        let tier_diversity = self.tier_diversity() as f64;
        self.total_mass() - (tier_diversity * self.entropy())
    }

    /// Is the distribution approximately uniform?
    ///
    /// True if entropy > 0.95 * max_entropy.
    #[must_use]
    pub fn is_equilibrium(&self) -> bool {
        if self.inventory.total_count() == 0 {
            return false;
        }
        self.entropy() > 0.95 * Self::max_entropy()
    }

    /// Primitives with count = 0 (unused slots).
    #[must_use]
    pub fn depleted(&self) -> Vec<LexPrimitiva> {
        let ops = PrimitiveInventory::operational_primitives();
        ops.iter()
            .filter(|&&p| self.inventory.count(p) == 0)
            .copied()
            .collect()
    }

    /// Primitives with count > mean + 2*stddev (outlier concentration).
    #[must_use]
    pub fn saturated(&self) -> Vec<LexPrimitiva> {
        let counts = self.inventory.counts();
        let total = self.inventory.total_count();
        if total == 0 {
            return Vec::new();
        }

        let n = SLOT_COUNT as f64;
        let mean = f64::from(total) / n;

        // Compute standard deviation
        let variance: f64 = counts
            .iter()
            .map(|&c| {
                let diff = f64::from(c) - mean;
                diff * diff
            })
            .sum::<f64>()
            / n;
        let stddev = variance.sqrt();

        let threshold = mean + 2.0 * stddev;

        let ops = PrimitiveInventory::operational_primitives();
        ops.iter()
            .filter(|&&p| f64::from(self.inventory.count(p)) > threshold)
            .copied()
            .collect()
    }

    /// The primitive inventory.
    #[must_use]
    pub fn inventory(&self) -> &PrimitiveInventory {
        &self.inventory
    }

    /// Count distinct tiers represented in the inventory.
    fn tier_diversity(&self) -> usize {
        let ops = PrimitiveInventory::operational_primitives();
        let mut tiers = HashSet::new();
        for &p in &ops {
            if self.inventory.count(p) > 0 {
                // Map each primitive to its tier via spatial_bridge classification
                let comp =
                    nexcore_lex_primitiva::primitiva::PrimitiveComposition::new(vec![p]);
                let tier = Tier::classify(&comp);
                tiers.insert(tier);
            }
        }
        tiers.len()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_total_mass_empty_is_zero() {
        let state = MassState::new(PrimitiveInventory::new());
        assert!(state.total_mass().abs() < 0.001);
    }

    #[test]
    fn test_total_mass_single_primitive() {
        let inv = PrimitiveInventory::from_primitives(&[LexPrimitiva::Quantity]);
        let state = MassState::new(inv);
        assert!(state.total_mass() > 0.0);
    }

    #[test]
    fn test_entropy_uniform_is_max() {
        // Put 1 of each operational primitive
        let ops = PrimitiveInventory::operational_primitives();
        let inv = PrimitiveInventory::from_primitives(&ops);
        let state = MassState::new(inv);
        let max_h = MassState::max_entropy();
        let h = state.entropy();
        assert!(
            (h - max_h).abs() < 0.001,
            "uniform entropy {h} should equal max {max_h}"
        );
    }

    #[test]
    fn test_entropy_single_prim_is_zero() {
        // All mass in one slot — entropy = 0 (since log2(1) = 0)
        let inv = PrimitiveInventory::from_primitives(&[LexPrimitiva::Causality]);
        let state = MassState::new(inv);
        assert!(
            state.entropy().abs() < 0.001,
            "single-slot entropy should be ~0, got {}",
            state.entropy()
        );
    }

    #[test]
    fn test_entropy_empty_is_zero() {
        let state = MassState::new(PrimitiveInventory::new());
        assert!(state.entropy().abs() < 0.001);
    }

    #[test]
    fn test_gibbs_calculation() {
        let inv = PrimitiveInventory::from_primitives(&[
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
        ]);
        let state = MassState::new(inv);
        let g = state.gibbs_free_energy();
        // G = total_mass - (tier_diversity * entropy)
        // Should be a finite number
        assert!(g.is_finite());
        // With 3 distinct primitives, entropy > 0, so G < total_mass
        assert!(g < state.total_mass() || state.entropy() == 0.0);
    }

    #[test]
    fn test_depleted_identifies_zeros() {
        let inv = PrimitiveInventory::from_primitives(&[LexPrimitiva::Causality]);
        let state = MassState::new(inv);
        let depleted = state.depleted();
        // Should have 14 depleted (all except Causality)
        assert_eq!(depleted.len(), 14);
        assert!(!depleted.contains(&LexPrimitiva::Causality));
    }

    #[test]
    fn test_saturated_identifies_outliers() {
        // Put 100 in one slot, 1 in others
        let mut inv = PrimitiveInventory::new();
        for _ in 0..100 {
            inv.add(LexPrimitiva::Causality);
        }
        let ops = PrimitiveInventory::operational_primitives();
        for &p in &ops {
            if p != LexPrimitiva::Causality {
                inv.add(p);
            }
        }
        let state = MassState::new(inv);
        let saturated = state.saturated();
        assert!(
            saturated.contains(&LexPrimitiva::Causality),
            "Causality with count 100 should be saturated"
        );
    }

    #[test]
    fn test_is_equilibrium_uniform() {
        let ops = PrimitiveInventory::operational_primitives();
        let inv = PrimitiveInventory::from_primitives(&ops);
        let state = MassState::new(inv);
        assert!(state.is_equilibrium());
    }

    #[test]
    fn test_is_equilibrium_nonuniform_false() {
        let inv = PrimitiveInventory::from_primitives(&[LexPrimitiva::Causality]);
        let state = MassState::new(inv);
        assert!(!state.is_equilibrium());
    }

    #[test]
    fn test_is_equilibrium_empty_false() {
        let state = MassState::new(PrimitiveInventory::new());
        assert!(!state.is_equilibrium());
    }

    #[test]
    fn test_from_equation() {
        let codec = crate::codec::StoichiometricCodec::builtin();
        if let Some(term) = codec.dictionary().lookup("Pharmacovigilance") {
            let state = MassState::from_equation(&term.equation);
            assert!(state.total_mass() > 0.0);
            assert!(state.entropy() > 0.0);
            assert!(!state.depleted().is_empty()); // not all 15 are used
        }
    }
}
