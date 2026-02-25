//! Primitive inventory — counts of the 15 operational Lex Primitiva.
//!
//! Product (×) is axiomatic and excluded from the 15 operational slots.

use nexcore_lex_primitiva::molecular_weight::AtomicMass;
use nexcore_lex_primitiva::primitiva::LexPrimitiva;
use serde::{Deserialize, Serialize};

/// Number of operational primitives (excludes Product which is axiomatic).
const OPERATIONAL_COUNT: usize = 15;

/// The 15 operational primitives in enum order (excludes Product).
const OPERATIONAL: [LexPrimitiva; OPERATIONAL_COUNT] = [
    LexPrimitiva::Sequence,
    LexPrimitiva::Mapping,
    LexPrimitiva::State,
    LexPrimitiva::Recursion,
    LexPrimitiva::Void,
    LexPrimitiva::Boundary,
    LexPrimitiva::Frequency,
    LexPrimitiva::Existence,
    LexPrimitiva::Persistence,
    LexPrimitiva::Causality,
    LexPrimitiva::Comparison,
    LexPrimitiva::Quantity,
    LexPrimitiva::Location,
    LexPrimitiva::Irreversibility,
    LexPrimitiva::Sum,
];

/// Counts of each of the 15 operational primitives.
///
/// Product (×) is axiomatic — structurally present in every compound —
/// and does not participate in stoichiometric balancing.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PrimitiveInventory {
    counts: [u32; OPERATIONAL_COUNT],
}

impl PrimitiveInventory {
    /// Create an empty inventory (all zeros).
    #[must_use]
    pub fn new() -> Self {
        Self {
            counts: [0; OPERATIONAL_COUNT],
        }
    }

    /// Build an inventory from a slice of primitives, counting occurrences.
    /// Product primitives are silently skipped.
    #[must_use]
    pub fn from_primitives(prims: &[LexPrimitiva]) -> Self {
        let mut inv = Self::new();
        for &p in prims {
            inv.add(p);
        }
        inv
    }

    /// Increment the count for a primitive. Product is silently ignored.
    pub fn add(&mut self, prim: LexPrimitiva) {
        if let Some(idx) = Self::to_index(prim) {
            self.counts[idx] += 1;
        }
    }

    /// Get the count for a specific primitive. Returns 0 for Product.
    #[must_use]
    pub fn count(&self, prim: LexPrimitiva) -> u32 {
        match Self::to_index(prim) {
            Some(idx) => self.counts[idx],
            None => 0,
        }
    }

    /// Total number of primitive instances across all slots.
    #[must_use]
    pub fn total_count(&self) -> u32 {
        self.counts.iter().sum()
    }

    /// Total mass in daltons (sum of count * atomic_mass for each primitive).
    #[must_use]
    pub fn total_mass(&self) -> f64 {
        OPERATIONAL
            .iter()
            .enumerate()
            .map(|(i, &p)| f64::from(self.counts[i]) * AtomicMass::of(p).bits())
            .sum()
    }

    /// Per-slot deficit: self - other. Positive = self has more. Negative = other has more.
    #[must_use]
    pub fn deficit(&self, other: &Self) -> [i32; OPERATIONAL_COUNT] {
        let mut result = [0i32; OPERATIONAL_COUNT];
        for i in 0..OPERATIONAL_COUNT {
            result[i] = self.counts[i] as i32 - other.counts[i] as i32;
        }
        result
    }

    /// Check if two inventories have identical counts.
    #[must_use]
    pub fn is_equal(&self, other: &Self) -> bool {
        self.counts == other.counts
    }

    /// The 15 operational primitives (excludes Product).
    #[must_use]
    pub fn operational_primitives() -> [LexPrimitiva; OPERATIONAL_COUNT] {
        OPERATIONAL
    }

    /// Map a primitive to its index (0-14). Returns `None` for Product.
    #[must_use]
    pub fn to_index(prim: LexPrimitiva) -> Option<usize> {
        match prim {
            LexPrimitiva::Sequence => Some(0),
            LexPrimitiva::Mapping => Some(1),
            LexPrimitiva::State => Some(2),
            LexPrimitiva::Recursion => Some(3),
            LexPrimitiva::Void => Some(4),
            LexPrimitiva::Boundary => Some(5),
            LexPrimitiva::Frequency => Some(6),
            LexPrimitiva::Existence => Some(7),
            LexPrimitiva::Persistence => Some(8),
            LexPrimitiva::Causality => Some(9),
            LexPrimitiva::Comparison => Some(10),
            LexPrimitiva::Quantity => Some(11),
            LexPrimitiva::Location => Some(12),
            LexPrimitiva::Irreversibility => Some(13),
            LexPrimitiva::Sum => Some(14),
            LexPrimitiva::Product => None,
            _ => None, // non_exhaustive fallback
        }
    }

    /// Get the raw counts array.
    #[must_use]
    pub fn counts(&self) -> &[u32; OPERATIONAL_COUNT] {
        &self.counts
    }
}

impl Default for PrimitiveInventory {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_empty_inventory_all_zeros() {
        let inv = PrimitiveInventory::new();
        assert_eq!(inv.total_count(), 0);
        for &p in &OPERATIONAL {
            assert_eq!(inv.count(p), 0);
        }
    }

    #[test]
    fn test_add_primitive_increments() {
        let mut inv = PrimitiveInventory::new();
        inv.add(LexPrimitiva::Causality);
        assert_eq!(inv.count(LexPrimitiva::Causality), 1);
        inv.add(LexPrimitiva::Causality);
        assert_eq!(inv.count(LexPrimitiva::Causality), 2);
    }

    #[test]
    fn test_total_count() {
        let mut inv = PrimitiveInventory::new();
        inv.add(LexPrimitiva::Boundary);
        inv.add(LexPrimitiva::Quantity);
        inv.add(LexPrimitiva::Existence);
        assert_eq!(inv.total_count(), 3);
    }

    #[test]
    fn test_from_primitives_vec() {
        let prims = vec![
            LexPrimitiva::Existence,
            LexPrimitiva::State,
            LexPrimitiva::Boundary,
            LexPrimitiva::Existence,
        ];
        let inv = PrimitiveInventory::from_primitives(&prims);
        assert_eq!(inv.count(LexPrimitiva::Existence), 2);
        assert_eq!(inv.count(LexPrimitiva::State), 1);
        assert_eq!(inv.count(LexPrimitiva::Boundary), 1);
        assert_eq!(inv.total_count(), 4);
    }

    #[test]
    fn test_index_excludes_product() {
        assert!(PrimitiveInventory::to_index(LexPrimitiva::Product).is_none());
        // All other 15 should have an index
        for &p in &OPERATIONAL {
            assert!(PrimitiveInventory::to_index(p).is_some());
        }
    }

    #[test]
    fn test_product_silently_ignored() {
        let mut inv = PrimitiveInventory::new();
        inv.add(LexPrimitiva::Product);
        assert_eq!(inv.total_count(), 0);
        assert_eq!(inv.count(LexPrimitiva::Product), 0);
    }

    #[test]
    fn test_mass_calculation() {
        let inv = PrimitiveInventory::from_primitives(&[LexPrimitiva::Quantity]);
        let expected = AtomicMass::of(LexPrimitiva::Quantity).bits();
        let diff = (inv.total_mass() - expected).abs();
        assert!(
            diff < 0.001,
            "mass mismatch: got {} expected {}",
            inv.total_mass(),
            expected
        );
    }

    #[test]
    fn test_deficit_between_two_inventories() {
        let a = PrimitiveInventory::from_primitives(&[
            LexPrimitiva::Causality,
            LexPrimitiva::Causality,
            LexPrimitiva::Boundary,
        ]);
        let b =
            PrimitiveInventory::from_primitives(&[LexPrimitiva::Causality, LexPrimitiva::Quantity]);
        let deficit = a.deficit(&b);
        // Causality: 2-1 = 1
        assert_eq!(deficit[9], 1);
        // Boundary: 1-0 = 1
        assert_eq!(deficit[5], 1);
        // Quantity: 0-1 = -1
        assert_eq!(deficit[11], -1);
    }

    #[test]
    fn test_is_equal() {
        let a =
            PrimitiveInventory::from_primitives(&[LexPrimitiva::Existence, LexPrimitiva::Boundary]);
        let b =
            PrimitiveInventory::from_primitives(&[LexPrimitiva::Existence, LexPrimitiva::Boundary]);
        assert!(a.is_equal(&b));
    }

    #[test]
    fn test_is_not_equal() {
        let a = PrimitiveInventory::from_primitives(&[LexPrimitiva::Existence]);
        let b = PrimitiveInventory::from_primitives(&[LexPrimitiva::Boundary]);
        assert!(!a.is_equal(&b));
    }
}
