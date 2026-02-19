//! Display implementations for taxonomy types — compact table format.

use std::fmt;

use crate::analysis::Bottleneck;
use crate::compare::TaxonomyComparison;
use crate::taxonomy::{DomainTaxonomy, Primitive, Tier};
use crate::transfer::TransferScore;

impl fmt::Display for Primitive {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "[{}] {} — {}",
            self.tier.label(),
            self.name,
            self.definition
        )
    }
}

impl fmt::Display for TransferScore {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "S={:.2} F={:.2} C={:.2} → {:.3}",
            self.structural,
            self.functional,
            self.contextual,
            self.confidence()
        )
    }
}

impl fmt::Display for DomainTaxonomy {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let counts = self.tier_counts();
        writeln!(f, "═══ {} ═══", self.name)?;
        writeln!(f, "{}", self.description)?;
        writeln!(f)?;
        writeln!(
            f,
            "Primitives: {} (T1:{} T2-P:{} T2-C:{} T3:{})",
            self.primitives.len(),
            counts.get(&Tier::T1).copied().unwrap_or(0),
            counts.get(&Tier::T2P).copied().unwrap_or(0),
            counts.get(&Tier::T2C).copied().unwrap_or(0),
            counts.get(&Tier::T3).copied().unwrap_or(0),
        )?;
        writeln!(f, "Irreducible atoms: {}", self.irreducible_atoms().len())?;
        writeln!(f, "Transfers: {}", self.transfers.len())?;

        for tier in [Tier::T1, Tier::T2P, Tier::T2C, Tier::T3] {
            let prims = self.by_tier(tier);
            if !prims.is_empty() {
                writeln!(f)?;
                writeln!(f, "── {} ({}) ──", tier.label(), prims.len())?;
                for p in prims {
                    writeln!(f, "  {}: {}", p.name, p.definition)?;
                    if !p.dependencies.is_empty() {
                        writeln!(f, "    ← [{}]", p.dependencies.join(", "))?;
                    }
                }
            }
        }
        Ok(())
    }
}

impl fmt::Display for Bottleneck {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{} [{}] fan-out={} reach={:.1}%",
            self.name,
            self.tier.label(),
            self.fan_out,
            self.reach_pct
        )
    }
}

impl fmt::Display for TaxonomyComparison {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(
            f,
            "═══ Comparison: {} × {} ═══",
            self.taxonomy_a, self.taxonomy_b
        )?;
        writeln!(
            f,
            "Shared: {} | Unique A: {} | Unique B: {} | Jaccard: {:.3}",
            self.shared.len(),
            self.unique_a.len(),
            self.unique_b.len(),
            self.jaccard
        )?;
        if !self.shared.is_empty() {
            writeln!(f, "\nShared primitives:")?;
            for s in &self.shared {
                let marker = if s.tier_match { "=" } else { "≠" };
                writeln!(
                    f,
                    "  {} [{}] {} [{}]",
                    s.name,
                    s.tier_a.label(),
                    marker,
                    s.tier_b.label()
                )?;
            }
        }
        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::golden_dome::golden_dome;

    #[test]
    fn taxonomy_display_renders() {
        let tax = golden_dome();
        let s = format!("{tax}");
        assert!(s.contains("Golden Dome"));
        assert!(s.contains("T1-Universal"));
        assert!(s.contains("detection"));
    }

    #[test]
    fn primitive_display() {
        let p = Primitive::new("detection", "Signal recognition", Tier::T1);
        let s = format!("{p}");
        assert!(s.contains("T1-Universal"));
        assert!(s.contains("detection"));
    }

    #[test]
    fn transfer_score_display() {
        let ts = TransferScore::new(0.95, 0.88, 0.75);
        let s = format!("{ts}");
        assert!(s.contains("0.882")); // confidence
    }
}
