//! Cross-domain transfer matrix: pairwise confidence between all registered taxonomies.
//!
//! Computes an N×N matrix where each cell (A→B) contains:
//! - Transfer count and average confidence from A targeting B
//! - Shared primitive count and tier-alignment
//! - Strongest bridge primitive
//!
//! Also extracts top cross-domain bridges (primitives appearing in 2+ taxonomies).

use std::collections::HashMap;

use serde::{Deserialize, Serialize};

use crate::compare;
use crate::registry::TaxonomyRegistry;
use crate::taxonomy::Tier;

/// Known aliases mapping transfer target_domain strings to registry keys.
fn domain_aliases() -> HashMap<&'static str, &'static str> {
    let mut m = HashMap::new();
    m.insert("missile-defense", "golden-dome");
    m.insert("medicine", "pharmacovigilance");
    // Direct matches (no alias needed, but explicit for documentation)
    m.insert("cybersecurity", "cybersecurity");
    m.insert("pharmacovigilance", "pharmacovigilance");
    m.insert("golden-dome", "golden-dome");
    m
}

/// Resolve a transfer target_domain string to a registry key.
fn resolve_target(target: &str) -> &str {
    let aliases = domain_aliases();
    let normalized = target.to_lowercase().replace(' ', "-");
    // Check aliases first, then fall back to normalized form
    if let Some(&key) = aliases.get(normalized.as_str()) {
        return key;
    }
    // Leak is avoided by returning the input if no alias found
    target
}

/// N×N transfer matrix between all registered taxonomies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TransferMatrix {
    /// Domain names in display order.
    pub domains: Vec<String>,
    /// Pairwise cells. Length = domains.len() * (domains.len() - 1) (no self-pairs).
    pub cells: Vec<MatrixCell>,
    /// Top bridges: primitives appearing in multiple taxonomies with high transfer utility.
    pub bridges: Vec<Bridge>,
}

/// One directed cell in the transfer matrix (from → to).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MatrixCell {
    pub from: String,
    pub to: String,
    /// Average confidence of transfers from `from` targeting `to`.
    pub avg_confidence: f64,
    /// Number of transfers from `from` targeting `to`.
    pub transfer_count: usize,
    /// Primitives shared by name between the two taxonomies.
    pub shared_count: usize,
    /// Shared primitives at the same tier in both.
    pub tier_aligned_count: usize,
    /// Highest-confidence transferable primitive name.
    pub strongest_primitive: Option<String>,
    /// Confidence of the strongest primitive.
    pub strongest_confidence: f64,
}

/// A cross-domain bridge: primitive appearing in 2+ registered taxonomies.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Bridge {
    /// Primitive name.
    pub name: String,
    /// Tier in the first taxonomy where it was found.
    pub tier: Tier,
    /// Registry keys of taxonomies containing this primitive.
    pub appears_in: Vec<String>,
    /// Average transfer confidence across all transfers involving this primitive.
    pub avg_confidence: f64,
    /// Whether tier classification is identical across all appearances.
    pub tier_consistent: bool,
}

/// Compute the full transfer matrix from all registered taxonomies.
pub fn compute(registry: &TaxonomyRegistry) -> TransferMatrix {
    let names = registry.list();
    let mut domains: Vec<String> = names.into_iter().map(|s| s.to_string()).collect();
    domains.sort();

    let mut cells = Vec::new();
    let mut all_shared: HashMap<String, Vec<(String, Tier)>> = HashMap::new();

    for from_name in &domains {
        let from_tax = match registry.get(from_name) {
            Some(t) => t,
            None => continue,
        };

        for to_name in &domains {
            if from_name == to_name {
                continue;
            }
            let to_tax = match registry.get(to_name) {
                Some(t) => t,
                None => continue,
            };

            // Find transfers from `from` that target `to`
            let to_key = to_name.to_lowercase().replace(' ', "-");
            let matching_transfers: Vec<_> = from_tax
                .transfers
                .iter()
                .filter(|t| {
                    let resolved = resolve_target(&t.target_domain);
                    resolved == to_key
                })
                .collect();

            let transfer_count = matching_transfers.len();
            let avg_confidence = if transfer_count > 0 {
                matching_transfers
                    .iter()
                    .map(|t| t.confidence())
                    .sum::<f64>()
                    / transfer_count as f64
            } else {
                0.0
            };

            // Find strongest
            let (strongest_primitive, strongest_confidence) = matching_transfers
                .iter()
                .max_by(|a, b| {
                    a.confidence()
                        .partial_cmp(&b.confidence())
                        .unwrap_or(std::cmp::Ordering::Equal)
                })
                .map(|t| (Some(t.primitive_name.clone()), t.confidence()))
                .unwrap_or((None, 0.0));

            // Shared primitives via compare
            let cmp = compare::compare(from_tax, to_tax);
            let tier_aligned = compare::tier_aligned(&cmp);

            // Collect shared primitives for bridge computation
            for sp in &cmp.shared {
                all_shared
                    .entry(sp.name.clone())
                    .or_default()
                    .push((from_name.clone(), sp.tier_a));
                all_shared
                    .entry(sp.name.clone())
                    .or_default()
                    .push((to_name.clone(), sp.tier_b));
            }

            cells.push(MatrixCell {
                from: from_name.clone(),
                to: to_name.clone(),
                avg_confidence,
                transfer_count,
                shared_count: cmp.shared.len(),
                tier_aligned_count: tier_aligned.len(),
                strongest_primitive,
                strongest_confidence,
            });
        }
    }

    // Compute bridges: primitives appearing in 2+ domains
    let bridges = compute_bridges(registry, &domains, &all_shared);

    TransferMatrix {
        domains,
        cells,
        bridges,
    }
}

/// Extract bridge primitives from shared primitive data.
fn compute_bridges(
    registry: &TaxonomyRegistry,
    domains: &[String],
    all_shared: &HashMap<String, Vec<(String, Tier)>>,
) -> Vec<Bridge> {
    let mut bridges = Vec::new();

    for (prim_name, appearances) in all_shared {
        // Deduplicate domains
        let mut domain_set: Vec<String> = Vec::new();
        let mut tiers: Vec<Tier> = Vec::new();
        for (domain, tier) in appearances {
            if !domain_set.contains(domain) {
                domain_set.push(domain.clone());
                tiers.push(*tier);
            }
        }

        if domain_set.len() < 2 {
            continue;
        }

        let tier = tiers.first().copied().unwrap_or(Tier::T1);
        let tier_consistent = tiers.iter().all(|&t| t == tier);

        // Average confidence from all transfers involving this primitive across all taxonomies
        let mut total_conf = 0.0;
        let mut conf_count = 0usize;
        for domain_name in domains {
            if let Some(tax) = registry.get(domain_name) {
                for t in &tax.transfers {
                    if t.primitive_name == *prim_name {
                        total_conf += t.confidence();
                        conf_count += 1;
                    }
                }
            }
        }

        let avg_confidence = if conf_count > 0 {
            total_conf / conf_count as f64
        } else {
            0.0
        };

        bridges.push(Bridge {
            name: prim_name.clone(),
            tier,
            appears_in: domain_set,
            avg_confidence,
            tier_consistent,
        });
    }

    // Sort by avg_confidence descending
    bridges.sort_by(|a, b| {
        b.avg_confidence
            .partial_cmp(&a.avg_confidence)
            .unwrap_or(std::cmp::Ordering::Equal)
    });

    bridges
}

/// Convenience: get a specific cell from the matrix.
pub fn get_cell<'a>(matrix: &'a TransferMatrix, from: &str, to: &str) -> Option<&'a MatrixCell> {
    let from_norm = from.to_lowercase().replace(' ', "-");
    let to_norm = to.to_lowercase().replace(' ', "-");
    matrix.cells.iter().find(|c| {
        c.from.to_lowercase().replace(' ', "-") == from_norm
            && c.to.to_lowercase().replace(' ', "-") == to_norm
    })
}

/// Get the top N bridges by confidence.
pub fn top_bridges(matrix: &TransferMatrix, n: usize) -> &[Bridge] {
    let len = matrix.bridges.len().min(n);
    &matrix.bridges[..len]
}

/// Symmetry check: how different is A→B vs B→A confidence?
pub fn asymmetry(matrix: &TransferMatrix, a: &str, b: &str) -> Option<f64> {
    let ab = get_cell(matrix, a, b)?;
    let ba = get_cell(matrix, b, a)?;
    Some((ab.avg_confidence - ba.avg_confidence).abs())
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn matrix_from_default_registry() {
        let reg = TaxonomyRegistry::new();
        let matrix = compute(&reg);
        // 3 domains → 3×2 = 6 directed cells
        assert_eq!(matrix.domains.len(), 3);
        assert_eq!(matrix.cells.len(), 6);
    }

    #[test]
    fn all_cells_have_transfers_or_shared() {
        let reg = TaxonomyRegistry::new();
        let matrix = compute(&reg);
        for cell in &matrix.cells {
            // Every pair of our 3 built-ins should have SOME shared primitives
            assert!(
                cell.shared_count > 0,
                "{} → {} has 0 shared primitives",
                cell.from,
                cell.to
            );
        }
    }

    #[test]
    fn golden_dome_to_cybersecurity_has_transfers() {
        let reg = TaxonomyRegistry::new();
        let matrix = compute(&reg);
        let cell = get_cell(&matrix, "Golden Dome", "Cybersecurity");
        assert!(cell.is_some(), "no cell for Golden Dome → Cybersecurity");
        let cell = cell.unwrap_or_else(|| &matrix.cells[0]);
        assert!(
            cell.transfer_count > 0,
            "Golden Dome should have transfers targeting cybersecurity"
        );
        assert!(cell.avg_confidence > 0.5);
    }

    #[test]
    fn pv_to_golden_dome_has_transfers() {
        let reg = TaxonomyRegistry::new();
        let matrix = compute(&reg);
        let cell = get_cell(&matrix, "Pharmacovigilance", "Golden Dome");
        assert!(cell.is_some(), "no cell for PV → Golden Dome");
        let cell = cell.unwrap_or_else(|| &matrix.cells[0]);
        // PV targets "missile-defense" which aliases to "golden-dome"
        assert!(
            cell.transfer_count > 0,
            "PV should have transfers targeting missile-defense (golden-dome)"
        );
    }

    #[test]
    fn cyber_to_pv_has_transfers() {
        let reg = TaxonomyRegistry::new();
        let matrix = compute(&reg);
        let cell = get_cell(&matrix, "Cybersecurity", "Pharmacovigilance");
        assert!(cell.is_some());
        let cell = cell.unwrap_or_else(|| &matrix.cells[0]);
        assert!(
            cell.transfer_count > 0,
            "Cyber should have transfers targeting pharmacovigilance"
        );
    }

    #[test]
    fn bridges_found() {
        let reg = TaxonomyRegistry::new();
        let matrix = compute(&reg);
        assert!(
            !matrix.bridges.is_empty(),
            "should find bridge primitives across domains"
        );
        // detection, threshold, tracking should be bridges (T1 in all 3)
        let bridge_names: Vec<&str> = matrix.bridges.iter().map(|b| b.name.as_str()).collect();
        assert!(
            bridge_names.contains(&"detection"),
            "detection should be a bridge"
        );
        assert!(
            bridge_names.contains(&"threshold"),
            "threshold should be a bridge"
        );
        assert!(
            bridge_names.contains(&"tracking"),
            "tracking should be a bridge"
        );
    }

    #[test]
    fn bridge_tier_consistency() {
        let reg = TaxonomyRegistry::new();
        let matrix = compute(&reg);
        // detection is T1 in all 3 domains → tier_consistent = true
        let detection = matrix.bridges.iter().find(|b| b.name == "detection");
        assert!(detection.is_some());
        let detection = detection.unwrap_or_else(|| &matrix.bridges[0]);
        assert!(detection.tier_consistent);
        assert_eq!(detection.tier, Tier::T1);
        assert!(detection.appears_in.len() >= 2);
    }

    #[test]
    fn top_bridges_limit() {
        let reg = TaxonomyRegistry::new();
        let matrix = compute(&reg);
        let top3 = top_bridges(&matrix, 3);
        assert!(top3.len() <= 3);
        // Should be sorted descending by confidence
        if top3.len() >= 2 {
            assert!(top3[0].avg_confidence >= top3[1].avg_confidence);
        }
    }

    #[test]
    fn asymmetry_check() {
        let reg = TaxonomyRegistry::new();
        let matrix = compute(&reg);
        let asym = asymmetry(&matrix, "Golden Dome", "Cybersecurity");
        assert!(asym.is_some());
        // Asymmetry should be a small non-negative number
        let asym = asym.unwrap_or(999.0);
        assert!(asym >= 0.0);
        assert!(asym < 1.0);
    }

    #[test]
    fn empty_registry_produces_empty_matrix() {
        let reg = TaxonomyRegistry::empty();
        let matrix = compute(&reg);
        assert!(matrix.domains.is_empty());
        assert!(matrix.cells.is_empty());
        assert!(matrix.bridges.is_empty());
    }

    #[test]
    fn self_pairs_excluded() {
        let reg = TaxonomyRegistry::new();
        let matrix = compute(&reg);
        for cell in &matrix.cells {
            assert_ne!(cell.from, cell.to, "matrix should not contain self-pairs");
        }
    }

    #[test]
    fn strongest_primitive_has_highest_confidence() {
        let reg = TaxonomyRegistry::new();
        let matrix = compute(&reg);
        for cell in &matrix.cells {
            if cell.transfer_count > 0 {
                assert!(
                    cell.strongest_primitive.is_some(),
                    "{} → {} has transfers but no strongest primitive",
                    cell.from,
                    cell.to
                );
                assert!(cell.strongest_confidence >= cell.avg_confidence);
            }
        }
    }

    #[test]
    fn resolve_alias_missile_defense() {
        assert_eq!(resolve_target("missile-defense"), "golden-dome");
    }

    #[test]
    fn resolve_alias_medicine() {
        assert_eq!(resolve_target("medicine"), "pharmacovigilance");
    }

    #[test]
    fn resolve_direct_match() {
        assert_eq!(resolve_target("cybersecurity"), "cybersecurity");
    }
}
