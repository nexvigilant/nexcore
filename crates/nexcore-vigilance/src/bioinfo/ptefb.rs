//! P-TEFb complex components and regulators.
//!
//! P-TEFb (Positive Transcription Elongation Factor b) is a critical
//! regulator of RNA polymerase II transcription elongation. This module
//! provides constant definitions for P-TEFb complex genes and their roles.
//!
//! ## P-TEFb Complex Overview
//!
//! The core P-TEFb complex consists of:
//! - **CDK9**: Cyclin-dependent kinase 9 (catalytic subunit)
//! - **CCNT1/CCNT2**: Cyclin T1/T2 (regulatory subunits)
//!
//! Key regulators include:
//! - **HEXIM1/HEXIM2**: Inhibitors that sequester P-TEFb in 7SK snRNP
//! - **BRD4**: Bromodomain protein that recruits P-TEFb to promoters
//! - **Super Elongation Complex (SEC)**: AFF4, ELL2, ENL, AF9

use serde::{Deserialize, Serialize};

/// Role of a gene in the P-TEFb regulatory network.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum PtefbRole {
    /// Catalytic subunit of P-TEFb (CDK9).
    CatalyticSubunit,
    /// Cyclin T1 regulatory subunit.
    CyclinT1,
    /// Cyclin T2 regulatory subunit.
    CyclinT2,
    /// Inhibitor that sequesters P-TEFb.
    Inhibitor,
    /// Stabilizes 7SK snRNP complex.
    SevenSkStabilizer,
    /// Provides 7SK capping enzyme activity.
    SevenSkCapping,
    /// Activator that recruits P-TEFb.
    Activator,
    /// Component of Super Elongation Complex.
    SecComponent,
}

impl PtefbRole {
    /// Returns a human-readable description of this role.
    #[must_use]
    pub const fn description(&self) -> &'static str {
        match self {
            Self::CatalyticSubunit => "Catalytic subunit providing kinase activity",
            Self::CyclinT1 => "Cyclin T1 regulatory subunit",
            Self::CyclinT2 => "Cyclin T2 regulatory subunit",
            Self::Inhibitor => "Sequesters P-TEFb in inactive 7SK snRNP complex",
            Self::SevenSkStabilizer => "Stabilizes 7SK snRNA in the inhibitory complex",
            Self::SevenSkCapping => "Provides 7SK snRNA capping enzyme activity",
            Self::Activator => "Recruits P-TEFb to chromatin for activation",
            Self::SecComponent => "Component of Super Elongation Complex",
        }
    }
}

/// Information about a P-TEFb-related gene.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct PtefbGene {
    /// Gene symbol (e.g., "CDK9").
    pub symbol: String,
    /// KEGG gene ID (e.g., "hsa:1025").
    pub kegg_id: String,
    /// Role in P-TEFb regulation.
    pub role: PtefbRole,
}

impl PtefbGene {
    /// Creates from static strings (for use with constants).
    #[must_use]
    pub fn from_static(symbol: &str, kegg_id: &str, role: PtefbRole) -> Self {
        Self {
            symbol: symbol.to_string(),
            kegg_id: kegg_id.to_string(),
            role,
        }
    }
}

/// Static P-TEFb gene data for compile-time access.
pub struct PtefbGeneData {
    /// Gene symbol.
    pub symbol: &'static str,
    /// KEGG gene ID.
    pub kegg_id: &'static str,
    /// Role in P-TEFb regulation.
    pub role: PtefbRole,
}

impl PtefbGeneData {
    /// Converts to owned PtefbGene.
    #[must_use]
    pub fn to_gene(&self) -> PtefbGene {
        PtefbGene::from_static(self.symbol, self.kegg_id, self.role)
    }
}

// =============================================================================
// P-TEFb Gene Constants
// =============================================================================

/// CDK9 - Cyclin-dependent kinase 9 (catalytic subunit of P-TEFb).
pub const CDK9: PtefbGeneData = PtefbGeneData {
    symbol: "CDK9",
    kegg_id: "hsa:1025",
    role: PtefbRole::CatalyticSubunit,
};

/// CCNT1 - Cyclin T1 (regulatory subunit).
pub const CCNT1: PtefbGeneData = PtefbGeneData {
    symbol: "CCNT1",
    kegg_id: "hsa:904",
    role: PtefbRole::CyclinT1,
};

/// CCNT2 - Cyclin T2 (regulatory subunit).
pub const CCNT2: PtefbGeneData = PtefbGeneData {
    symbol: "CCNT2",
    kegg_id: "hsa:905",
    role: PtefbRole::CyclinT2,
};

/// HEXIM1 - HEXIM1 P-TEFb inhibitor.
pub const HEXIM1: PtefbGeneData = PtefbGeneData {
    symbol: "HEXIM1",
    kegg_id: "hsa:10614",
    role: PtefbRole::Inhibitor,
};

/// HEXIM2 - HEXIM2 P-TEFb inhibitor.
pub const HEXIM2: PtefbGeneData = PtefbGeneData {
    symbol: "HEXIM2",
    kegg_id: "hsa:124790",
    role: PtefbRole::Inhibitor,
};

/// LARP7 - La ribonucleoprotein 7 (7SK stabilizer).
pub const LARP7: PtefbGeneData = PtefbGeneData {
    symbol: "LARP7",
    kegg_id: "hsa:51574",
    role: PtefbRole::SevenSkStabilizer,
};

/// MEPCE - Methylphosphate capping enzyme (7SK capping).
pub const MEPCE: PtefbGeneData = PtefbGeneData {
    symbol: "MEPCE",
    kegg_id: "hsa:56257",
    role: PtefbRole::SevenSkCapping,
};

/// BRD4 - Bromodomain containing 4 (P-TEFb activator).
pub const BRD4: PtefbGeneData = PtefbGeneData {
    symbol: "BRD4",
    kegg_id: "hsa:23476",
    role: PtefbRole::Activator,
};

/// AFF4 - AF4/FMR2 family member 4 (SEC component).
pub const AFF4: PtefbGeneData = PtefbGeneData {
    symbol: "AFF4",
    kegg_id: "hsa:27125",
    role: PtefbRole::SecComponent,
};

/// ELL2 - Elongation factor for RNA polymerase II 2 (SEC component).
pub const ELL2: PtefbGeneData = PtefbGeneData {
    symbol: "ELL2",
    kegg_id: "hsa:22936",
    role: PtefbRole::SecComponent,
};

/// ENL/MLLT1 - MLLT1 super elongation complex subunit.
pub const ENL: PtefbGeneData = PtefbGeneData {
    symbol: "ENL",
    kegg_id: "hsa:4297",
    role: PtefbRole::SecComponent,
};

/// AF9/MLLT3 - MLLT3 super elongation complex subunit.
pub const AF9: PtefbGeneData = PtefbGeneData {
    symbol: "AF9",
    kegg_id: "hsa:4299",
    role: PtefbRole::SecComponent,
};

/// All P-TEFb genes as a static array.
pub const ALL_PTEFB_GENES: &[PtefbGeneData] = &[
    CDK9, CCNT1, CCNT2, HEXIM1, HEXIM2, LARP7, MEPCE, BRD4, AFF4, ELL2, ENL, AF9,
];

/// Returns all P-TEFb genes as owned values.
#[must_use]
pub fn all_ptefb_genes() -> Vec<PtefbGene> {
    ALL_PTEFB_GENES.iter().map(PtefbGeneData::to_gene).collect()
}

/// Looks up a P-TEFb gene by symbol.
#[must_use]
pub fn get_ptefb_gene(symbol: &str) -> Option<PtefbGene> {
    ALL_PTEFB_GENES
        .iter()
        .find(|g| g.symbol.eq_ignore_ascii_case(symbol))
        .map(PtefbGeneData::to_gene)
}

/// Returns all P-TEFb genes with a specific role.
#[must_use]
pub fn genes_by_role(role: PtefbRole) -> Vec<PtefbGene> {
    ALL_PTEFB_GENES
        .iter()
        .filter(|g| g.role == role)
        .map(PtefbGeneData::to_gene)
        .collect()
}

/// Returns the KEGG IDs for all P-TEFb genes.
#[must_use]
pub fn all_ptefb_kegg_ids() -> Vec<&'static str> {
    ALL_PTEFB_GENES.iter().map(|g| g.kegg_id).collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_all_ptefb_genes_count() {
        let genes = all_ptefb_genes();
        assert_eq!(genes.len(), 12);
    }

    #[test]
    fn test_get_ptefb_gene() {
        let gene = get_ptefb_gene("CDK9");
        assert!(gene.is_some());

        if let Some(g) = gene {
            assert_eq!(g.symbol, "CDK9");
            assert_eq!(g.kegg_id, "hsa:1025");
            assert_eq!(g.role, PtefbRole::CatalyticSubunit);
        }
    }

    #[test]
    fn test_get_ptefb_gene_case_insensitive() {
        let gene = get_ptefb_gene("cdk9");
        assert!(gene.is_some());

        let gene2 = get_ptefb_gene("Hexim1");
        assert!(gene2.is_some());
    }

    #[test]
    fn test_get_ptefb_gene_not_found() {
        let gene = get_ptefb_gene("INVALID");
        assert!(gene.is_none());
    }

    #[test]
    fn test_genes_by_role_inhibitors() {
        let inhibitors = genes_by_role(PtefbRole::Inhibitor);
        assert_eq!(inhibitors.len(), 2);

        let symbols: Vec<&str> = inhibitors.iter().map(|g| g.symbol.as_str()).collect();
        assert!(symbols.contains(&"HEXIM1"));
        assert!(symbols.contains(&"HEXIM2"));
    }

    #[test]
    fn test_genes_by_role_sec_components() {
        let sec = genes_by_role(PtefbRole::SecComponent);
        assert_eq!(sec.len(), 4);
    }

    #[test]
    fn test_all_ptefb_kegg_ids() {
        let ids = all_ptefb_kegg_ids();
        assert_eq!(ids.len(), 12);
        assert!(ids.contains(&"hsa:1025")); // CDK9
        assert!(ids.contains(&"hsa:10614")); // HEXIM1
    }

    #[test]
    fn test_ptefb_role_description() {
        assert!(CDK9.role.description().contains("kinase"));
        assert!(HEXIM1.role.description().contains("Sequesters"));
        assert!(BRD4.role.description().contains("Recruits"));
    }
}
