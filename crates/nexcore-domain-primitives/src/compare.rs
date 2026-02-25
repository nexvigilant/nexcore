//! Cross-taxonomy comparison: find shared primitives between two domain extractions.

use serde::{Deserialize, Serialize};

use crate::taxonomy::{DomainTaxonomy, Tier};

/// Result of comparing two taxonomies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct TaxonomyComparison {
    /// Names of the two taxonomies compared.
    pub taxonomy_a: String,
    pub taxonomy_b: String,
    /// Primitives found in both (matched by name).
    pub shared: Vec<SharedPrimitive>,
    /// Primitives only in taxonomy A.
    pub unique_a: Vec<String>,
    /// Primitives only in taxonomy B.
    pub unique_b: Vec<String>,
    /// Jaccard similarity = |shared| / |union|.
    pub jaccard: f64,
}

/// A primitive found in both taxonomies.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
pub struct SharedPrimitive {
    pub name: String,
    pub tier_a: Tier,
    pub tier_b: Tier,
    /// Whether the tier classification agrees.
    pub tier_match: bool,
}

/// Compare two taxonomies and find shared/unique primitives.
pub fn compare(a: &DomainTaxonomy, b: &DomainTaxonomy) -> TaxonomyComparison {
    let names_a: std::collections::HashSet<&str> =
        a.primitives.iter().map(|p| p.name.as_str()).collect();
    let names_b: std::collections::HashSet<&str> =
        b.primitives.iter().map(|p| p.name.as_str()).collect();

    let shared_names: Vec<&str> = names_a.intersection(&names_b).copied().collect();

    let shared: Vec<SharedPrimitive> = shared_names
        .iter()
        .filter_map(|name| {
            let pa = a.get(name)?;
            let pb = b.get(name)?;
            Some(SharedPrimitive {
                name: name.to_string(),
                tier_a: pa.tier,
                tier_b: pb.tier,
                tier_match: pa.tier == pb.tier,
            })
        })
        .collect();

    let unique_a: Vec<String> = names_a
        .difference(&names_b)
        .map(|s| s.to_string())
        .collect();
    let unique_b: Vec<String> = names_b
        .difference(&names_a)
        .map(|s| s.to_string())
        .collect();

    let union_size = names_a.union(&names_b).count();
    let jaccard = if union_size > 0 {
        shared.len() as f64 / union_size as f64
    } else {
        0.0
    };

    TaxonomyComparison {
        taxonomy_a: a.name.clone(),
        taxonomy_b: b.name.clone(),
        shared,
        unique_a,
        unique_b,
        jaccard,
    }
}

/// Find primitives that appear at the same tier in both taxonomies (strongest cross-domain signal).
pub fn tier_aligned(comparison: &TaxonomyComparison) -> Vec<&SharedPrimitive> {
    comparison.shared.iter().filter(|s| s.tier_match).collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::golden_dome::golden_dome;
    use crate::taxonomy::{DomainTaxonomy, Primitive, Tier};

    fn cyber_taxonomy() -> DomainTaxonomy {
        let mut tax = DomainTaxonomy::new("Cybersecurity", "Network defense primitives");
        // Shared with Golden Dome
        tax.primitives.push(Primitive::new(
            "detection",
            "Anomaly recognition in network traffic",
            Tier::T1,
        ));
        tax.primitives.push(Primitive::new(
            "defense-in-depth",
            "Layered security controls",
            Tier::T2P,
        ));
        tax.primitives.push(Primitive::new(
            "tracking",
            "Session and threat tracking",
            Tier::T1,
        ));
        // Unique to cyber
        tax.primitives.push(Primitive::new(
            "zero-trust",
            "Never trust, always verify",
            Tier::T2P,
        ));
        tax.primitives.push(Primitive::new(
            "sandboxing",
            "Isolated execution environment",
            Tier::T2C,
        ));
        tax
    }

    #[test]
    fn compare_golden_dome_with_cyber() {
        let gd = golden_dome();
        let cyber = cyber_taxonomy();
        let cmp = compare(&gd, &cyber);

        assert_eq!(cmp.shared.len(), 3);
        assert!(cmp.unique_b.contains(&"zero-trust".to_string()));
        assert!(cmp.unique_b.contains(&"sandboxing".to_string()));
        assert!(cmp.jaccard > 0.0);
        assert!(cmp.jaccard < 1.0);
    }

    #[test]
    fn tier_aligned_primitives() {
        let gd = golden_dome();
        let cyber = cyber_taxonomy();
        let cmp = compare(&gd, &cyber);
        let aligned = tier_aligned(&cmp);

        // detection (T1/T1), tracking (T1/T1), defense-in-depth (T2P/T2P) — all match
        assert_eq!(aligned.len(), 3);
        for a in &aligned {
            assert!(a.tier_match);
        }
    }

    #[test]
    fn identical_taxonomy() {
        let gd1 = golden_dome();
        let gd2 = golden_dome();
        let cmp = compare(&gd1, &gd2);
        assert!((cmp.jaccard - 1.0).abs() < 1e-10);
        assert!(cmp.unique_a.is_empty());
        assert!(cmp.unique_b.is_empty());
    }

    #[test]
    fn disjoint_taxonomies() {
        let a = DomainTaxonomy::new("A", "test");
        let b = DomainTaxonomy::new("B", "test");
        let cmp = compare(&a, &b);
        assert!((cmp.jaccard).abs() < 1e-10);
    }
}
