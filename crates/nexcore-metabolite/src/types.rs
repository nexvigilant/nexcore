// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Core data types for metabolite prediction.
//!
//! [`Transformation`] enumerates every reaction class the engine can identify.
//! [`Metabolite`] pairs a transformation with site metadata and a probability
//! estimate.  [`MetaboliteTree`] collects all predictions for one parent
//! molecule, partitioned by metabolic phase and degradation pathway.

use serde::{Deserialize, Serialize};

/// A metabolic or degradation transformation type.
///
/// Variants are grouped by metabolic phase:
/// - **Phase I** (CYP-mediated): [`Hydroxylation`], [`NDealkylation`],
///   [`ODealkylation`], [`Epoxidation`], [`Reduction`]
/// - **Phase II** (conjugation): [`Glucuronidation`], [`Sulfation`],
///   [`Acetylation`], [`GlutathioneConjugation`]
/// - **Degradation**: [`Hydrolysis`], [`Oxidation`], [`Photodegradation`]
///
/// [`Hydroxylation`]: Transformation::Hydroxylation
/// [`NDealkylation`]: Transformation::NDealkylation
/// [`ODealkylation`]: Transformation::ODealkylation
/// [`Epoxidation`]: Transformation::Epoxidation
/// [`Reduction`]: Transformation::Reduction
/// [`Glucuronidation`]: Transformation::Glucuronidation
/// [`Sulfation`]: Transformation::Sulfation
/// [`Acetylation`]: Transformation::Acetylation
/// [`GlutathioneConjugation`]: Transformation::GlutathioneConjugation
/// [`Hydrolysis`]: Transformation::Hydrolysis
/// [`Oxidation`]: Transformation::Oxidation
/// [`Photodegradation`]: Transformation::Photodegradation
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum Transformation {
    // ------------------------------------------------------------------
    // Phase I
    // ------------------------------------------------------------------
    /// Aromatic or aliphatic hydroxylation at atom `site`.
    Hydroxylation {
        /// Atom index in the parent molecule where hydroxylation occurs.
        site: usize,
    },
    /// N-dealkylation removing an alkyl group at atom `site`.
    NDealkylation {
        /// Atom index of the carbon being cleaved from nitrogen.
        site: usize,
    },
    /// O-dealkylation removing an alkyl group at atom `site`.
    ODealkylation {
        /// Atom index of the carbon being cleaved from oxygen.
        site: usize,
    },
    /// Aromatic epoxidation across the bond between `site1` and `site2`.
    Epoxidation {
        /// First atom index of the epoxidised double bond.
        site1: usize,
        /// Second atom index of the epoxidised double bond.
        site2: usize,
    },
    /// Reductive transformation at atom `site`.
    Reduction {
        /// Atom index undergoing reduction.
        site: usize,
    },
    // ------------------------------------------------------------------
    // Phase II
    // ------------------------------------------------------------------
    /// UDP-glucuronosyltransferase (UGT) conjugation at atom `site`.
    Glucuronidation {
        /// Atom index of the acceptor heteroatom.
        site: usize,
    },
    /// Sulfotransferase (SULT) conjugation at atom `site`.
    Sulfation {
        /// Atom index of the acceptor heteroatom.
        site: usize,
    },
    /// N-acetyltransferase (NAT) conjugation at atom `site`.
    Acetylation {
        /// Atom index of the amine nitrogen being acetylated.
        site: usize,
    },
    /// Glutathione-S-transferase (GST) conjugation at atom `site`.
    GlutathioneConjugation {
        /// Atom index at the conjugation site.
        site: usize,
    },
    // ------------------------------------------------------------------
    // Degradation
    // ------------------------------------------------------------------
    /// Hydrolytic cleavage of an ester or amide bond at atom `site`.
    Hydrolysis {
        /// Atom index of the bond-bridging heteroatom (O or N).
        site: usize,
    },
    /// Chemical oxidation of a labile position at atom `site`.
    Oxidation {
        /// Atom index undergoing oxidation.
        site: usize,
    },
    /// Photodegradation at atom `site`.
    Photodegradation {
        /// Atom index susceptible to photolytic cleavage.
        site: usize,
    },
}

/// A single predicted metabolite or degradant.
///
/// # Examples
///
/// ```rust
/// use nexcore_metabolite::types::{Metabolite, Transformation};
///
/// let m = Metabolite {
///     transformation: Transformation::Hydroxylation { site: 0 },
///     site_description: "Aromatic C at position 0".to_string(),
///     probability: 0.4,
///     reactive_intermediate: false,
///     enzyme: Some("CYP1A2".to_string()),
/// };
/// assert!(!m.reactive_intermediate);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Metabolite {
    /// The reaction class and site index.
    pub transformation: Transformation,
    /// Human-readable description of the susceptible site.
    pub site_description: String,
    /// Estimated probability that this transformation occurs (0.0–1.0).
    pub probability: f64,
    /// `true` if the metabolite is a reactive intermediate with genotoxic
    /// or electrophilic risk (e.g. arene oxides, quinones).
    pub reactive_intermediate: bool,
    /// Enzyme primarily responsible (e.g. `"CYP3A4"`, `"UGT"`), if known.
    pub enzyme: Option<String>,
}

/// Complete metabolite prediction tree for one parent molecule.
///
/// Produced by [`predict_metabolites`] or [`predict_from_smiles`].
///
/// [`predict_metabolites`]: crate::predict::predict_metabolites
/// [`predict_from_smiles`]: crate::predict::predict_from_smiles
///
/// # Examples
///
/// ```rust
/// use nexcore_metabolite::predict_from_smiles;
///
/// let tree = predict_from_smiles("CCO").unwrap_or_default();
/// assert_eq!(tree.parent_smiles, "CCO");
/// ```
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct MetaboliteTree {
    /// SMILES of the parent (input) molecule.
    pub parent_smiles: String,
    /// Phase I (CYP-mediated) metabolites.
    pub phase1: Vec<Metabolite>,
    /// Phase II (conjugation) metabolites.
    pub phase2: Vec<Metabolite>,
    /// Phase I reactive intermediates flagged for safety review.
    pub reactive_intermediates: Vec<Metabolite>,
    /// Predicted degradation products.
    pub degradants: Vec<Metabolite>,
}
