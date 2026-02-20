// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Core alert types — categories, sources, definitions, and match results.

use serde::{Deserialize, Serialize};

/// Category of toxicological concern associated with a structural alert.
///
/// # Examples
///
/// ```rust
/// use nexcore_structural_alerts::AlertCategory;
///
/// let cat = AlertCategory::Mutagenicity;
/// assert_eq!(format!("{cat:?}"), "Mutagenicity");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertCategory {
    /// DNA-damaging potential causing heritable mutations.
    Mutagenicity,
    /// Long-term risk of malignant tumour formation.
    Carcinogenicity,
    /// Liver toxicity via reactive metabolite formation.
    Hepatotoxicity,
    /// Cardiac toxicity, including hERG channel liability.
    Cardiotoxicity,
    /// Genotoxic potential not necessarily mutagenic.
    Genotoxicity,
    /// General structural concern without a specific organ target.
    General,
}

/// Provenance of a structural alert definition.
///
/// # Examples
///
/// ```rust
/// use nexcore_structural_alerts::AlertSource;
///
/// let src = AlertSource::IchM7;
/// assert_eq!(format!("{src:?}"), "IchM7");
/// ```
#[derive(Debug, Clone, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum AlertSource {
    /// ICH M7 guideline on mutagenic impurities.
    IchM7,
    /// Derek Nexus knowledge base.
    DerekNexus,
    /// User-defined custom alert.
    Custom,
    /// Retrocasting / retrosynthetic alert derivation.
    Retrocasting,
}

/// A structural alert definition comprising a SMILES pattern and metadata.
///
/// # Examples
///
/// ```rust
/// use nexcore_structural_alerts::{AlertCategory, AlertSource, StructuralAlert};
///
/// let alert = StructuralAlert {
///     id: "M7-001".to_string(),
///     name: "Aromatic amine".to_string(),
///     smiles_pattern: "c1ccccc1N".to_string(),
///     category: AlertCategory::Mutagenicity,
///     source: AlertSource::IchM7,
///     confidence: 0.95,
///     description: "Primary aromatic amines are potential mutagens.".to_string(),
/// };
/// assert_eq!(alert.id, "M7-001");
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StructuralAlert {
    /// Unique identifier for the alert (e.g. `"M7-001"`).
    pub id: String,
    /// Human-readable name.
    pub name: String,
    /// SMILES string used as the substructure query pattern.
    pub smiles_pattern: String,
    /// Toxicological category.
    pub category: AlertCategory,
    /// Knowledge-base provenance.
    pub source: AlertSource,
    /// Confidence score in the range `[0.0, 1.0]`.
    pub confidence: f64,
    /// Free-text description of the toxicophore.
    pub description: String,
}

/// The result of matching a single [`StructuralAlert`] against a molecule.
///
/// # Examples
///
/// ```rust
/// use nexcore_structural_alerts::{AlertCategory, AlertSource, AlertMatch, StructuralAlert};
///
/// let alert = StructuralAlert {
///     id: "CUSTOM-001".to_string(),
///     name: "Test".to_string(),
///     smiles_pattern: "CC".to_string(),
///     category: AlertCategory::General,
///     source: AlertSource::Custom,
///     confidence: 0.5,
///     description: "Test pattern".to_string(),
/// };
/// let m = AlertMatch { alert, match_count: 2 };
/// assert_eq!(m.match_count, 2);
/// ```
#[derive(Debug, Clone)]
pub struct AlertMatch {
    /// The alert whose pattern was found.
    pub alert: StructuralAlert,
    /// Number of non-overlapping occurrences of the pattern in the molecule.
    pub match_count: usize,
}
