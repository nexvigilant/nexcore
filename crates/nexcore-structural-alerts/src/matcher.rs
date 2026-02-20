// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Substructure-based scanning engine for structural alerts.
//!
//! Uses the VF2 algorithm (via [`nexcore_molcore::substruct`]) to check each
//! alert pattern against a query molecule, collecting all matches together
//! with their non-overlapping occurrence counts.
//!
//! # Examples
//!
//! ```rust
//! use nexcore_structural_alerts::{AlertLibrary, scan_smiles, AlertCategory};
//!
//! let lib = AlertLibrary::default_library();
//! let matches = scan_smiles("c1ccccc1[N+](=O)[O-]", &lib)
//!     .unwrap_or_default();
//! let has_mutagen = matches.iter()
//!     .any(|m| m.alert.category == AlertCategory::Mutagenicity);
//! assert!(has_mutagen);
//! ```

use nexcore_molcore::graph::MolGraph;
use nexcore_molcore::smiles::parse;
use nexcore_molcore::substruct::{count_matches, has_substructure};

use crate::alert::AlertMatch;
use crate::error::{AlertError, AlertResult};
use crate::library::AlertLibrary;

/// Scan a pre-built [`MolGraph`] for all structural alerts in `library`.
///
/// For each alert in the library the alert's SMILES pattern is parsed into a
/// [`MolGraph`] and checked for substructure presence using VF2.  When a
/// match is found the non-overlapping occurrence count is recorded.
///
/// Returns `Ok(Vec<AlertMatch>)` — empty when no alerts fire — or
/// `Err(AlertError::InvalidPattern)` if a built-in or custom pattern cannot
/// be parsed.
///
/// # Errors
///
/// Returns [`AlertError::InvalidPattern`] if any alert's `smiles_pattern`
/// fails SMILES parsing.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
/// use nexcore_structural_alerts::{AlertLibrary, scan};
///
/// let mol = MolGraph::from_molecule(parse("c1ccccc1N").unwrap_or_default());
/// let lib = AlertLibrary::default_library();
/// let matches = scan(&mol, &lib).unwrap_or_default();
/// assert!(!matches.is_empty());
/// ```
pub fn scan(molecule: &MolGraph, library: &AlertLibrary) -> AlertResult<Vec<AlertMatch>> {
    let mut matches = Vec::new();

    for alert in library.alerts() {
        let pattern_mol = parse(&alert.smiles_pattern).map_err(|e| {
            AlertError::InvalidPattern(format!("{}: {e}", alert.id))
        })?;
        let pattern_graph = MolGraph::from_molecule(pattern_mol);

        if has_substructure(molecule, &pattern_graph) {
            let match_count = count_matches(molecule, &pattern_graph);
            matches.push(AlertMatch {
                alert: alert.clone(),
                match_count,
            });
        }
    }

    Ok(matches)
}

/// Scan a SMILES string for all structural alerts in `library`.
///
/// Parses `smiles` into a molecule, builds a [`MolGraph`], then delegates to
/// [`scan`].  This is the primary convenience entry point.
///
/// # Errors
///
/// Returns [`AlertError::SmilesParse`] if `smiles` is not valid SMILES.
/// Returns [`AlertError::InvalidPattern`] if any alert pattern is invalid.
///
/// # Examples
///
/// ```rust
/// use nexcore_structural_alerts::{AlertLibrary, scan_smiles, AlertCategory};
///
/// let lib = AlertLibrary::default_library();
/// let matches = scan_smiles("c1ccccc1N", &lib).unwrap_or_default();
/// assert!(!matches.is_empty());
/// ```
pub fn scan_smiles(smiles: &str, library: &AlertLibrary) -> AlertResult<Vec<AlertMatch>> {
    let mol = parse(smiles)
        .map_err(|e| AlertError::SmilesParse(e.to_string()))?;
    let graph = MolGraph::from_molecule(mol);
    scan(&graph, library)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use crate::alert::{AlertCategory, AlertSource, StructuralAlert};
    use crate::library::AlertLibrary;

    /// Build a [`MolGraph`] from a SMILES string, falling back to an empty
    /// molecule on parse failure (mirrors behaviour required by no-panic rule).
    fn graph_for(smiles: &str) -> MolGraph {
        MolGraph::from_molecule(parse(smiles).unwrap_or_default())
    }

    #[test]
    fn test_library_not_empty() {
        let lib = AlertLibrary::default_library();
        assert!(lib.len() >= 10, "library should have at least 10 alerts");
    }

    #[test]
    fn test_benzene_no_mutagenicity() {
        let lib = AlertLibrary::default_library();
        let mol = graph_for("c1ccccc1");
        let matches = scan(&mol, &lib).unwrap_or_default();
        let has_mutagen = matches
            .iter()
            .any(|m| m.alert.category == AlertCategory::Mutagenicity);
        assert!(
            !has_mutagen,
            "pure benzene should not trigger mutagenicity alerts"
        );
    }

    #[test]
    fn test_nitrobenzene_mutagenicity_alert() {
        let lib = AlertLibrary::default_library();
        let mol = graph_for("c1ccccc1[N+](=O)[O-]");
        let matches = scan(&mol, &lib).unwrap_or_default();
        let has_mutagen = matches
            .iter()
            .any(|m| m.alert.category == AlertCategory::Mutagenicity);
        assert!(has_mutagen, "nitrobenzene should trigger a mutagenicity alert");
    }

    #[test]
    fn test_aniline_alert() {
        let lib = AlertLibrary::default_library();
        let matches = scan_smiles("c1ccccc1N", &lib).unwrap_or_default();
        assert!(!matches.is_empty(), "aniline should trigger at least one alert");
    }

    #[test]
    fn test_ethanol_no_mutagenicity() {
        let lib = AlertLibrary::default_library();
        let matches = scan_smiles("CCO", &lib).unwrap_or_default();
        let mutagen_count = matches
            .iter()
            .filter(|m| m.alert.category == AlertCategory::Mutagenicity)
            .count();
        assert_eq!(mutagen_count, 0, "ethanol has no mutagenicity alerts");
    }

    #[test]
    fn test_scan_smiles_invalid_returns_err() {
        let lib = AlertLibrary::default_library();
        let result = scan_smiles("INVALID$$", &lib);
        assert!(result.is_err(), "invalid SMILES must return Err");
    }

    #[test]
    fn test_custom_alert_matches() {
        let mut lib = AlertLibrary::new();
        lib.add_alert(StructuralAlert {
            id: "CUSTOM-001".to_string(),
            name: "Test pattern".to_string(),
            smiles_pattern: "CC".to_string(),
            category: AlertCategory::General,
            source: AlertSource::Custom,
            confidence: 0.5,
            description: "Test".to_string(),
        });
        let matches = scan_smiles("CCCC", &lib).unwrap_or_default();
        assert!(
            !matches.is_empty(),
            "custom CC pattern should match CCCC"
        );
    }

    #[test]
    fn test_empty_library_no_matches() {
        let lib = AlertLibrary::new();
        let matches =
            scan_smiles("c1ccccc1[N+](=O)[O-]", &lib).unwrap_or_default();
        assert!(matches.is_empty(), "empty library must produce no matches");
    }

    #[test]
    fn test_alerts_by_category_mutagenicity() {
        let lib = AlertLibrary::default_library();
        let mutagen = lib.alerts_by_category(&AlertCategory::Mutagenicity);
        assert!(
            mutagen.len() >= 3,
            "should have at least 3 mutagenicity alerts, got {}",
            mutagen.len()
        );
    }

    #[test]
    fn test_epoxide_genotoxicity_alert() {
        let lib = AlertLibrary::default_library();
        // Ethylene oxide (simplest epoxide)
        let matches = scan_smiles("C1OC1", &lib).unwrap_or_default();
        let has_genotox = matches
            .iter()
            .any(|m| m.alert.category == AlertCategory::Genotoxicity);
        assert!(has_genotox, "epoxide should trigger a genotoxicity alert");
    }

    #[test]
    fn test_match_count_positive() {
        let lib = AlertLibrary::default_library();
        // Nitrobenzene should produce match_count >= 1 for each firing alert.
        let matches = scan_smiles("c1ccccc1[N+](=O)[O-]", &lib).unwrap_or_default();
        for m in &matches {
            assert!(
                m.match_count >= 1,
                "every alert match must have match_count >= 1, alert: {}",
                m.alert.id
            );
        }
    }

    #[test]
    fn test_scan_and_scan_smiles_agree() {
        let lib = AlertLibrary::default_library();
        let smiles = "c1ccccc1N";
        let via_graph = {
            let g = graph_for(smiles);
            scan(&g, &lib).unwrap_or_default()
        };
        let via_smiles = scan_smiles(smiles, &lib).unwrap_or_default();
        assert_eq!(
            via_graph.len(),
            via_smiles.len(),
            "scan and scan_smiles must return the same number of matches"
        );
    }
}
