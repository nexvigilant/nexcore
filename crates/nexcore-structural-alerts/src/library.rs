// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Built-in ICH M7 alert library and library management.
//!
//! The [`AlertLibrary`] holds a collection of [`StructuralAlert`] definitions.
//! Use [`AlertLibrary::default_library`] to obtain the full built-in set, or
//! [`AlertLibrary::new`] for an empty library that you populate with
//! [`AlertLibrary::add_alert`].
//!
//! # Examples
//!
//! ```rust
//! use nexcore_structural_alerts::{AlertCategory, AlertLibrary};
//!
//! let lib = AlertLibrary::default_library();
//! assert!(lib.len() >= 10);
//! let mutagens = lib.alerts_by_category(&AlertCategory::Mutagenicity);
//! assert!(!mutagens.is_empty());
//! ```

use crate::alert::{AlertCategory, AlertSource, StructuralAlert};

/// A collection of [`StructuralAlert`] definitions used as the query library
/// for substructure scanning.
#[derive(Debug, Clone, Default)]
pub struct AlertLibrary {
    alerts: Vec<StructuralAlert>,
}

impl AlertLibrary {
    /// Create an empty library with no alerts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_structural_alerts::AlertLibrary;
    ///
    /// let lib = AlertLibrary::new();
    /// assert!(lib.is_empty());
    /// ```
    #[must_use]
    pub fn new() -> Self {
        Self { alerts: Vec::new() }
    }

    /// Create the default library containing all built-in ICH M7 alerts.
    ///
    /// Returns a library pre-populated with ~15 structural alerts covering
    /// mutagenicity, carcinogenicity, genotoxicity, hepatotoxicity, and
    /// general concerns.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_structural_alerts::AlertLibrary;
    ///
    /// let lib = AlertLibrary::default_library();
    /// assert!(lib.len() >= 10);
    /// ```
    #[must_use]
    pub fn default_library() -> Self {
        let alerts = builtin_alerts();
        Self { alerts }
    }

    /// Add a custom alert to the library.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_structural_alerts::{AlertCategory, AlertSource, AlertLibrary, StructuralAlert};
    ///
    /// let mut lib = AlertLibrary::new();
    /// lib.add_alert(StructuralAlert {
    ///     id: "CUSTOM-001".to_string(),
    ///     name: "Test".to_string(),
    ///     smiles_pattern: "CC".to_string(),
    ///     category: AlertCategory::General,
    ///     source: AlertSource::Custom,
    ///     confidence: 0.5,
    ///     description: "Ethane fragment".to_string(),
    /// });
    /// assert_eq!(lib.len(), 1);
    /// ```
    pub fn add_alert(&mut self, alert: StructuralAlert) {
        self.alerts.push(alert);
    }

    /// Return a slice of all alerts in the library.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_structural_alerts::AlertLibrary;
    ///
    /// let lib = AlertLibrary::default_library();
    /// assert!(!lib.alerts().is_empty());
    /// ```
    #[must_use]
    pub fn alerts(&self) -> &[StructuralAlert] {
        &self.alerts
    }

    /// Return all alerts belonging to a given [`AlertCategory`].
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_structural_alerts::{AlertCategory, AlertLibrary};
    ///
    /// let lib = AlertLibrary::default_library();
    /// let hits = lib.alerts_by_category(&AlertCategory::Mutagenicity);
    /// assert!(hits.len() >= 3);
    /// ```
    #[must_use]
    pub fn alerts_by_category(&self, category: &AlertCategory) -> Vec<&StructuralAlert> {
        self.alerts
            .iter()
            .filter(|a| &a.category == category)
            .collect()
    }

    /// Return the total number of alerts in the library.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_structural_alerts::AlertLibrary;
    ///
    /// let lib = AlertLibrary::new();
    /// assert_eq!(lib.len(), 0);
    /// ```
    #[must_use]
    pub fn len(&self) -> usize {
        self.alerts.len()
    }

    /// Return `true` if the library contains no alerts.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use nexcore_structural_alerts::AlertLibrary;
    ///
    /// let lib = AlertLibrary::new();
    /// assert!(lib.is_empty());
    /// ```
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.alerts.is_empty()
    }
}

// ---------------------------------------------------------------------------
// Built-in alert definitions
// ---------------------------------------------------------------------------

/// Construct the built-in ICH M7 alert set.
///
/// SMILES patterns are simplified substructure queries; the VF2 matcher
/// handles element and bond-order semantics.
fn builtin_alerts() -> Vec<StructuralAlert> {
    vec![
        StructuralAlert {
            id: "M7-001".to_string(),
            name: "Aromatic amine".to_string(),
            smiles_pattern: "c1ccccc1N".to_string(),
            category: AlertCategory::Mutagenicity,
            source: AlertSource::IchM7,
            confidence: 0.92,
            description: "Primary aromatic amines can form reactive arylnitrenium ions that \
                 alkylate DNA."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-002".to_string(),
            name: "Nitro group".to_string(),
            smiles_pattern: "[N+](=O)[O-]".to_string(),
            category: AlertCategory::Mutagenicity,
            source: AlertSource::IchM7,
            confidence: 0.88,
            description: "Nitro groups are bioreductively activated to reactive hydroxylamines \
                 and nitroso intermediates."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-003".to_string(),
            name: "Aldehyde".to_string(),
            smiles_pattern: "C=O".to_string(),
            category: AlertCategory::Genotoxicity,
            source: AlertSource::IchM7,
            confidence: 0.70,
            description: "Aliphatic aldehydes form Schiff bases with DNA bases and may \
                 cross-link proteins."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-004".to_string(),
            name: "Epoxide".to_string(),
            smiles_pattern: "C1OC1".to_string(),
            category: AlertCategory::Genotoxicity,
            source: AlertSource::IchM7,
            confidence: 0.85,
            description: "Epoxides are electrophilic and alkylate nucleophilic sites on DNA."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-005".to_string(),
            name: "Alkyl chloride".to_string(),
            smiles_pattern: "CCl".to_string(),
            category: AlertCategory::Mutagenicity,
            source: AlertSource::IchM7,
            confidence: 0.78,
            description: "Alkyl chlorides are SN2-reactive electrophiles capable of alkylating \
                 DNA."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-006".to_string(),
            name: "Alkyl bromide".to_string(),
            smiles_pattern: "CBr".to_string(),
            category: AlertCategory::Mutagenicity,
            source: AlertSource::IchM7,
            confidence: 0.80,
            description: "Alkyl bromides are potent SN2 alkylating agents with higher reactivity \
                 than chlorides."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-007".to_string(),
            name: "N-nitroso compound".to_string(),
            smiles_pattern: "NN=O".to_string(),
            category: AlertCategory::Carcinogenicity,
            source: AlertSource::IchM7,
            confidence: 0.93,
            description: "N-nitrosamines are bioactivated by CYP450 to diazonium ions that \
                 alkylate DNA — a major class of carcinogens."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-008".to_string(),
            name: "Acyl chloride".to_string(),
            smiles_pattern: "C(=O)Cl".to_string(),
            category: AlertCategory::General,
            source: AlertSource::IchM7,
            confidence: 0.82,
            description: "Acyl chlorides are highly reactive acylating agents that modify \
                 proteins and nucleic acids."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-009".to_string(),
            name: "Michael acceptor (enone)".to_string(),
            smiles_pattern: "C=CC=O".to_string(),
            category: AlertCategory::Genotoxicity,
            source: AlertSource::IchM7,
            confidence: 0.75,
            description: "Alpha,beta-unsaturated carbonyls undergo Michael addition with \
                 nucleophilic thiol and amine groups on DNA."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-010".to_string(),
            name: "Hydrazine".to_string(),
            smiles_pattern: "NN".to_string(),
            category: AlertCategory::Mutagenicity,
            source: AlertSource::IchM7,
            confidence: 0.80,
            description: "Hydrazines are oxidatively activated to diazenes and reactive \
                 radical species that damage DNA."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-011".to_string(),
            name: "Nitroaromatic".to_string(),
            smiles_pattern: "c1ccccc1[N+](=O)[O-]".to_string(),
            category: AlertCategory::Mutagenicity,
            source: AlertSource::IchM7,
            confidence: 0.91,
            description: "Nitroaromatic compounds combine the mutagenic liability of both the \
                 aromatic ring and the nitro group."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-012".to_string(),
            name: "Aromatic N-oxide".to_string(),
            smiles_pattern: "c1ccncc1".to_string(),
            category: AlertCategory::General,
            source: AlertSource::IchM7,
            confidence: 0.65,
            description: "Pyridine-type aromatic N-oxides can generate reactive intermediates \
                 under metabolic reduction."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-013".to_string(),
            name: "Phenol".to_string(),
            smiles_pattern: "c1ccccc1O".to_string(),
            category: AlertCategory::Hepatotoxicity,
            source: AlertSource::IchM7,
            confidence: 0.60,
            description: "Phenols are oxidised to quinones and semi-quinone radicals that \
                 covalently bind hepatic proteins."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-014".to_string(),
            name: "Aniline (primary aromatic amine)".to_string(),
            smiles_pattern: "c1ccccc1N".to_string(),
            category: AlertCategory::Mutagenicity,
            source: AlertSource::DerekNexus,
            confidence: 0.94,
            description: "Aniline and its derivatives are N-hydroxylated to electrophilic \
                 arylhydroxylamines."
                .to_string(),
        },
        StructuralAlert {
            id: "M7-015".to_string(),
            name: "Quinone".to_string(),
            smiles_pattern: "O=C1C=CC(=O)C=C1".to_string(),
            category: AlertCategory::Genotoxicity,
            source: AlertSource::IchM7,
            confidence: 0.83,
            description: "Quinones are bi-electrophilic Michael acceptors and redox-cyclers \
                 that generate reactive oxygen species and strand breaks."
                .to_string(),
        },
    ]
}
