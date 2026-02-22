//! Antibody / immunoglobulin topology analysis for 3D biologics visualization.
//!
//! Identifies IgG chain types (heavy/light), classifies immunoglobulin domains
//! (VH, VL, CH1-CH3, CL), locates CDR loops via Kabat numbering, and maps
//! Fab/Fc fragment boundaries for structural rendering.
//!
//! Primitive formula: antibody = μ(chain_map) × ∂(domain_boundaries) + σ(CDR_loops)
//!
//! # Example
//!
//! ```rust
//! use nexcore_viz::antibody::{analyze_antibody, IgChainType, IgDomain};
//! use nexcore_viz::molecular::Molecule;
//!
//! let mol = Molecule::new("IgG1-test");
//! let topology = analyze_antibody(&mol);
//! // Empty molecule has no chains, so no chain types
//! assert!(topology.chain_types.is_empty());
//! ```

use serde::{Deserialize, Serialize};

use crate::molecular::{Atom, Chain, Element, Molecule, Residue};

// ============================================================================
// Error type
// ============================================================================

/// Errors produced during antibody topology analysis.
#[derive(Debug, Clone)]
pub enum AntibodyError {
    /// A requested chain was not found in the molecule.
    ChainNotFound(char),
    /// Atom index out of bounds.
    AtomIndexOutOfBounds(usize),
}

impl std::fmt::Display for AntibodyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ChainNotFound(id) => write!(f, "chain '{id}' not found in molecule"),
            Self::AtomIndexOutOfBounds(idx) => {
                write!(f, "atom index {idx} is out of bounds")
            }
        }
    }
}

impl std::error::Error for AntibodyError {}

// ============================================================================
// IgChainType
// ============================================================================

/// Immunoglobulin chain classification.
///
/// A canonical IgG antibody has two heavy chains and two light chains.
/// Light chains are further classified by their constant-region isotype.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IgChainType {
    /// Heavy chain (~450 residues): VH-CH1-Hinge-CH2-CH3.
    Heavy,
    /// Light kappa chain (~220 residues) — the more common human isotype (~60 %).
    LightKappa,
    /// Light lambda chain (~220 residues) — human isotype (~40 %).
    LightLambda,
    /// Chain could not be classified from the available sequence/length data.
    Unknown,
}

// ============================================================================
// IgDomain
// ============================================================================

/// Immunoglobulin domain classification within a single chain.
///
/// Domain boundaries follow approximate Kabat numbering conventions.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum IgDomain {
    /// Variable heavy domain (Kabat residues 1–113).
    VH,
    /// Variable light domain (Kabat residues 1–107).
    VL,
    /// Heavy constant domain 1 (residues 114–223).
    CH1,
    /// Heavy constant domain 2 (residues 244–360).
    CH2,
    /// Heavy constant domain 3 (residues 361–476).
    CH3,
    /// Light constant domain (residues 108–214).
    CL,
    /// Hinge region connecting CH1 to CH2 (residues 224–243).
    Hinge,
    /// Domain could not be classified.
    Unknown,
}

// ============================================================================
// CdrId
// ============================================================================

/// Identifier for one of the six complementarity-determining regions (CDRs).
///
/// Three reside in the heavy chain (H1–H3) and three in the light chain (L1–L3).
/// Together they form the antigen-binding paratope.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CdrId {
    /// Heavy-chain CDR 1 (Kabat 26–35, simplified).
    H1,
    /// Heavy-chain CDR 2 (Kabat 50–65).
    H2,
    /// Heavy-chain CDR 3 (Kabat 95–102) — highest diversity, antigen contact.
    H3,
    /// Light-chain CDR 1 (Kabat 24–34).
    L1,
    /// Light-chain CDR 2 (Kabat 50–56).
    L2,
    /// Light-chain CDR 3 (Kabat 89–97).
    L3,
}

impl CdrId {
    /// Human-readable label (e.g. `"CDR-H1"`).
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::H1 => "CDR-H1",
            Self::H2 => "CDR-H2",
            Self::H3 => "CDR-H3",
            Self::L1 => "CDR-L1",
            Self::L2 => "CDR-L2",
            Self::L3 => "CDR-L3",
        }
    }
}

// ============================================================================
// CdrLoop
// ============================================================================

/// A single complementarity-determining region (CDR) loop.
///
/// Contains chain identity, Kabat residue boundaries, length, and the
/// one-letter amino-acid sequence extracted from the residue names.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CdrLoop {
    /// Which CDR this is (H1/H2/H3/L1/L2/L3).
    pub id: CdrId,
    /// Chain identifier this CDR belongs to.
    pub chain_id: char,
    /// First residue sequence number (Kabat-approximate).
    pub start_residue: i32,
    /// Last residue sequence number (Kabat-approximate).
    pub end_residue: i32,
    /// Number of residues in the loop.
    pub length: usize,
    /// One-letter amino-acid sequence of the CDR residues.
    pub sequence: String,
}

// ============================================================================
// DomainRegion
// ============================================================================

/// A classified immunoglobulin domain region within one protein chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainRegion {
    /// Domain classification.
    pub domain: IgDomain,
    /// Chain this region belongs to.
    pub chain_id: char,
    /// First residue sequence number.
    pub start_residue: i32,
    /// Last residue sequence number.
    pub end_residue: i32,
    /// Number of residues in this domain.
    pub residue_count: usize,
}

// ============================================================================
// FragmentType
// ============================================================================

/// Antibody fragment classification for structural / functional rendering.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum FragmentType {
    /// Antigen-binding fragment: VH+CH1 (heavy) + VL+CL (light).
    Fab,
    /// Crystallisable fragment: CH2+CH3 from both heavy chains.
    Fc,
    /// Hinge region connecting Fab arms to Fc.
    Hinge,
}

// ============================================================================
// AntibodyFragment
// ============================================================================

/// One discrete fragment of an antibody (Fab, Fc, or Hinge).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntibodyFragment {
    /// Fragment type.
    pub fragment_type: FragmentType,
    /// Chain identifiers that make up this fragment.
    pub chains: Vec<char>,
    /// Domain regions contained within this fragment.
    pub domain_regions: Vec<DomainRegion>,
}

// ============================================================================
// AntibodyTopology
// ============================================================================

/// Complete antibody topology analysis result.
///
/// Produced by [`analyze_antibody`] and contains all structural annotations
/// needed to drive fragment-coloured ribbon or surface rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AntibodyTopology {
    /// Chain identifiers paired with their classified type.
    pub chain_types: Vec<(char, IgChainType)>,
    /// All domain regions across all chains.
    pub domains: Vec<DomainRegion>,
    /// All CDR loops found across heavy and light chains.
    pub cdrs: Vec<CdrLoop>,
    /// Fab/Fc/Hinge fragments assembled from domains.
    pub fragments: Vec<AntibodyFragment>,
    /// Pairs of atom indices forming disulfide bonds (S–S distance ≤ threshold).
    pub disulfide_bonds: Vec<(usize, usize)>,
}

// ============================================================================
// Helper: three-letter → one-letter amino acid code
// ============================================================================

/// Convert a three-letter residue name to its one-letter code.
///
/// Returns `'X'` for unrecognised names, matching IUPAC convention.
#[must_use]
fn three_to_one(name: &str) -> char {
    match name {
        "ALA" => 'A',
        "ARG" => 'R',
        "ASN" => 'N',
        "ASP" => 'D',
        "CYS" => 'C',
        "GLN" => 'Q',
        "GLU" => 'E',
        "GLY" => 'G',
        "HIS" => 'H',
        "ILE" => 'I',
        "LEU" => 'L',
        "LYS" => 'K',
        "MET" => 'M',
        "PHE" => 'F',
        "PRO" => 'P',
        "SER" => 'S',
        "THR" => 'T',
        "TRP" => 'W',
        "TYR" => 'Y',
        "VAL" => 'V',
        _ => 'X',
    }
}

// ============================================================================
// classify_chain
// ============================================================================

/// Classify a protein chain as heavy, kappa light, lambda light, or unknown.
///
/// Classification strategy (heuristic, no sequence alignment needed):
///
/// 1. Chain length drives the primary split:
///    - `> 300` residues → Heavy (heavy chains are ~450 residues).
///    - `≤ 300` residues → tentatively Light; sub-classify as kappa or lambda.
///
/// 2. For light chains the last residue of the constant region carries a
///    characteristic fingerprint:
///    - Kappa ends in `…Thr-Val-Ser-Ser` — the final constant residue is
///      typically **CYS** (Kabat 214) followed by conserved Kappa motif.
///    - Lambda ends in similar length but with **SER** at the equivalent
///      position.
///    - When sequence information is ambiguous, a 50/50 split at residue
///      count 215 is used: `≤ 215` → Kappa, else Lambda.
///
/// In practice, crystallographic structures rarely require more than this
/// heuristic to correctly assign the two light-chain isotypes for rendering.
#[must_use]
pub fn classify_chain(chain: &Chain, _atoms: &[Atom]) -> IgChainType {
    let residue_count = chain.residues.len();

    if residue_count == 0 {
        return IgChainType::Unknown;
    }

    // Primary split: heavy vs light based on chain length.
    if residue_count > 300 {
        return IgChainType::Heavy;
    }

    // Light-chain sub-classification.
    // Look for characteristic residues near the C-terminus of the constant
    // domain (roughly positions 205–214 in Kabat numbering).
    // Kappa has CYS at position 214; lambda is defined by the absence.
    let has_c_terminal_cys = chain.residues.iter().rev().take(20).any(|r| r.name == "CYS");

    // Additionally, check for a kappa-marker residue name pattern: kappa
    // light chains typically have THR near the very C-terminus.
    let has_c_terminal_thr = chain.residues.iter().rev().take(10).any(|r| r.name == "THR");

    // Lambda light chains carry a SER at their penultimate constant position.
    let has_c_terminal_ser = chain.residues.iter().rev().take(10).any(|r| r.name == "SER");

    if has_c_terminal_cys && has_c_terminal_thr && !has_c_terminal_ser {
        IgChainType::LightKappa
    } else if has_c_terminal_ser {
        IgChainType::LightLambda
    } else {
        // Fallback: use residue count parity.
        if residue_count <= 215 {
            IgChainType::LightKappa
        } else {
            IgChainType::LightLambda
        }
    }
}

// ============================================================================
// identify_domains
// ============================================================================

/// Partition a chain into its constituent immunoglobulin domains.
///
/// Kabat-approximate boundary table used:
///
/// | Chain  | Domain | Start | End |
/// |--------|--------|-------|-----|
/// | Heavy  | VH     | 1     | 113 |
/// | Heavy  | CH1    | 114   | 223 |
/// | Heavy  | Hinge  | 224   | 243 |
/// | Heavy  | CH2    | 244   | 360 |
/// | Heavy  | CH3    | 361   | 476 |
/// | Light  | VL     | 1     | 107 |
/// | Light  | CL     | 108   | 214 |
///
/// Residues that fall outside all defined windows are assigned [`IgDomain::Unknown`].
#[must_use]
pub fn identify_domains(chain: &Chain, chain_type: IgChainType) -> Vec<DomainRegion> {
    // Define (domain, start_seq, end_seq) windows for each chain type.
    let windows: &[(IgDomain, i32, i32)] = match chain_type {
        IgChainType::Heavy => &[
            (IgDomain::VH, 1, 113),
            (IgDomain::CH1, 114, 223),
            (IgDomain::Hinge, 224, 243),
            (IgDomain::CH2, 244, 360),
            (IgDomain::CH3, 361, 476),
        ],
        IgChainType::LightKappa | IgChainType::LightLambda => &[
            (IgDomain::VL, 1, 107),
            (IgDomain::CL, 108, 214),
        ],
        IgChainType::Unknown => &[],
    };

    windows
        .iter()
        .filter_map(|&(domain, start, end)| {
            // Collect residues whose seq falls within [start, end].
            let matching: Vec<&Residue> = chain
                .residues
                .iter()
                .filter(|r| r.seq >= start && r.seq <= end)
                .collect();

            if matching.is_empty() {
                return None;
            }

            let actual_start = matching.first().map(|r| r.seq).unwrap_or(start);
            let actual_end = matching.last().map(|r| r.seq).unwrap_or(end);

            Some(DomainRegion {
                domain,
                chain_id: chain.id,
                start_residue: actual_start,
                end_residue: actual_end,
                residue_count: matching.len(),
            })
        })
        .collect()
}

// ============================================================================
// locate_cdrs_kabat
// ============================================================================

/// Locate CDR loops using simplified Kabat residue-number boundaries.
///
/// CDR definitions (Kabat simplified):
///
/// | CDR  | Chain  | Residues |
/// |------|--------|----------|
/// | H1   | Heavy  | 26–35    |
/// | H2   | Heavy  | 50–65    |
/// | H3   | Heavy  | 95–102   |
/// | L1   | Light  | 24–34    |
/// | L2   | Light  | 50–56    |
/// | L3   | Light  | 89–97    |
///
/// Each CDR is represented as a [`CdrLoop`] with the one-letter sequence
/// extracted from the residue names present in the chain.
#[must_use]
pub fn locate_cdrs_kabat(
    chain: &Chain,
    chain_type: IgChainType,
    _atoms: &[Atom],
) -> Vec<CdrLoop> {
    // (CdrId, start_seq, end_seq)
    let cdr_windows: &[(CdrId, i32, i32)] = match chain_type {
        IgChainType::Heavy => &[
            (CdrId::H1, 26, 35),
            (CdrId::H2, 50, 65),
            (CdrId::H3, 95, 102),
        ],
        IgChainType::LightKappa | IgChainType::LightLambda => &[
            (CdrId::L1, 24, 34),
            (CdrId::L2, 50, 56),
            (CdrId::L3, 89, 97),
        ],
        IgChainType::Unknown => &[],
    };

    cdr_windows
        .iter()
        .filter_map(|&(cdr_id, start, end)| {
            let residues_in_cdr: Vec<&Residue> = chain
                .residues
                .iter()
                .filter(|r| r.seq >= start && r.seq <= end)
                .collect();

            if residues_in_cdr.is_empty() {
                return None;
            }

            let actual_start = residues_in_cdr.first().map(|r| r.seq).unwrap_or(start);
            let actual_end = residues_in_cdr.last().map(|r| r.seq).unwrap_or(end);
            let sequence: String = residues_in_cdr
                .iter()
                .map(|r| three_to_one(&r.name))
                .collect();

            Some(CdrLoop {
                id: cdr_id,
                chain_id: chain.id,
                start_residue: actual_start,
                end_residue: actual_end,
                length: residues_in_cdr.len(),
                sequence,
            })
        })
        .collect()
}

// ============================================================================
// detect_disulfide_bonds
// ============================================================================

/// Detect disulfide bonds by finding pairs of sulfur (SG) atoms within
/// `max_distance` angstroms of each other.
///
/// A canonical disulfide bond spans ~2.05 Å (S–S bond length). Passing
/// `max_distance = 2.2` captures all crystallographic S–S bonds while
/// excluding van-der-Waals contacts (~3.6 Å) between non-bonded sulfurs.
///
/// Only atoms named `"SG"` on `CYS` residues are considered as candidates,
/// matching the standard PDB atom naming convention.
///
/// Returns a `Vec` of `(atom_index_a, atom_index_b)` pairs where both indices
/// are 0-based positions into `mol.atoms`.
#[must_use]
pub fn detect_disulfide_bonds(mol: &Molecule, max_distance: f64) -> Vec<(usize, usize)> {
    // Collect indices of all sulfur SG atoms on CYS residues.
    let sg_indices: Vec<usize> = mol
        .atoms
        .iter()
        .enumerate()
        .filter(|(_, atom)| {
            atom.element == Element::S
                && atom.name == "SG"
                && atom
                    .residue_name
                    .as_deref()
                    .map(|n| n == "CYS")
                    .unwrap_or(false)
        })
        .map(|(idx, _)| idx)
        .collect();

    let mut bonds = Vec::new();

    // O(n²) over the SG set — this is tiny (≤ ~20 cysteines per antibody).
    for i in 0..sg_indices.len() {
        for j in (i + 1)..sg_indices.len() {
            let idx_a = sg_indices[i];
            let idx_b = sg_indices[j];

            // Both indices are valid by construction (collected from enumeration).
            if let (Some(atom_a), Some(atom_b)) =
                (mol.atoms.get(idx_a), mol.atoms.get(idx_b))
            {
                if atom_a.distance_to(atom_b) <= max_distance {
                    bonds.push((idx_a, idx_b));
                }
            }
        }
    }

    bonds
}

// ============================================================================
// map_fragments
// ============================================================================

/// Assemble Fab, Fc, and Hinge fragments from classified chains and domains.
///
/// Fragment assembly rules:
///
/// - **Fab**: one heavy chain (VH + CH1) paired with one light chain (VL + CL).
///   In a symmetric IgG there are two Fabs; each gets its own entry.
/// - **Fc**: CH2 + CH3 domains from both heavy chains combined.
/// - **Hinge**: Hinge domains from heavy chains.
///
/// Pairing strategy: heavy chains are matched in order with light chains in
/// order, so chain H with chain L, then chain H' with chain L' (if present).
#[must_use]
pub fn map_fragments(
    chain_types: &[(char, IgChainType)],
    domains: &[DomainRegion],
) -> Vec<AntibodyFragment> {
    let heavy_chains: Vec<char> = chain_types
        .iter()
        .filter(|(_, t)| *t == IgChainType::Heavy)
        .map(|&(id, _)| id)
        .collect();

    let light_chains: Vec<char> = chain_types
        .iter()
        .filter(|(_, t)| matches!(t, IgChainType::LightKappa | IgChainType::LightLambda))
        .map(|&(id, _)| id)
        .collect();

    let mut fragments = Vec::new();

    // Build Fab fragments — one per heavy/light pair.
    let pair_count = heavy_chains.len().min(light_chains.len());
    for pair_idx in 0..pair_count {
        let heavy_id = heavy_chains[pair_idx];
        let light_id = light_chains[pair_idx];

        let fab_heavy_domains: Vec<DomainRegion> = domains
            .iter()
            .filter(|d| {
                d.chain_id == heavy_id
                    && matches!(d.domain, IgDomain::VH | IgDomain::CH1)
            })
            .cloned()
            .collect();

        let fab_light_domains: Vec<DomainRegion> = domains
            .iter()
            .filter(|d| {
                d.chain_id == light_id
                    && matches!(d.domain, IgDomain::VL | IgDomain::CL)
            })
            .cloned()
            .collect();

        let mut fab_domains = fab_heavy_domains;
        fab_domains.extend(fab_light_domains);

        if !fab_domains.is_empty() {
            fragments.push(AntibodyFragment {
                fragment_type: FragmentType::Fab,
                chains: vec![heavy_id, light_id],
                domain_regions: fab_domains,
            });
        }
    }

    // Build Hinge fragment — hinge regions from all heavy chains.
    let hinge_domains: Vec<DomainRegion> = domains
        .iter()
        .filter(|d| {
            heavy_chains.contains(&d.chain_id) && d.domain == IgDomain::Hinge
        })
        .cloned()
        .collect();

    if !hinge_domains.is_empty() {
        fragments.push(AntibodyFragment {
            fragment_type: FragmentType::Hinge,
            chains: heavy_chains.clone(),
            domain_regions: hinge_domains,
        });
    }

    // Build Fc fragment — CH2 + CH3 from all heavy chains.
    let fc_domains: Vec<DomainRegion> = domains
        .iter()
        .filter(|d| {
            heavy_chains.contains(&d.chain_id)
                && matches!(d.domain, IgDomain::CH2 | IgDomain::CH3)
        })
        .cloned()
        .collect();

    if !fc_domains.is_empty() {
        fragments.push(AntibodyFragment {
            fragment_type: FragmentType::Fc,
            chains: heavy_chains,
            domain_regions: fc_domains,
        });
    }

    fragments
}

// ============================================================================
// analyze_antibody — public orchestrator
// ============================================================================

/// Perform complete antibody topology analysis on a [`Molecule`].
///
/// This orchestrates all sub-analyses:
///
/// 1. Classify each chain as Heavy / LightKappa / LightLambda / Unknown.
/// 2. Identify immunoglobulin domain regions within each chain.
/// 3. Locate CDR loops using Kabat numbering.
/// 4. Detect disulfide bonds (S–S pairs within 2.2 Å).
/// 5. Map Fab / Hinge / Fc fragments.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::antibody::analyze_antibody;
/// use nexcore_viz::molecular::Molecule;
///
/// let mol = Molecule::new("empty");
/// let topo = analyze_antibody(&mol);
/// assert!(topo.chain_types.is_empty());
/// assert!(topo.cdrs.is_empty());
/// ```
#[must_use]
pub fn analyze_antibody(mol: &Molecule) -> AntibodyTopology {
    // Step 1: classify chains.
    let chain_types: Vec<(char, IgChainType)> = mol
        .chains
        .iter()
        .map(|chain| {
            let chain_type = classify_chain(chain, &mol.atoms);
            (chain.id, chain_type)
        })
        .collect();

    // Step 2: identify domains for each chain.
    let domains: Vec<DomainRegion> = mol
        .chains
        .iter()
        .flat_map(|chain| {
            let chain_type = chain_types
                .iter()
                .find(|(id, _)| *id == chain.id)
                .map(|&(_, t)| t)
                .unwrap_or(IgChainType::Unknown);
            identify_domains(chain, chain_type)
        })
        .collect();

    // Step 3: locate CDR loops.
    let cdrs: Vec<CdrLoop> = mol
        .chains
        .iter()
        .flat_map(|chain| {
            let chain_type = chain_types
                .iter()
                .find(|(id, _)| *id == chain.id)
                .map(|&(_, t)| t)
                .unwrap_or(IgChainType::Unknown);
            locate_cdrs_kabat(chain, chain_type, &mol.atoms)
        })
        .collect();

    // Step 4: detect disulfide bonds.
    let disulfide_bonds = detect_disulfide_bonds(mol, 2.2);

    // Step 5: map fragments.
    let fragments = map_fragments(&chain_types, &domains);

    AntibodyTopology {
        chain_types,
        domains,
        cdrs,
        fragments,
        disulfide_bonds,
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{Atom, Chain, Element, Molecule, Residue, SecondaryStructure};

    // -------------------------------------------------------------------------
    // Test helpers
    // -------------------------------------------------------------------------

    /// Build a synthetic atom at the given position with SG / CYS annotations.
    fn cys_sg_atom(id: u32, position: [f64; 3], chain_id: char, seq: i32) -> Atom {
        Atom {
            id,
            element: Element::S,
            position,
            charge: 0,
            name: "SG".to_string(),
            residue_name: Some("CYS".to_string()),
            residue_seq: Some(seq),
            chain_id: Some(chain_id),
            b_factor: Some(10.0),
        }
    }

    /// Build a plain CA atom for backbone representation.
    fn ca_atom(id: u32, position: [f64; 3], residue_name: &str, chain_id: char, seq: i32) -> Atom {
        Atom {
            id,
            element: Element::C,
            position,
            charge: 0,
            name: "CA".to_string(),
            residue_name: Some(residue_name.to_string()),
            residue_seq: Some(seq),
            chain_id: Some(chain_id),
            b_factor: Some(15.0),
        }
    }

    /// Create a [`Residue`] whose atom indices point into a molecule atom list.
    fn make_residue(name: &str, seq: i32, atom_indices: Vec<usize>) -> Residue {
        Residue {
            name: name.to_string(),
            seq,
            insertion_code: None,
            atom_indices,
            secondary_structure: SecondaryStructure::Coil,
        }
    }

    /// Build a chain containing `n` residues numbered 1..=n, all ALA by default.
    fn make_chain(id: char, n: usize, base_atom_id: u32) -> (Chain, Vec<Atom>) {
        let mut atoms = Vec::with_capacity(n);
        let mut residues = Vec::with_capacity(n);

        for i in 0..n {
            let seq = (i + 1) as i32;
            let atom_idx = i; // 0-based offset; caller must add base_atom_id offset in molecule
            let atom_id = base_atom_id + i as u32;
            let x = i as f64 * 3.8; // ~peptide bond Cα–Cα distance
            atoms.push(ca_atom(atom_id, [x, 0.0, 0.0], "ALA", id, seq));
            residues.push(make_residue("ALA", seq, vec![atom_idx]));
        }

        (Chain { id, residues }, atoms)
    }

    /// Build a chain with specified residue names for each position.
    fn make_named_chain(id: char, names: &[&str], base_atom_id: u32) -> (Chain, Vec<Atom>) {
        let n = names.len();
        let mut atoms = Vec::with_capacity(n);
        let mut residues = Vec::with_capacity(n);

        for (i, &name) in names.iter().enumerate() {
            let seq = (i + 1) as i32;
            let atom_id = base_atom_id + i as u32;
            atoms.push(ca_atom(atom_id, [i as f64 * 3.8, 0.0, 0.0], name, id, seq));
            residues.push(make_residue(name, seq, vec![i]));
        }

        (Chain { id, residues }, atoms)
    }

    // -------------------------------------------------------------------------
    // Test 1: Short chain classified as light
    // -------------------------------------------------------------------------

    #[test]
    fn classify_short_chain_as_light() {
        // 210 residues — below the 300 threshold, should be classified as light.
        let (chain, atoms) = make_chain('L', 210, 1);
        let result = classify_chain(&chain, &atoms);
        assert!(
            matches!(result, IgChainType::LightKappa | IgChainType::LightLambda),
            "expected a light chain, got {result:?}"
        );
    }

    // -------------------------------------------------------------------------
    // Test 2: Long chain classified as heavy
    // -------------------------------------------------------------------------

    #[test]
    fn classify_long_chain_as_heavy() {
        // 450 residues — above the 300 threshold, should be Heavy.
        let (chain, atoms) = make_chain('H', 450, 1);
        let result = classify_chain(&chain, &atoms);
        assert_eq!(result, IgChainType::Heavy);
    }

    // -------------------------------------------------------------------------
    // Test 3: Heavy-chain domains
    // -------------------------------------------------------------------------

    #[test]
    fn identify_heavy_domains() {
        // Build a heavy chain with 476 residues to cover all domain windows.
        let (chain, _) = make_chain('H', 476, 1);
        let domains = identify_domains(&chain, IgChainType::Heavy);

        let domain_types: Vec<IgDomain> = domains.iter().map(|d| d.domain).collect();

        assert!(domain_types.contains(&IgDomain::VH), "missing VH");
        assert!(domain_types.contains(&IgDomain::CH1), "missing CH1");
        assert!(domain_types.contains(&IgDomain::Hinge), "missing Hinge");
        assert!(domain_types.contains(&IgDomain::CH2), "missing CH2");
        assert!(domain_types.contains(&IgDomain::CH3), "missing CH3");
        assert_eq!(domain_types.len(), 5, "expected exactly 5 heavy domains");
    }

    // -------------------------------------------------------------------------
    // Test 4: Light-chain domains
    // -------------------------------------------------------------------------

    #[test]
    fn identify_light_domains() {
        // Build a light chain with 214 residues to cover VL + CL.
        let (chain, _) = make_chain('L', 214, 1);
        let domains = identify_domains(&chain, IgChainType::LightKappa);

        let domain_types: Vec<IgDomain> = domains.iter().map(|d| d.domain).collect();

        assert!(domain_types.contains(&IgDomain::VL), "missing VL");
        assert!(domain_types.contains(&IgDomain::CL), "missing CL");
        assert_eq!(domain_types.len(), 2, "expected exactly 2 light domains");
    }

    // -------------------------------------------------------------------------
    // Test 5: CDR-H1 location
    // -------------------------------------------------------------------------

    #[test]
    fn cdr_h1_location() {
        // 450-residue heavy chain — Kabat H1 spans residues 26–35.
        let (chain, atoms) = make_chain('H', 450, 1);
        let cdrs = locate_cdrs_kabat(&chain, IgChainType::Heavy, &atoms);

        let h1 = cdrs.iter().find(|c| c.id == CdrId::H1);
        assert!(h1.is_some(), "CDR-H1 not found");

        let h1 = h1.unwrap_or_else(|| unreachable!());
        assert!(h1.start_residue >= 26, "H1 starts too early: {}", h1.start_residue);
        assert!(h1.end_residue <= 35, "H1 ends too late: {}", h1.end_residue);
        assert!(h1.length > 0, "H1 must have residues");
    }

    // -------------------------------------------------------------------------
    // Test 6: CDR-L1 location
    // -------------------------------------------------------------------------

    #[test]
    fn cdr_l1_location() {
        // 214-residue light chain — Kabat L1 spans residues 24–34.
        let (chain, atoms) = make_chain('L', 214, 1);
        let cdrs = locate_cdrs_kabat(&chain, IgChainType::LightKappa, &atoms);

        let l1 = cdrs.iter().find(|c| c.id == CdrId::L1);
        assert!(l1.is_some(), "CDR-L1 not found");

        let l1 = l1.unwrap_or_else(|| unreachable!());
        assert!(l1.start_residue >= 24, "L1 starts too early: {}", l1.start_residue);
        assert!(l1.end_residue <= 34, "L1 ends too late: {}", l1.end_residue);
        assert!(l1.length > 0, "L1 must have residues");
    }

    // -------------------------------------------------------------------------
    // Test 7: Close sulfurs form a disulfide bond
    // -------------------------------------------------------------------------

    #[test]
    fn disulfide_close_sulfurs() {
        // Two SG atoms separated by 2.0 Å — within the 2.2 Å threshold.
        let mut mol = Molecule::new("ss-test");
        mol.atoms.push(cys_sg_atom(1, [0.0, 0.0, 0.0], 'A', 22));
        mol.atoms.push(cys_sg_atom(2, [2.0, 0.0, 0.0], 'A', 95));

        let bonds = detect_disulfide_bonds(&mol, 2.2);
        assert_eq!(bonds.len(), 1, "expected one disulfide bond, got {}", bonds.len());
        assert_eq!(bonds[0], (0, 1));
    }

    // -------------------------------------------------------------------------
    // Test 8: Far sulfurs are not detected as disulfide bonds
    // -------------------------------------------------------------------------

    #[test]
    fn disulfide_far_sulfurs_ignored() {
        // Two SG atoms separated by 5.0 Å — beyond the 2.2 Å threshold.
        let mut mol = Molecule::new("ss-far-test");
        mol.atoms.push(cys_sg_atom(1, [0.0, 0.0, 0.0], 'A', 22));
        mol.atoms.push(cys_sg_atom(2, [5.0, 0.0, 0.0], 'A', 95));

        let bonds = detect_disulfide_bonds(&mol, 2.2);
        assert!(bonds.is_empty(), "no bonds expected for atoms 5.0 Å apart");
    }

    // -------------------------------------------------------------------------
    // Test 9: Fragment mapping produces Fab and Fc
    // -------------------------------------------------------------------------

    #[test]
    fn fragment_mapping_produces_fab_fc() {
        // Simulate a minimal IgG: two heavy chains (H, I) and two light chains (L, M).
        let chain_types = vec![
            ('H', IgChainType::Heavy),
            ('I', IgChainType::Heavy),
            ('L', IgChainType::LightKappa),
            ('M', IgChainType::LightKappa),
        ];

        // Provide representative domain regions for both heavy chains and both lights.
        let domains = vec![
            // Heavy H — Fab portion
            DomainRegion { domain: IgDomain::VH,    chain_id: 'H', start_residue: 1,   end_residue: 113, residue_count: 113 },
            DomainRegion { domain: IgDomain::CH1,   chain_id: 'H', start_residue: 114, end_residue: 223, residue_count: 110 },
            DomainRegion { domain: IgDomain::Hinge, chain_id: 'H', start_residue: 224, end_residue: 243, residue_count: 20  },
            DomainRegion { domain: IgDomain::CH2,   chain_id: 'H', start_residue: 244, end_residue: 360, residue_count: 117 },
            DomainRegion { domain: IgDomain::CH3,   chain_id: 'H', start_residue: 361, end_residue: 476, residue_count: 116 },
            // Heavy I — same structure
            DomainRegion { domain: IgDomain::VH,    chain_id: 'I', start_residue: 1,   end_residue: 113, residue_count: 113 },
            DomainRegion { domain: IgDomain::CH1,   chain_id: 'I', start_residue: 114, end_residue: 223, residue_count: 110 },
            DomainRegion { domain: IgDomain::Hinge, chain_id: 'I', start_residue: 224, end_residue: 243, residue_count: 20  },
            DomainRegion { domain: IgDomain::CH2,   chain_id: 'I', start_residue: 244, end_residue: 360, residue_count: 117 },
            DomainRegion { domain: IgDomain::CH3,   chain_id: 'I', start_residue: 361, end_residue: 476, residue_count: 116 },
            // Light L — Fab portion
            DomainRegion { domain: IgDomain::VL, chain_id: 'L', start_residue: 1,   end_residue: 107, residue_count: 107 },
            DomainRegion { domain: IgDomain::CL, chain_id: 'L', start_residue: 108, end_residue: 214, residue_count: 107 },
            // Light M
            DomainRegion { domain: IgDomain::VL, chain_id: 'M', start_residue: 1,   end_residue: 107, residue_count: 107 },
            DomainRegion { domain: IgDomain::CL, chain_id: 'M', start_residue: 108, end_residue: 214, residue_count: 107 },
        ];

        let fragments = map_fragments(&chain_types, &domains);

        let fab_count = fragments
            .iter()
            .filter(|f| f.fragment_type == FragmentType::Fab)
            .count();
        let fc_count = fragments
            .iter()
            .filter(|f| f.fragment_type == FragmentType::Fc)
            .count();
        let hinge_count = fragments
            .iter()
            .filter(|f| f.fragment_type == FragmentType::Hinge)
            .count();

        assert_eq!(fab_count, 2, "expected 2 Fab fragments (one per heavy-light pair)");
        assert_eq!(fc_count, 1, "expected 1 Fc fragment");
        assert_eq!(hinge_count, 1, "expected 1 Hinge fragment");
    }

    // -------------------------------------------------------------------------
    // Test 10: analyze_antibody on empty molecule
    // -------------------------------------------------------------------------

    #[test]
    fn analyze_empty_molecule_returns_empty_topology() {
        let mol = Molecule::new("empty");
        let topo = analyze_antibody(&mol);
        assert!(topo.chain_types.is_empty());
        assert!(topo.domains.is_empty());
        assert!(topo.cdrs.is_empty());
        assert!(topo.fragments.is_empty());
        assert!(topo.disulfide_bonds.is_empty());
    }

    // -------------------------------------------------------------------------
    // Test 11: CDR sequence extraction
    // -------------------------------------------------------------------------

    #[test]
    fn cdr_sequence_extraction() {
        // Build a heavy chain where CDR-H1 residues (26–35) are known types.
        // We want to verify the one-letter codes appear in the CDR sequence.
        let mut names = vec!["ALA"; 450];
        // Place known residues in CDR-H1 window (indices 25–34, 0-based → seq 26–35).
        for i in 25..35 {
            names[i] = "TYR"; // Y
        }

        let name_strs: Vec<&str> = names;
        let (chain, atoms) = make_named_chain('H', &name_strs, 1);
        let cdrs = locate_cdrs_kabat(&chain, IgChainType::Heavy, &atoms);

        let h1 = cdrs.iter().find(|c| c.id == CdrId::H1);
        assert!(h1.is_some(), "CDR-H1 not found");
        let h1 = h1.unwrap_or_else(|| unreachable!());
        assert!(
            h1.sequence.chars().all(|c| c == 'Y'),
            "expected all TYR (Y) in H1 sequence, got: {}",
            h1.sequence
        );
    }

    // -------------------------------------------------------------------------
    // Test 12: Non-SG sulfur atoms not counted as disulfide candidates
    // -------------------------------------------------------------------------

    #[test]
    fn non_sg_sulfur_ignored_for_disulfide() {
        // A MET SD atom (sulfur but not SG on CYS) should not count.
        let mut mol = Molecule::new("met-test");
        mol.atoms.push(Atom {
            id: 1,
            element: Element::S,
            position: [0.0, 0.0, 0.0],
            charge: 0,
            name: "SD".to_string(),
            residue_name: Some("MET".to_string()),
            residue_seq: Some(10),
            chain_id: Some('A'),
            b_factor: None,
        });
        mol.atoms.push(Atom {
            id: 2,
            element: Element::S,
            position: [1.5, 0.0, 0.0],
            charge: 0,
            name: "SD".to_string(),
            residue_name: Some("MET".to_string()),
            residue_seq: Some(50),
            chain_id: Some('A'),
            b_factor: None,
        });

        let bonds = detect_disulfide_bonds(&mol, 2.2);
        assert!(
            bonds.is_empty(),
            "MET SD atoms should not be counted as disulfide candidates"
        );
    }

    // -------------------------------------------------------------------------
    // Test 13: three_to_one coverage
    // -------------------------------------------------------------------------

    #[test]
    fn three_to_one_known_and_unknown() {
        assert_eq!(three_to_one("ALA"), 'A');
        assert_eq!(three_to_one("CYS"), 'C');
        assert_eq!(three_to_one("TRP"), 'W');
        assert_eq!(three_to_one("UNK"), 'X');
        assert_eq!(three_to_one(""), 'X');
    }

    // -------------------------------------------------------------------------
    // Test 14: Kappa classification via C-terminal fingerprint
    // -------------------------------------------------------------------------

    #[test]
    fn classify_kappa_via_cys_thr_fingerprint() {
        // Build a 214-residue chain ending in …CYS, THR, CYS (kappa fingerprint).
        let mut names = vec!["ALA"; 214];
        // Place CYS and THR near the end.
        names[210] = "CYS";
        names[211] = "THR";
        names[212] = "CYS";
        names[213] = "GLY";

        let (chain, atoms) = make_named_chain('L', &names, 1);
        let result = classify_chain(&chain, &atoms);

        // Has CYS and THR near end, no SER → Kappa
        assert_eq!(result, IgChainType::LightKappa);
    }
}
