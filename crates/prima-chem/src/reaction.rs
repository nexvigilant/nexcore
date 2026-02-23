// Copyright © 2026 NexVigilant LLC. All Rights Reserved.
// Intellectual Property of Matthew Alexander Campion, PharmD

//! Chemical reaction modeling.
//!
//! ## Primitive Grounding: → (Causality) + N (Quantity for coefficients) + ∂ (Conservation boundary)
//!
//! A reaction is a causal transformation: reactants → products.
//! Conservation (∂) enforces that atom counts are preserved across the boundary.
//! Stoichiometric coefficients (N) scale each molecule's contribution.
//!
//! ## Tier Classification
//!
//! - [`ReactionComponent`]: T2-P (N × μ) — coefficient scales molecule
//! - [`Reaction`]: T2-C (→ + Σ + ∂) — causal transform with conservation boundary
//!
//! ## Example: Water synthesis
//!
//! ```rust
//! # use prima_chem::reaction::{Reaction, ReactionComponent};
//! # use prima_chem::types::{Atom, Bond, Molecule};
//! # fn main() -> Result<(), Box<dyn std::error::Error>> {
//! // H2
//! let mut h2 = Molecule::new();
//! h2.add_atom(Atom::hydrogen());
//! h2.add_atom(Atom::hydrogen());
//!
//! // O2
//! let mut o2 = Molecule::new();
//! o2.add_atom(Atom::oxygen());
//! o2.add_atom(Atom::oxygen());
//!
//! // H2O
//! let mut water = Molecule::new();
//! let o = water.add_atom(Atom::oxygen());
//! let h1 = water.add_atom(Atom::hydrogen());
//! let h2a = water.add_atom(Atom::hydrogen());
//! water.add_bond(Bond::single(o, h1))?;
//! water.add_bond(Bond::single(o, h2a))?;
//!
//! // 2H2 + O2 → 2H2O
//! let rxn = Reaction::new(
//!     vec![ReactionComponent::new(h2, 2), ReactionComponent::new(o2, 1)],
//!     vec![ReactionComponent::new(water, 2)],
//! );
//! assert!(rxn.is_balanced());
//! # Ok(())
//! # }
//! ```

use std::collections::HashMap;

use crate::error::{ChemError, ChemResult};
use crate::types::Molecule;

/// A molecule with a stoichiometric coefficient in a reaction.
///
/// ## Primitive Grounding: N (Quantity) + μ (Mapping to molecule)
///
/// `ReactionComponent { molecule: M, coefficient: k }` represents k·M.
/// The coefficient N scales the molecule's participation in the transformation.
///
/// # Examples
///
/// ```rust
/// use prima_chem::reaction::ReactionComponent;
/// use prima_chem::types::Molecule;
///
/// let mut mol = Molecule::new();
/// mol.name = Some("H2".to_string());
/// let comp = ReactionComponent::new(mol, 2);
/// assert_eq!(comp.coefficient, 2);
/// ```
#[derive(Debug, Clone)]
pub struct ReactionComponent {
    /// The molecule participating in the reaction.
    pub molecule: Molecule,
    /// Stoichiometric coefficient (how many units of the molecule participate).
    pub coefficient: u32,
}

impl ReactionComponent {
    /// Create a new reaction component.
    ///
    /// # Arguments
    ///
    /// * `molecule` - The molecule
    /// * `coefficient` - Stoichiometric coefficient (≥ 1)
    #[must_use]
    pub fn new(molecule: Molecule, coefficient: u32) -> Self {
        Self {
            molecule,
            coefficient,
        }
    }
}

/// A chemical reaction: reactants → products.
///
/// ## Primitive Grounding: → (Causality) + N (Quantity for coefficients) + ∂ (Conservation boundary)
///
/// Reactants are causally transformed into products (→).
/// Atom conservation (∂) enforces that element counts are preserved.
/// Stoichiometric coefficients (N) balance the transformation.
#[derive(Debug, Clone)]
pub struct Reaction {
    /// Reactants — left side of the → operator.
    pub reactants: Vec<ReactionComponent>,
    /// Products — right side of the → operator.
    pub products: Vec<ReactionComponent>,
    /// Optional human-readable reaction name.
    pub name: Option<String>,
    /// Optional reaction conditions (temperature, pressure, catalyst, solvent).
    pub conditions: Option<String>,
}

impl Reaction {
    /// Create a new reaction.
    ///
    /// # Arguments
    ///
    /// * `reactants` - Left-side components
    /// * `products` - Right-side components
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prima_chem::reaction::Reaction;
    ///
    /// let rxn = Reaction::new(vec![], vec![]);
    /// assert!(!rxn.is_balanced());
    /// ```
    #[must_use]
    pub fn new(reactants: Vec<ReactionComponent>, products: Vec<ReactionComponent>) -> Self {
        Self {
            reactants,
            products,
            name: None,
            conditions: None,
        }
    }

    /// Attach a name to this reaction (builder pattern).
    #[must_use]
    pub fn with_name(mut self, name: &str) -> Self {
        self.name = Some(name.to_string());
        self
    }

    /// Attach reaction conditions (builder pattern).
    #[must_use]
    pub fn with_conditions(mut self, conditions: &str) -> Self {
        self.conditions = Some(conditions.to_string());
        self
    }

    /// Check whether the reaction satisfies atom conservation.
    ///
    /// Returns `true` when, for every element present on either side,
    /// the total atom count (coefficient × atom count per molecule) matches
    /// between reactants and products.
    ///
    /// An empty reactant or product list is considered unbalanced.
    ///
    /// ## Primitive Grounding: ∂ (Conservation boundary)
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use prima_chem::reaction::{Reaction, ReactionComponent};
    /// # use prima_chem::types::{Atom, Bond, Molecule};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // 2H2 + O2 → 2H2O is balanced
    /// let mut h2 = Molecule::new();
    /// h2.add_atom(Atom::hydrogen());
    /// h2.add_atom(Atom::hydrogen());
    ///
    /// let mut o2 = Molecule::new();
    /// o2.add_atom(Atom::oxygen());
    /// o2.add_atom(Atom::oxygen());
    ///
    /// let mut water = Molecule::new();
    /// let o = water.add_atom(Atom::oxygen());
    /// let h1 = water.add_atom(Atom::hydrogen());
    /// let h2a = water.add_atom(Atom::hydrogen());
    /// water.add_bond(Bond::single(o, h1))?;
    /// water.add_bond(Bond::single(o, h2a))?;
    ///
    /// let rxn = Reaction::new(
    ///     vec![ReactionComponent::new(h2, 2), ReactionComponent::new(o2, 1)],
    ///     vec![ReactionComponent::new(water, 2)],
    /// );
    /// assert!(rxn.is_balanced());
    /// # Ok(())
    /// # }
    /// ```
    #[must_use]
    pub fn is_balanced(&self) -> bool {
        if self.reactants.is_empty() || self.products.is_empty() {
            return false;
        }

        let reactant_counts = element_counts(&self.reactants);
        let product_counts = element_counts(&self.products);

        let all_elements: std::collections::HashSet<&String> = reactant_counts
            .keys()
            .chain(product_counts.keys())
            .collect();

        for element in all_elements {
            let r = reactant_counts.get(element).copied().unwrap_or(0);
            let p = product_counts.get(element).copied().unwrap_or(0);
            if r != p {
                return false;
            }
        }

        true
    }

    /// Attempt to balance the reaction by finding integer stoichiometric coefficients.
    ///
    /// Uses brute-force search over coefficients in `1..=10` for reactions with
    /// up to 4 total components (reactants + products combined). On success,
    /// the coefficients of all components are updated in place.
    ///
    /// The search finds the lexicographically smallest valid coefficient set,
    /// which for well-posed reactions corresponds to the simplest whole-number ratios.
    ///
    /// ## Primitive Grounding: → (Causality) + N (Quantity) + ∂ (Conservation)
    ///
    /// # Errors
    ///
    /// Returns [`ChemError::Reaction`] if:
    /// - The reaction has more than 4 total components
    /// - No valid coefficients ≤ 10 exist that balance the reaction
    ///
    /// On error, all coefficients are reset to 1.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use prima_chem::reaction::{Reaction, ReactionComponent};
    /// # use prima_chem::types::{Atom, Bond, Molecule};
    /// # fn main() -> Result<(), Box<dyn std::error::Error>> {
    /// // H2 + O2 → H2O (unbalanced) — balances to 2H2 + O2 → 2H2O
    /// let mut h2 = Molecule::new();
    /// h2.add_atom(Atom::hydrogen());
    /// h2.add_atom(Atom::hydrogen());
    ///
    /// let mut o2 = Molecule::new();
    /// o2.add_atom(Atom::oxygen());
    /// o2.add_atom(Atom::oxygen());
    ///
    /// let mut water = Molecule::new();
    /// let o = water.add_atom(Atom::oxygen());
    /// let h1 = water.add_atom(Atom::hydrogen());
    /// let h2a = water.add_atom(Atom::hydrogen());
    /// water.add_bond(Bond::single(o, h1))?;
    /// water.add_bond(Bond::single(o, h2a))?;
    ///
    /// let mut rxn = Reaction::new(
    ///     vec![ReactionComponent::new(h2, 1), ReactionComponent::new(o2, 1)],
    ///     vec![ReactionComponent::new(water, 1)],
    /// );
    /// assert!(rxn.balance().is_ok());
    /// assert!(rxn.is_balanced());
    /// # Ok(())
    /// # }
    /// ```
    pub fn balance(&mut self) -> ChemResult<()> {
        if self.is_balanced() {
            return Ok(());
        }

        let total = self.reactants.len() + self.products.len();

        if total > 4 {
            return Err(ChemError::Reaction(
                "Balancing only supported for reactions with ≤ 4 total components".to_string(),
            ));
        }

        const MAX_COEFF: u32 = 10;

        // Brute-force: at most 10^4 = 10,000 iterations — fast for small reactions
        for coeffs in CoeffIterator::new(total, MAX_COEFF) {
            self.apply_coefficients(&coeffs);
            if self.is_balanced() {
                return Ok(());
            }
        }

        // Reset to unit coefficients on failure
        self.apply_coefficients(&vec![1; total]);
        Err(ChemError::Reaction(
            "Could not balance reaction with coefficients ≤ 10".to_string(),
        ))
    }

    /// Calculate the atom economy of this reaction.
    ///
    /// Atom economy = (MW of desired product × its coefficient / total MW of reactants) × 100%
    ///
    /// The "desired product" is the **first** entry in `products`.
    /// Returns `0.0` when there are no products, no reactants, or total reactant
    /// molecular weight is zero.
    ///
    /// ## Primitive Grounding: N (Quantity) + κ (Comparison of MW ratios)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prima_chem::reaction::Reaction;
    ///
    /// let rxn = Reaction::new(vec![], vec![]);
    /// assert_eq!(rxn.atom_economy(), 0.0);
    /// ```
    #[must_use]
    pub fn atom_economy(&self) -> f64 {
        let Some(desired) = self.products.first() else {
            return 0.0;
        };

        let total_reactant_mw: f64 = self
            .reactants
            .iter()
            .map(|c| c.coefficient as f64 * c.molecule.molecular_weight())
            .sum();

        if total_reactant_mw == 0.0 {
            return 0.0;
        }

        let desired_mw = desired.coefficient as f64 * desired.molecule.molecular_weight();
        (desired_mw / total_reactant_mw) * 100.0
    }

    /// Format the reaction as a human-readable equation string.
    ///
    /// Format: `"2H2 + O2 → 2H2O"`
    ///
    /// Each molecule is labeled by its [`Molecule::name`] if set, otherwise
    /// by its molecular formula (Hill notation). A coefficient of 1 is omitted.
    ///
    /// ## Primitive Grounding: σ (Sequence) + μ (Mapping labels to symbols)
    ///
    /// # Examples
    ///
    /// ```rust
    /// use prima_chem::reaction::{Reaction, ReactionComponent};
    /// use prima_chem::types::{Atom, Molecule};
    ///
    /// let mut h2 = Molecule::new();
    /// h2.name = Some("H2".to_string());
    /// h2.add_atom(Atom::hydrogen());
    /// h2.add_atom(Atom::hydrogen());
    ///
    /// let mut water = Molecule::new();
    /// water.name = Some("H2O".to_string());
    /// water.add_atom(Atom::oxygen());
    /// water.add_atom(Atom::hydrogen());
    /// water.add_atom(Atom::hydrogen());
    ///
    /// let rxn = Reaction::new(
    ///     vec![ReactionComponent::new(h2, 2)],
    ///     vec![ReactionComponent::new(water, 1)],
    /// );
    /// let eq = rxn.to_string_equation();
    /// assert!(eq.contains("2H2"));
    /// assert!(eq.contains('→'));
    /// assert!(eq.contains("H2O"));
    /// ```
    #[must_use]
    pub fn to_string_equation(&self) -> String {
        let format_side = |components: &[ReactionComponent]| -> String {
            components
                .iter()
                .map(|c| {
                    let label = c
                        .molecule
                        .name
                        .clone()
                        .unwrap_or_else(|| c.molecule.molecular_formula());
                    if c.coefficient == 1 {
                        label
                    } else {
                        format!("{}{}", c.coefficient, label)
                    }
                })
                .collect::<Vec<_>>()
                .join(" + ")
        };

        let lhs = format_side(&self.reactants);
        let rhs = format_side(&self.products);
        format!("{lhs} → {rhs}")
    }

    /// Apply a flat coefficient slice to all components (reactants first, then products).
    fn apply_coefficients(&mut self, coeffs: &[u32]) {
        let n_reactants = self.reactants.len();
        for (i, &c) in coeffs.iter().enumerate() {
            if i < n_reactants {
                self.reactants[i].coefficient = c;
            } else {
                self.products[i - n_reactants].coefficient = c;
            }
        }
    }
}

// ── Private helpers ──────────────────────────────────────────────────────────

/// Compute per-element atom counts across a list of reaction components.
///
/// Each atom's element symbol is counted once per occurrence, scaled by the
/// component's stoichiometric coefficient. Implicit hydrogens on each atom
/// are also included.
fn element_counts(components: &[ReactionComponent]) -> HashMap<String, u64> {
    let mut counts: HashMap<String, u64> = HashMap::new();
    for comp in components {
        let coeff = comp.coefficient as u64;
        for atom in &comp.molecule.atoms {
            if let Some(elem) = atom.element() {
                *counts.entry(elem.symbol.to_string()).or_insert(0) += coeff;
            }
            if atom.implicit_h > 0 {
                *counts.entry("H".to_string()).or_insert(0) += coeff * (atom.implicit_h as u64);
            }
        }
    }
    counts
}

/// Lexicographic iterator over all coefficient tuples of length `n` with values in `1..=max`.
///
/// For `n = 3` and `max = 2` this yields:
/// `[1,1,1], [1,1,2], [1,2,1], [1,2,2], [2,1,1], [2,1,2], [2,2,1], [2,2,2]`
///
/// Maximum iteration count: `max^n`. For n=4, max=10 → 10,000 iterations.
struct CoeffIterator {
    current: Vec<u32>,
    max: u32,
    done: bool,
}

impl CoeffIterator {
    fn new(n: usize, max: u32) -> Self {
        if n == 0 {
            Self {
                current: Vec::new(),
                max,
                done: true,
            }
        } else {
            Self {
                current: vec![1; n],
                max,
                done: false,
            }
        }
    }
}

impl Iterator for CoeffIterator {
    type Item = Vec<u32>;

    fn next(&mut self) -> Option<Self::Item> {
        if self.done {
            return None;
        }

        let result = self.current.clone();

        // Increment like a mixed-radix counter (rightmost digit first)
        let n = self.current.len();
        let mut carry = true;
        for i in (0..n).rev() {
            if carry {
                self.current[i] += 1;
                if self.current[i] > self.max {
                    self.current[i] = 1;
                    // carry continues
                } else {
                    carry = false;
                }
            }
        }
        if carry {
            // All positions wrapped — exhausted
            self.done = true;
        }

        Some(result)
    }
}

// ── Tests ────────────────────────────────────────────────────────────────────

#[cfg(test)]
mod tests {
    use super::*;
    use crate::types::{Atom, Bond};

    // ── Molecule constructors ────────────────────────────────────────────────

    fn make_h2() -> Molecule {
        let mut mol = Molecule::new();
        mol.add_atom(Atom::hydrogen());
        mol.add_atom(Atom::hydrogen());
        mol.name = Some("H2".to_string());
        mol
    }

    fn make_o2() -> Molecule {
        let mut mol = Molecule::new();
        mol.add_atom(Atom::oxygen());
        mol.add_atom(Atom::oxygen());
        mol.name = Some("O2".to_string());
        mol
    }

    fn make_h2o() -> Molecule {
        let mut mol = Molecule::new();
        let o = mol.add_atom(Atom::oxygen());
        let h1 = mol.add_atom(Atom::hydrogen());
        let h2 = mol.add_atom(Atom::hydrogen());
        assert!(mol.add_bond(Bond::single(o, h1)).is_ok());
        assert!(mol.add_bond(Bond::single(o, h2)).is_ok());
        mol.name = Some("H2O".to_string());
        mol
    }

    fn make_ch4() -> Molecule {
        let mut mol = Molecule::new();
        let c = mol.add_atom(Atom::carbon());
        for _ in 0..4 {
            let h = mol.add_atom(Atom::hydrogen());
            assert!(mol.add_bond(Bond::single(c, h)).is_ok());
        }
        mol.name = Some("CH4".to_string());
        mol
    }

    fn make_co2() -> Molecule {
        let mut mol = Molecule::new();
        let c = mol.add_atom(Atom::carbon());
        let o1 = mol.add_atom(Atom::oxygen());
        let o2 = mol.add_atom(Atom::oxygen());
        assert!(mol.add_bond(Bond::double(c, o1)).is_ok());
        assert!(mol.add_bond(Bond::double(c, o2)).is_ok());
        mol.name = Some("CO2".to_string());
        mol
    }

    // ── CoeffIterator ────────────────────────────────────────────────────────

    #[test]
    fn test_coeff_iterator_n1() {
        let vals: Vec<_> = CoeffIterator::new(1, 3).collect();
        assert_eq!(vals, vec![vec![1], vec![2], vec![3]]);
    }

    #[test]
    fn test_coeff_iterator_n2() {
        let vals: Vec<_> = CoeffIterator::new(2, 2).collect();
        assert_eq!(vals, vec![vec![1, 1], vec![1, 2], vec![2, 1], vec![2, 2],]);
    }

    #[test]
    fn test_coeff_iterator_empty() {
        let vals: Vec<_> = CoeffIterator::new(0, 5).collect();
        assert!(vals.is_empty());
    }

    // ── is_balanced ──────────────────────────────────────────────────────────

    #[test]
    fn test_empty_reaction_is_not_balanced() {
        let rxn = Reaction::new(vec![], vec![]);
        assert!(!rxn.is_balanced());
    }

    #[test]
    fn test_no_products_is_not_balanced() {
        let rxn = Reaction::new(vec![ReactionComponent::new(make_h2(), 1)], vec![]);
        assert!(!rxn.is_balanced());
    }

    #[test]
    fn test_balanced_water_synthesis() {
        // 2H2 + O2 → 2H2O
        let rxn = Reaction::new(
            vec![
                ReactionComponent::new(make_h2(), 2),
                ReactionComponent::new(make_o2(), 1),
            ],
            vec![ReactionComponent::new(make_h2o(), 2)],
        );
        assert!(rxn.is_balanced());
    }

    #[test]
    fn test_unbalanced_water_synthesis() {
        // H2 + O2 → H2O (unbalanced)
        let rxn = Reaction::new(
            vec![
                ReactionComponent::new(make_h2(), 1),
                ReactionComponent::new(make_o2(), 1),
            ],
            vec![ReactionComponent::new(make_h2o(), 1)],
        );
        assert!(!rxn.is_balanced());
    }

    #[test]
    fn test_balanced_methane_combustion() {
        // CH4 + 2O2 → CO2 + 2H2O
        let rxn = Reaction::new(
            vec![
                ReactionComponent::new(make_ch4(), 1),
                ReactionComponent::new(make_o2(), 2),
            ],
            vec![
                ReactionComponent::new(make_co2(), 1),
                ReactionComponent::new(make_h2o(), 2),
            ],
        );
        assert!(rxn.is_balanced());
    }

    // ── balance ──────────────────────────────────────────────────────────────

    #[test]
    fn test_balance_water_synthesis() {
        // Start with all coefficients = 1 → should balance to 2H2 + O2 → 2H2O
        let mut rxn = Reaction::new(
            vec![
                ReactionComponent::new(make_h2(), 1),
                ReactionComponent::new(make_o2(), 1),
            ],
            vec![ReactionComponent::new(make_h2o(), 1)],
        );

        assert!(rxn.balance().is_ok());
        assert!(rxn.is_balanced());

        // Canonical coefficients: 2H2 + O2 → 2H2O
        assert_eq!(rxn.reactants[0].coefficient, 2); // H2
        assert_eq!(rxn.reactants[1].coefficient, 1); // O2
        assert_eq!(rxn.products[0].coefficient, 2); // H2O
    }

    #[test]
    fn test_balance_already_balanced_is_noop() {
        // CH4 + 2O2 → CO2 + 2H2O — coefficients must be unchanged
        let mut rxn = Reaction::new(
            vec![
                ReactionComponent::new(make_ch4(), 1),
                ReactionComponent::new(make_o2(), 2),
            ],
            vec![
                ReactionComponent::new(make_co2(), 1),
                ReactionComponent::new(make_h2o(), 2),
            ],
        );

        assert!(rxn.balance().is_ok());
        assert!(rxn.is_balanced());
        assert_eq!(rxn.reactants[0].coefficient, 1);
        assert_eq!(rxn.reactants[1].coefficient, 2);
        assert_eq!(rxn.products[0].coefficient, 1);
        assert_eq!(rxn.products[1].coefficient, 2);
    }

    #[test]
    fn test_balance_too_many_components_returns_error() {
        // 5 components total: exceeds the 4-component limit
        let reactants: Vec<_> = (0..3)
            .map(|_| ReactionComponent::new(make_h2(), 1))
            .collect();
        let products: Vec<_> = (0..2)
            .map(|_| ReactionComponent::new(make_h2o(), 1))
            .collect();
        let mut rxn = Reaction::new(reactants, products);

        let result = rxn.balance();
        assert!(result.is_err());
        if let Err(ChemError::Reaction(msg)) = result {
            assert!(msg.contains('4'));
        }
    }

    // ── atom_economy ─────────────────────────────────────────────────────────

    #[test]
    fn test_atom_economy_empty() {
        let rxn = Reaction::new(vec![], vec![]);
        assert_eq!(rxn.atom_economy(), 0.0);
    }

    #[test]
    fn test_atom_economy_water_synthesis() {
        // 2H2 + O2 → 2H2O — all atoms end up in the product (≈ 100%)
        let rxn = Reaction::new(
            vec![
                ReactionComponent::new(make_h2(), 2),
                ReactionComponent::new(make_o2(), 1),
            ],
            vec![ReactionComponent::new(make_h2o(), 2)],
        );

        let ae = rxn.atom_economy();
        // 2×H2O MW / (2×H2 MW + O2 MW) × 100 ≈ 100%
        assert!((ae - 100.0).abs() < 1.0, "atom economy: {ae:.2}%");
    }

    // ── to_string_equation ───────────────────────────────────────────────────

    #[test]
    fn test_to_string_equation_water_synthesis() {
        let rxn = Reaction::new(
            vec![
                ReactionComponent::new(make_h2(), 2),
                ReactionComponent::new(make_o2(), 1),
            ],
            vec![ReactionComponent::new(make_h2o(), 2)],
        );

        assert_eq!(rxn.to_string_equation(), "2H2 + O2 → 2H2O");
    }

    #[test]
    fn test_to_string_equation_methane_combustion() {
        let rxn = Reaction::new(
            vec![
                ReactionComponent::new(make_ch4(), 1),
                ReactionComponent::new(make_o2(), 2),
            ],
            vec![
                ReactionComponent::new(make_co2(), 1),
                ReactionComponent::new(make_h2o(), 2),
            ],
        );

        assert_eq!(rxn.to_string_equation(), "CH4 + 2O2 → CO2 + 2H2O");
    }

    #[test]
    fn test_to_string_equation_falls_back_to_formula() {
        // Molecule with no name — should use molecular_formula()
        let mut mol = Molecule::new();
        mol.add_atom(Atom::oxygen());
        mol.add_atom(Atom::hydrogen());
        mol.add_atom(Atom::hydrogen());
        // name is None → molecular_formula() = "H2O"

        let rxn = Reaction::new(vec![], vec![ReactionComponent::new(mol, 3)]);
        let eq = rxn.to_string_equation();
        assert!(eq.contains("3H2O"), "expected '3H2O' in '{eq}'");
    }

    // ── builder methods ──────────────────────────────────────────────────────

    #[test]
    fn test_reaction_builders() {
        let rxn = Reaction::new(vec![], vec![])
            .with_name("Water synthesis")
            .with_conditions("spark, 600°C");

        assert_eq!(rxn.name.as_deref(), Some("Water synthesis"));
        assert_eq!(rxn.conditions.as_deref(), Some("spark, 600°C"));
    }
}
