// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! # Morgan / ECFP Fingerprints
//!
//! Implements circular (Morgan / ECFP-style) fingerprints with Tanimoto and
//! Dice similarity coefficients.
//!
//! ## Algorithm Overview
//!
//! 1. **Initialisation** — each atom receives an invariant hash derived from
//!    its atomic number, degree, formal charge, implicit hydrogen count,
//!    aromaticity flag, and ring membership.
//! 2. **Iteration** — for `radius` rounds each atom's hash is re-computed by
//!    combining its own hash with the sorted hashes of its neighbours.
//! 3. **Projection** — at every step the current atom hash is reduced modulo
//!    `nbits` to set a bit in the fingerprint.
//!
//! ## Primitive Grounding
//!
//! - ρ (Recursion): iterative Morgan hash update
//! - σ (Sequence): ordered neighbour hash accumulation
//! - N (Quantity): bit-vector popcount
//! - κ (Comparison): Tanimoto / Dice similarity scores
//!
//! ## Examples
//!
//! ```rust
//! use nexcore_molcore::fingerprint::{morgan_fingerprint, tanimoto, dice};
//! use nexcore_molcore::graph::MolGraph;
//! use nexcore_molcore::smiles::parse;
//!
//! let mol = parse("c1ccccc1").unwrap_or_default();
//! let g = MolGraph::from_molecule(mol);
//! let fp = morgan_fingerprint(&g, 2, 2048);
//!
//! assert_eq!(fp.size, 2048);
//! assert_eq!(fp.radius, 2);
//! assert!(fp.popcount() > 0);
//!
//! let t = tanimoto(&fp, &fp);
//! assert!((t - 1.0).abs() < f64::EPSILON);
//! ```

use std::collections::hash_map::DefaultHasher;
use std::hash::{Hash, Hasher};

use prima_chem::types::AtomId;

use crate::graph::MolGraph;

// ---------------------------------------------------------------------------
// Core type
// ---------------------------------------------------------------------------

/// Morgan / ECFP fingerprint as a dense bit vector.
///
/// Each bit position corresponds to a structural feature discovered during
/// the Morgan iteration.  Two fingerprints can be compared using
/// [`tanimoto`] or [`dice`].
///
/// ## Primitive Grounding
///
/// - N (Quantity): `size` and `popcount`
/// - π (Persistence): stored bit state
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Fingerprint {
    /// Dense bit vector (`true` = bit set).
    bits: Vec<bool>,
    /// Total number of bits in the vector.
    pub size: usize,
    /// Morgan radius used to generate this fingerprint.
    pub radius: u8,
}

impl Fingerprint {
    /// Create an empty (all-zero) fingerprint with `size` bits.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_molcore::fingerprint::Fingerprint;
    ///
    /// let fp = Fingerprint::new(2048, 2);
    /// assert_eq!(fp.size, 2048);
    /// assert_eq!(fp.radius, 2);
    /// assert_eq!(fp.popcount(), 0);
    /// ```
    #[must_use]
    pub fn new(size: usize, radius: u8) -> Self {
        Self {
            bits: vec![false; size],
            size,
            radius,
        }
    }

    /// Count the number of set bits (Hamming weight).
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_molcore::fingerprint::Fingerprint;
    ///
    /// let fp = Fingerprint::new(8, 2);
    /// assert_eq!(fp.popcount(), 0);
    /// ```
    #[must_use]
    pub fn popcount(&self) -> usize {
        self.bits.iter().filter(|&&b| b).count()
    }

    /// Get the bit at `idx`.
    ///
    /// Returns `false` for out-of-range indices rather than panicking.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_molcore::fingerprint::Fingerprint;
    ///
    /// let fp = Fingerprint::new(16, 2);
    /// assert!(!fp.get(0));
    /// assert!(!fp.get(9999)); // out of range → false
    /// ```
    #[must_use]
    pub fn get(&self, idx: usize) -> bool {
        self.bits.get(idx).copied().unwrap_or(false)
    }

    /// Set the bit at `idx` (no-op for out-of-range indices).
    fn set(&mut self, idx: usize) {
        if let Some(slot) = self.bits.get_mut(idx) {
            *slot = true;
        }
    }
}

// ---------------------------------------------------------------------------
// Hashing helpers
// ---------------------------------------------------------------------------

/// Mix `value` into `seed` using a multiplicative hash step.
///
/// Inspired by LCG / FNV-style mixing: cheap yet spreads bits well enough
/// for fingerprint projection.
#[inline]
fn hash_combine(seed: u64, value: u64) -> u64 {
    seed.wrapping_mul(6_364_136_223_846_793_005)
        .wrapping_add(value)
}

/// Hash a single `u64` value through [`DefaultHasher`].
///
/// Using the standard library hasher ensures portability without pulling in
/// any third-party dependency.
fn hash_u64(value: u64) -> u64 {
    let mut h = DefaultHasher::new();
    value.hash(&mut h);
    h.finish()
}

/// Compute the initial atomic invariant hash for atom `idx` in `graph`.
///
/// The invariant encodes:
/// - atomic number (element identity)
/// - graph degree (connectivity)
/// - formal charge
/// - implicit hydrogen count
/// - aromaticity flag
/// - ring membership
fn atom_invariant(graph: &MolGraph, idx: AtomId) -> u64 {
    let atom = match graph.molecule.atoms.get(idx) {
        Some(a) => a,
        None => return 0,
    };

    let mut h: u64 = 0;
    h = hash_combine(h, u64::from(atom.atomic_number));
    h = hash_combine(h, graph.degree(idx) as u64);
    // Encode charge as u64 by reinterpreting the i8 bit pattern.
    // `i8::to_ne_bytes` gives the raw byte, avoiding a sign-loss cast.
    h = hash_combine(h, u64::from(atom.charge.to_ne_bytes()[0]));
    h = hash_combine(h, u64::from(atom.implicit_h));
    h = hash_combine(h, u64::from(atom.aromatic));
    h = hash_combine(h, u64::from(graph.is_in_ring(idx)));
    hash_u64(h)
}

// ---------------------------------------------------------------------------
// Public API — fingerprint generation
// ---------------------------------------------------------------------------

/// Generate a Morgan / ECFP-style circular fingerprint.
///
/// # Parameters
///
/// - `graph`  — molecular graph to fingerprint.
/// - `radius` — number of Morgan iterations (0 = atom invariants only,
///   2 = ECFP4, 3 = ECFP6).
/// - `nbits`  — length of the output bit vector.
///
/// # Returns
///
/// A [`Fingerprint`] of length `nbits`.  If `nbits` is 0, the returned
/// fingerprint has `size = 0` and `popcount() == 0`.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::fingerprint::morgan_fingerprint;
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
///
/// let mol = parse("CCO").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let fp = morgan_fingerprint(&g, 2, 2048);
///
/// assert_eq!(fp.size, 2048);
/// assert_eq!(fp.radius, 2);
/// assert!(fp.popcount() > 0);
/// ```
#[must_use]
pub fn morgan_fingerprint(graph: &MolGraph, radius: u8, nbits: usize) -> Fingerprint {
    let n = graph.atom_count();
    let mut fp = Fingerprint::new(nbits, radius);

    if nbits == 0 || n == 0 {
        return fp;
    }

    // Step 1 — initialise with atom invariants.
    let mut hashes: Vec<u64> = (0..n).map(|i| atom_invariant(graph, i)).collect();

    // `nbits` as u64 is safe: usize ≤ u64 on every supported platform.
    let nbits_u64 = nbits as u64;

    // Project initial hashes.  The modulo result fits in usize because
    // it is strictly less than `nbits` which is itself a usize.
    for &h in &hashes {
        fp.set((h % nbits_u64) as usize);
    }

    // Step 2 — iterate for `radius` rounds.
    for _ in 0..radius {
        let prev = hashes.clone();
        for atom in 0..n {
            // Collect neighbour hashes and sort for canonical ordering.
            let mut neighbour_hashes: Vec<u64> = graph
                .neighbors(atom)
                .iter()
                .map(|&(nb, _order)| prev[nb])
                .collect();
            neighbour_hashes.sort_unstable();

            // Build new hash: start from own previous hash, mix in neighbours.
            let mut new_h = hash_combine(0, prev[atom]);
            for nh in neighbour_hashes {
                new_h = hash_combine(new_h, nh);
            }
            new_h = hash_u64(new_h);

            hashes[atom] = new_h;
            fp.set((new_h % nbits_u64) as usize);
        }
    }

    fp
}

// ---------------------------------------------------------------------------
// Public API — similarity metrics
// ---------------------------------------------------------------------------

/// Tanimoto (Jaccard) coefficient between two fingerprints.
///
/// ```text
/// tanimoto = |A ∩ B| / (|A| + |B| − |A ∩ B|)
/// ```
///
/// - Returns `1.0` when both fingerprints are empty (all-zero).
/// - Returns a value in `[0.0, 1.0]` otherwise.
/// - If the fingerprints have different `size`s only the shorter length is
///   compared; extra bits in the longer fingerprint are treated as unset in
///   the shorter one.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::fingerprint::{morgan_fingerprint, tanimoto};
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
///
/// let mol = parse("c1ccccc1").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let fp = morgan_fingerprint(&g, 2, 2048);
///
/// assert!((tanimoto(&fp, &fp) - 1.0).abs() < f64::EPSILON);
/// ```
#[must_use]
pub fn tanimoto(a: &Fingerprint, b: &Fingerprint) -> f64 {
    let shared = bit_intersection(a, b);
    let pop_a = a.popcount();
    let pop_b = b.popcount();
    let union = pop_a + pop_b - shared;

    if union == 0 {
        // Both fingerprints are all-zero.
        1.0
    } else {
        shared as f64 / union as f64
    }
}

/// Dice coefficient between two fingerprints.
///
/// ```text
/// dice = 2 × |A ∩ B| / (|A| + |B|)
/// ```
///
/// - Returns `1.0` when both fingerprints are empty.
/// - Returns a value in `[0.0, 1.0]` otherwise.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::fingerprint::{morgan_fingerprint, dice};
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
///
/// let mol = parse("CCO").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let fp = morgan_fingerprint(&g, 2, 2048);
///
/// assert!((dice(&fp, &fp) - 1.0).abs() < f64::EPSILON);
/// ```
#[must_use]
pub fn dice(a: &Fingerprint, b: &Fingerprint) -> f64 {
    let shared = bit_intersection(a, b);
    let pop_a = a.popcount();
    let pop_b = b.popcount();
    let denom = pop_a + pop_b;

    if denom == 0 {
        1.0
    } else {
        (2 * shared) as f64 / denom as f64
    }
}

// ---------------------------------------------------------------------------
// Private helpers
// ---------------------------------------------------------------------------

/// Count bits set in both `a` and `b` (cardinality of intersection).
///
/// Iterates over the shorter length to handle mismatched sizes gracefully.
fn bit_intersection(a: &Fingerprint, b: &Fingerprint) -> usize {
    let len = a.size.min(b.size);
    (0..len).filter(|&i| a.bits[i] && b.bits[i]).count()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::graph::MolGraph;
    use crate::smiles::parse;

    /// Parse a SMILES string and build a fingerprint with ECFP4 defaults.
    fn fp_for(smiles: &str) -> Fingerprint {
        let mol = parse(smiles).unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        morgan_fingerprint(&g, 2, 2048)
    }

    // ------------------------------------------------------------------
    // Self-similarity
    // ------------------------------------------------------------------

    #[test]
    fn test_tanimoto_self_similarity() {
        let fp = fp_for("c1ccccc1");
        let t = tanimoto(&fp, &fp);
        assert!(
            (t - 1.0).abs() < f64::EPSILON,
            "self-similarity must be 1.0, got {t}"
        );
    }

    #[test]
    fn test_tanimoto_identical_molecules() {
        let fp1 = fp_for("CCO");
        let fp2 = fp_for("CCO");
        let t = tanimoto(&fp1, &fp2);
        assert!(
            (t - 1.0).abs() < f64::EPSILON,
            "identical molecules must have tanimoto 1.0, got {t}"
        );
    }

    // ------------------------------------------------------------------
    // Different molecules
    // ------------------------------------------------------------------

    #[test]
    fn test_tanimoto_different_molecules() {
        let benzene = fp_for("c1ccccc1");
        let ethanol = fp_for("CCO");
        let t = tanimoto(&benzene, &ethanol);
        assert!(
            t < 1.0,
            "different molecules should have tanimoto < 1.0, got {t}"
        );
        assert!(t >= 0.0, "tanimoto must be non-negative, got {t}");
    }

    // ------------------------------------------------------------------
    // Dice coefficient
    // ------------------------------------------------------------------

    #[test]
    fn test_dice_self_similarity() {
        let fp = fp_for("CCO");
        let d = dice(&fp, &fp);
        assert!(
            (d - 1.0).abs() < f64::EPSILON,
            "dice self-similarity must be 1.0, got {d}"
        );
    }

    #[test]
    fn test_dice_different() {
        let fp1 = fp_for("c1ccccc1");
        let fp2 = fp_for("CCCC");
        let d = dice(&fp1, &fp2);
        assert!(
            d < 1.0,
            "dice for different molecules must be < 1.0, got {d}"
        );
        assert!(d >= 0.0, "dice must be non-negative, got {d}");
    }

    // ------------------------------------------------------------------
    // Popcount
    // ------------------------------------------------------------------

    #[test]
    fn test_fingerprint_has_bits_set() {
        let fp = fp_for("c1ccccc1");
        assert!(fp.popcount() > 0, "benzene fingerprint must have bits set");
    }

    // ------------------------------------------------------------------
    // Empty fingerprint edge case
    // ------------------------------------------------------------------

    #[test]
    fn test_empty_fingerprint_tanimoto() {
        let fp1 = Fingerprint::new(2048, 2);
        let fp2 = Fingerprint::new(2048, 2);
        let t = tanimoto(&fp1, &fp2);
        assert!(
            (t - 1.0).abs() < f64::EPSILON,
            "empty vs empty tanimoto must be 1.0, got {t}"
        );
    }

    #[test]
    fn test_empty_fingerprint_dice() {
        let fp1 = Fingerprint::new(2048, 2);
        let fp2 = Fingerprint::new(2048, 2);
        let d = dice(&fp1, &fp2);
        assert!(
            (d - 1.0).abs() < f64::EPSILON,
            "empty vs empty dice must be 1.0, got {d}"
        );
    }

    // ------------------------------------------------------------------
    // Radius 0
    // ------------------------------------------------------------------

    #[test]
    fn test_radius_zero() {
        let mol = parse("CCO").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        let fp = morgan_fingerprint(&g, 0, 2048);
        assert!(
            fp.popcount() > 0,
            "radius-0 fingerprint must still have bits set"
        );
    }

    // ------------------------------------------------------------------
    // Symmetry
    // ------------------------------------------------------------------

    #[test]
    fn test_tanimoto_symmetry() {
        let fp1 = fp_for("c1ccccc1");
        let fp2 = fp_for("CCO");
        let t1 = tanimoto(&fp1, &fp2);
        let t2 = tanimoto(&fp2, &fp1);
        assert!(
            (t1 - t2).abs() < f64::EPSILON,
            "tanimoto must be symmetric: {t1} != {t2}"
        );
    }

    #[test]
    fn test_dice_symmetry() {
        let fp1 = fp_for("c1ccccc1");
        let fp2 = fp_for("CCO");
        let d1 = dice(&fp1, &fp2);
        let d2 = dice(&fp2, &fp1);
        assert!(
            (d1 - d2).abs() < f64::EPSILON,
            "dice must be symmetric: {d1} != {d2}"
        );
    }

    // ------------------------------------------------------------------
    // Default parameters
    // ------------------------------------------------------------------

    #[test]
    fn test_default_ecfp4() {
        let fp = fp_for("c1ccccc1");
        assert_eq!(fp.size, 2048, "default size must be 2048");
        assert_eq!(fp.radius, 2, "default radius must be 2 (ECFP4)");
    }

    // ------------------------------------------------------------------
    // get() accessor
    // ------------------------------------------------------------------

    #[test]
    fn test_get_out_of_range_returns_false() {
        let fp = Fingerprint::new(16, 2);
        assert!(!fp.get(9999), "out-of-range get() must return false");
    }

    // ------------------------------------------------------------------
    // Fingerprint::new produces all-false bits
    // ------------------------------------------------------------------

    #[test]
    fn test_new_fingerprint_is_zero() {
        let fp = Fingerprint::new(64, 3);
        assert_eq!(fp.popcount(), 0, "new fingerprint must be all zeros");
        assert_eq!(fp.size, 64);
        assert_eq!(fp.radius, 3);
    }

    // ------------------------------------------------------------------
    // nbits = 0 edge case
    // ------------------------------------------------------------------

    #[test]
    fn test_zero_nbits() {
        let mol = parse("C").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        let fp = morgan_fingerprint(&g, 2, 0);
        assert_eq!(fp.size, 0);
        assert_eq!(fp.popcount(), 0);
    }

    // ------------------------------------------------------------------
    // Tanimoto range: similar molecules score between 0 and 1
    // ------------------------------------------------------------------

    #[test]
    fn test_tanimoto_range() {
        let fp1 = fp_for("c1ccccc1");
        let fp2 = fp_for("c1ccncc1"); // pyridine — structurally similar to benzene
        let t = tanimoto(&fp1, &fp2);
        assert!(t >= 0.0 && t <= 1.0, "tanimoto must be in [0,1], got {t}");
    }

    // ------------------------------------------------------------------
    // Radius affects fingerprint content
    // ------------------------------------------------------------------

    #[test]
    fn test_different_radii_may_differ() {
        let mol = parse("c1ccccc1").unwrap_or_default();
        let g = MolGraph::from_molecule(mol);
        let fp0 = morgan_fingerprint(&g, 0, 2048);
        let fp3 = morgan_fingerprint(&g, 3, 2048);
        // radius-3 must have at least as many bits set as radius-0 since
        // every radius-0 bit is also present in radius-3.
        assert!(
            fp3.popcount() >= fp0.popcount(),
            "higher radius must set at least as many bits"
        );
    }
}
