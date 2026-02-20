// Copyright © 2026 NexVigilant LLC. All Rights Reserved.

//! Applicability domain assessment using descriptor bounding-box approach.
//!
//! The training domain is defined by drug-like chemical space boundaries
//! derived from the Lipinski / Veber / Ertl rule-sets:
//!
//! | Descriptor | Lower | Upper |
//! |------------|-------|-------|
//! | MW (Da)    | 100   | 800   |
//! | LogP       | −3    | 8     |
//! | TPSA (Å²)  | 0     | 250   |
//! | HBA        | 0     | 15    |
//! | HBD        | 0     | 8     |
//!
//! A compound that violates **zero** boundaries is `InDomain`, exactly
//! **one** boundary gives `Borderline`, and **two or more** gives
//! `OutOfDomain`.

use nexcore_molcore::descriptor::Descriptors;

use crate::types::DomainStatus;

/// Assess whether a compound is within the applicability domain.
///
/// # Examples
///
/// ```rust
/// use nexcore_molcore::descriptor::calculate_descriptors;
/// use nexcore_molcore::graph::MolGraph;
/// use nexcore_molcore::smiles::parse;
/// use nexcore_qsar::applicability::assess_domain;
/// use nexcore_qsar::types::DomainStatus;
///
/// let mol = parse("CC(=O)Oc1ccccc1C(=O)O").unwrap_or_default();
/// let g = MolGraph::from_molecule(mol);
/// let d = calculate_descriptors(&g);
/// let status = assess_domain(&d);
/// assert!(matches!(status, DomainStatus::InDomain { .. }));
/// ```
#[must_use]
pub fn assess_domain(descriptors: &Descriptors) -> DomainStatus {
    let mut violations: Vec<&str> = Vec::new();

    if descriptors.molecular_weight < 100.0 || descriptors.molecular_weight > 800.0 {
        violations.push("MW outside 100–800 Da");
    }
    if descriptors.logp < -3.0 || descriptors.logp > 8.0 {
        violations.push("LogP outside −3 to 8");
    }
    if descriptors.tpsa > 250.0 {
        violations.push("TPSA > 250 Å²");
    }
    if descriptors.hba > 15 {
        violations.push("HBA > 15");
    }
    if descriptors.hbd > 8 {
        violations.push("HBD > 8");
    }

    match violations.len() {
        0 => DomainStatus::InDomain { confidence: 0.90 },
        1 => DomainStatus::Borderline {
            confidence: 0.60,
            warning: violations.join("; "),
        },
        _ => DomainStatus::OutOfDomain {
            distance: violations.len() as f64,
            warning: violations.join("; "),
        },
    }
}
