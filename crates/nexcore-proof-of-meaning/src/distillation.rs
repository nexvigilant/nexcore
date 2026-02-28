//! Distillation — decompose expressions by semantic volatility.
//!
//! Chemistry: Separate a mixture by differential boiling points.
//! Semantics: Separate a compound expression by context-dependence.

use ordered_float::OrderedFloat;
use serde::{Deserialize, Serialize};

use crate::element::Atom;

/// A single distillation fraction — one atom separated from the mixture.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Fraction {
    /// The separated atom.
    pub atom: Atom,
    /// What "temperature" was needed to separate this.
    /// Lower = more volatile = more context-dependent.
    pub separation_temperature: OrderedFloat<f64>,
    /// Purity of the fraction — was this a clean separation?
    /// 1.0 = pure. < 1.0 = co-eluted with other atoms.
    pub purity: OrderedFloat<f64>,
    /// Which step in the distillation sequence this was.
    pub fraction_number: usize,
}

/// Complete distillation result — an auditable proof trail.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DistillationResult {
    /// The original expression that was distilled.
    pub input_expression: String,
    /// Ordered fractions (most volatile first).
    pub fractions: Vec<Fraction>,
    /// Residue — anything that couldn't be separated.
    pub residue: Vec<UnseparableResidue>,
    /// Mass balance: does input mass = sum of fraction masses + residue?
    pub mass_balance: MassBalance,
}

/// Material that couldn't be separated into constituent atoms.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnseparableResidue {
    pub fragment: String,
    pub reason: ResidueReason,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum ResidueReason {
    /// Two atoms that can't be separated (identical volatility).
    Azeotrope { co_eluting_atoms: Vec<String> },
    /// Irreducible idiom — this fragment IS an atom, not a compound.
    IrreducibleIdiom,
    /// Unknown fragment — not in the registry at all.
    UnknownFragment,
}

/// Mass balance check — conservation of semantic content.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MassBalance {
    pub input_mass: f64,
    pub recovered_mass: f64,
    pub loss_percent: f64,
    pub verdict: MassBalanceVerdict,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum MassBalanceVerdict {
    /// Loss < 2% — excellent separation.
    Quantitative,
    /// Loss 2-10% — acceptable, minor meaning loss.
    Acceptable,
    /// Loss > 10% — significant meaning was lost.
    SignificantLoss,
}

/// The distillation engine.
pub struct Distiller {
    /// Minimum purity threshold.
    pub min_purity: f64,
    /// Maximum fractions before aborting.
    pub max_fractions: usize,
}

impl Distiller {
    pub fn new() -> Self {
        Self {
            min_purity: 0.85,
            max_fractions: 20,
        }
    }

    /// Distill a regulatory text expression into constituent atoms.
    ///
    /// Every intermediate step is recorded. The record IS the proof.
    pub fn distill(&self, expression: &str) -> DistillationResult {
        let tokens: Vec<&str> = expression.split_whitespace().collect();
        let mut fractions = Vec::new();
        let mut recovered_mass = 0.0;
        let input_mass = tokens.len() as f64;

        for (i, token) in tokens.iter().enumerate() {
            let volatility = estimate_volatility(token);

            // Resolve class via synonym registry (instead of defaulting to Modifier)
            let registry = crate::synonymy::SynonymRegistry::pv_standard();
            let class = registry
                .resolve(&token.to_lowercase())
                .map(|(_, c, _)| c)
                .unwrap_or(crate::element::ElementClass::Modifier);

            let atom = Atom::new(token, class, volatility);

            let mass = 1.0; // simplified — real mass comes from vector norm
            recovered_mass += mass;

            fractions.push(Fraction {
                atom,
                separation_temperature: OrderedFloat(1.0 - volatility),
                purity: OrderedFloat(0.95),
                fraction_number: i + 1,
            });
        }

        // Sort by volatility (most volatile first = lowest separation temp)
        fractions.sort_by(|a, b| a.separation_temperature.cmp(&b.separation_temperature));

        // Renumber after sorting
        for (i, f) in fractions.iter_mut().enumerate() {
            f.fraction_number = i + 1;
        }

        let loss_percent = if input_mass > 0.0 {
            ((input_mass - recovered_mass) / input_mass).abs() * 100.0
        } else {
            0.0
        };

        DistillationResult {
            input_expression: expression.to_string(),
            fractions,
            residue: Vec::new(),
            mass_balance: MassBalance {
                input_mass,
                recovered_mass,
                loss_percent,
                verdict: if loss_percent < 2.0 {
                    MassBalanceVerdict::Quantitative
                } else if loss_percent < 10.0 {
                    MassBalanceVerdict::Acceptable
                } else {
                    MassBalanceVerdict::SignificantLoss
                },
            },
        }
    }
}

impl Default for Distiller {
    fn default() -> Self {
        Self::new()
    }
}

/// Estimate semantic volatility of a token.
/// In production, this comes from corpus analysis and registry lookup.
fn estimate_volatility(token: &str) -> f64 {
    match token.to_lowercase().as_str() {
        // Very stable — context-independent
        "drug" | "patient" | "dose" | "treatment" => 0.05,
        // Stable in medical contexts
        "cardiac" | "hepatic" | "renal" | "neurological" => 0.15,
        // Moderately stable
        "reaction" | "event" | "effect" | "finding" => 0.25,
        "adverse" | "related" | "associated" => 0.35,
        // Context-dependent
        "serious" | "severe" | "significant" => 0.55,
        // Highly context-dependent
        "unexpected" | "unlisted" | "novel" => 0.80,
        // Unknown — treat as moderately volatile
        _ => 0.50,
    }
}
