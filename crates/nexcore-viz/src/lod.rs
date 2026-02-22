//! # Molecular Level-of-Detail (LOD) System
//!
//! Selects and generates coarse-to-fine molecular representations based on
//! the number of visible atoms, enabling interactive rendering of macromolecules
//! at all zoom levels without GPU overload.
//!
//! ## The Five Semantic Zoom Levels
//!
//! | Level | Atom count | Representation |
//! |-------|-----------|----------------|
//! | `FullAtom` | < 500 | Every atom + bond |
//! | `CalphaTrace` | 500–4999 | Cα backbone trace |
//! | `SecondaryStructure` | 5000–19999 | Helix cylinders, sheet arrows, coil tubes |
//! | `DomainBlob` | 20000–49999 | One sphere per domain segment |
//! | `ConvexHull` | ≥ 50000 | One convex hull per chain |
//!
//! ## Quick start
//!
//! ```rust
//! use nexcore_viz::lod::{LodConfig, auto_lod};
//! use nexcore_viz::molecular::Molecule;
//!
//! let mol = Molecule::new("empty");
//! let config = LodConfig::default();
//! // auto_lod returns Err(LodError::EmptyMolecule) for an empty molecule
//! assert!(auto_lod(&mol, &config).is_err());
//! ```
//!
//! Primitive decomposition:
//!   σ(LOD sequence) + κ(threshold comparison) + μ(atom→representation) + ς(state: level choice)

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::molecular::{Molecule, SecondaryStructure};

// ============================================================================
// LodError
// ============================================================================

/// Errors produced by the LOD system.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum LodError {
    /// The molecule contains no atoms.
    EmptyMolecule,
    /// The molecule has no chain hierarchy (required for protein LOD levels).
    NoChains,
    /// The requested LOD level is inconsistent with the molecule's atom count.
    InvalidLevel {
        /// The level that was requested.
        requested: String,
        /// Actual atom count.
        atom_count: usize,
    },
    /// No Cα atoms could be found in the molecule.
    NoCalpha,
}

impl fmt::Display for LodError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMolecule => write!(f, "molecule contains no atoms"),
            Self::NoChains => write!(f, "molecule has no chain hierarchy"),
            Self::InvalidLevel {
                requested,
                atom_count,
            } => write!(
                f,
                "LOD level '{requested}' is inconsistent with {atom_count} atoms"
            ),
            Self::NoCalpha => write!(f, "no Cα atoms found in molecule"),
        }
    }
}

impl std::error::Error for LodError {}

// ============================================================================
// LodLevel
// ============================================================================

/// The five semantic zoom levels for molecular visualization.
///
/// Each level trades geometric detail for rendering performance.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LodLevel {
    /// All atoms and all bonds — used when fewer than 500 atoms are visible.
    FullAtom,
    /// Only Cα atoms connected by trace bonds — 500 to 4 999 atoms.
    CalphaTrace,
    /// Helix cylinders, sheet arrows, coil tubes — 5 000 to 19 999 atoms.
    SecondaryStructure,
    /// One sphere per domain/chain segment — 20 000 to 49 999 atoms.
    DomainBlob,
    /// Single convex hull per chain — 50 000 or more atoms.
    ConvexHull,
}

impl LodLevel {
    /// Select the appropriate LOD level for `n` visible atoms using the
    /// default threshold table `[500, 5000, 20000, 50000]`.
    ///
    /// ```
    /// use nexcore_viz::lod::LodLevel;
    ///
    /// assert_eq!(LodLevel::from_atom_count(100), LodLevel::FullAtom);
    /// assert_eq!(LodLevel::from_atom_count(500), LodLevel::CalphaTrace);
    /// assert_eq!(LodLevel::from_atom_count(5_000), LodLevel::SecondaryStructure);
    /// assert_eq!(LodLevel::from_atom_count(20_000), LodLevel::DomainBlob);
    /// assert_eq!(LodLevel::from_atom_count(50_000), LodLevel::ConvexHull);
    /// ```
    #[must_use]
    pub fn from_atom_count(n: usize) -> Self {
        if n < 500 {
            Self::FullAtom
        } else if n < 5_000 {
            Self::CalphaTrace
        } else if n < 20_000 {
            Self::SecondaryStructure
        } else if n < 50_000 {
            Self::DomainBlob
        } else {
            Self::ConvexHull
        }
    }

    /// Detail index: 0 = coarsest (`ConvexHull`), 4 = finest (`FullAtom`).
    ///
    /// ```
    /// use nexcore_viz::lod::LodLevel;
    ///
    /// assert_eq!(LodLevel::FullAtom.detail_index(), 4);
    /// assert_eq!(LodLevel::ConvexHull.detail_index(), 0);
    /// ```
    #[must_use]
    pub fn detail_index(&self) -> u8 {
        match self {
            Self::ConvexHull => 0,
            Self::DomainBlob => 1,
            Self::SecondaryStructure => 2,
            Self::CalphaTrace => 3,
            Self::FullAtom => 4,
        }
    }
}

impl fmt::Display for LodLevel {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let name = match self {
            Self::FullAtom => "FullAtom",
            Self::CalphaTrace => "CalphaTrace",
            Self::SecondaryStructure => "SecondaryStructure",
            Self::DomainBlob => "DomainBlob",
            Self::ConvexHull => "ConvexHull",
        };
        write!(f, "{name}")
    }
}

// ============================================================================
// LodConfig
// ============================================================================

/// Configuration for LOD threshold selection and blending.
///
/// The four thresholds `[t0, t1, t2, t3]` partition atom-count space:
///
/// ```text
/// [0, t0)     → FullAtom
/// [t0, t1)    → CalphaTrace
/// [t1, t2)    → SecondaryStructure
/// [t2, t3)    → DomainBlob
/// [t3, ∞)     → ConvexHull
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodConfig {
    /// Transition thresholds `[t0, t1, t2, t3]`.
    ///
    /// Default: `[500, 5000, 20000, 50000]`.
    pub thresholds: [usize; 4],
    /// Blend factor in `[0.0, 1.0]` for smooth LOD transitions.
    ///
    /// A value of `0.0` means hard cuts; `1.0` means the renderer may
    /// overlap two consecutive representations in the transition zone.
    pub transition_blend: f64,
}

impl Default for LodConfig {
    fn default() -> Self {
        Self {
            thresholds: [500, 5_000, 20_000, 50_000],
            transition_blend: 0.1,
        }
    }
}

impl LodConfig {
    /// Derive the `LodLevel` for `n` atoms using this config's thresholds.
    ///
    /// ```
    /// use nexcore_viz::lod::{LodConfig, LodLevel};
    ///
    /// let cfg = LodConfig { thresholds: [100, 1000, 5000, 10000], transition_blend: 0.0 };
    /// assert_eq!(cfg.level_for(50), LodLevel::FullAtom);
    /// assert_eq!(cfg.level_for(100), LodLevel::CalphaTrace);
    /// assert_eq!(cfg.level_for(10_000), LodLevel::ConvexHull);
    /// ```
    #[must_use]
    pub fn level_for(&self, n: usize) -> LodLevel {
        let [t0, t1, t2, t3] = self.thresholds;
        if n < t0 {
            LodLevel::FullAtom
        } else if n < t1 {
            LodLevel::CalphaTrace
        } else if n < t2 {
            LodLevel::SecondaryStructure
        } else if n < t3 {
            LodLevel::DomainBlob
        } else {
            LodLevel::ConvexHull
        }
    }
}

// ============================================================================
// CalphaPoint
// ============================================================================

/// A single Cα atom position with residue metadata for trace rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalphaPoint {
    /// 3D position in angstroms.
    pub position: [f64; 3],
    /// Three-letter residue name (e.g. `"ALA"`).
    pub residue_name: String,
    /// Chain identifier.
    pub chain_id: char,
    /// Residue sequence number.
    pub seq: i32,
    /// Secondary structure at this position.
    pub secondary_structure: SecondaryStructure,
}

// ============================================================================
// SecondaryStructureSegment
// ============================================================================

/// A contiguous run of residues sharing the same secondary structure type.
///
/// Used to render helices as cylinders, sheets as arrows, and coils as tubes.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SecondaryStructureSegment {
    /// Secondary structure type for this segment.
    pub kind: SecondaryStructure,
    /// Chain that contains this segment.
    pub chain_id: char,
    /// First residue sequence number.
    pub start_seq: i32,
    /// Last residue sequence number.
    pub end_seq: i32,
    /// Cα positions along the segment centre line.
    pub points: Vec<[f64; 3]>,
    /// Unit direction vector from start to end Cα.
    pub direction: [f64; 3],
}

// ============================================================================
// DomainBlob
// ============================================================================

/// A sphere enclosing a group of consecutive residues (one "domain" chunk).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DomainBlob {
    /// Geometric centre of the blob.
    pub center: [f64; 3],
    /// Enclosing sphere radius.
    pub radius: f64,
    /// Chain that the blob belongs to.
    pub chain_id: char,
    /// First residue sequence number in the group.
    pub start_seq: i32,
    /// Last residue sequence number in the group.
    pub end_seq: i32,
    /// Number of heavy atoms inside the blob.
    pub atom_count: usize,
    /// Suggested color key for the frontend (e.g. secondary structure hint).
    pub color_hint: String,
}

// ============================================================================
// ConvexHullData
// ============================================================================

/// A 3-D convex hull for a single chain.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ConvexHullData {
    /// Hull vertex positions.
    pub vertices: Vec<[f64; 3]>,
    /// Triangles as index triples into `vertices`.
    pub triangles: Vec<[usize; 3]>,
    /// Geometric centre of the hull.
    pub center: [f64; 3],
    /// Chain identifier.
    pub chain_id: char,
}

// ============================================================================
// LodRepresentation
// ============================================================================

/// The computed LOD representation for a molecule at a particular zoom level.
///
/// Exactly one of the `Option` fields is `Some`, corresponding to `level`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LodRepresentation {
    /// The zoom level this representation was generated for.
    pub level: LodLevel,
    /// Cα trace (present when `level == CalphaTrace`).
    pub calpha_trace: Option<Vec<CalphaPoint>>,
    /// Secondary structure segments (present when `level == SecondaryStructure`).
    pub ss_segments: Option<Vec<SecondaryStructureSegment>>,
    /// Domain blobs (present when `level == DomainBlob`).
    pub domain_blobs: Option<Vec<DomainBlob>>,
    /// Convex hulls (present when `level == ConvexHull`).
    pub convex_hulls: Option<Vec<ConvexHullData>>,
    /// Total atom count in the source molecule.
    pub atom_count: usize,
    /// Number of geometric primitives in this representation.
    ///
    /// For `FullAtom` this equals `atom_count`; for coarser levels it is
    /// the number of trace points, segments, blobs, or hull vertices.
    pub reduced_count: usize,
}

// ============================================================================
// Public API
// ============================================================================

/// Select the LOD level for `mol` using `config`'s thresholds.
///
/// ```
/// use nexcore_viz::lod::{LodConfig, LodLevel, select_lod_level};
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
///
/// let mut mol = Molecule::new("tiny");
/// for i in 0..10u32 {
///     mol.atoms.push(Atom::new(i + 1, Element::C, [i as f64, 0.0, 0.0]));
/// }
/// assert_eq!(select_lod_level(&mol, &LodConfig::default()), LodLevel::FullAtom);
/// ```
#[must_use]
pub fn select_lod_level(mol: &Molecule, config: &LodConfig) -> LodLevel {
    config.level_for(mol.atoms.len())
}

/// Extract Cα atoms from all chains in `mol`, ordered by chain then residue.
///
/// # Errors
///
/// - [`LodError::EmptyMolecule`] if `mol` has no atoms.
/// - [`LodError::NoChains`] if `mol` has no chain hierarchy.
/// - [`LodError::NoCalpha`] if no Cα atom was found in any residue.
pub fn extract_calpha_trace(mol: &Molecule) -> Result<Vec<CalphaPoint>, LodError> {
    if mol.atoms.is_empty() {
        return Err(LodError::EmptyMolecule);
    }
    if mol.chains.is_empty() {
        return Err(LodError::NoChains);
    }

    let mut points: Vec<CalphaPoint> = Vec::new();

    for chain in &mol.chains {
        for residue in &chain.residues {
            // Look for an atom named "CA" (Cα) among this residue's atoms.
            let ca = residue.atom_indices.iter().find_map(|&idx| {
                let atom = mol.atoms.get(idx)?;
                if atom.name.trim() == "CA" {
                    Some(atom)
                } else {
                    None
                }
            });

            if let Some(atom) = ca {
                points.push(CalphaPoint {
                    position: atom.position,
                    residue_name: residue.name.clone(),
                    chain_id: chain.id,
                    seq: residue.seq,
                    secondary_structure: residue.secondary_structure,
                });
            }
        }
    }

    if points.is_empty() {
        return Err(LodError::NoCalpha);
    }

    Ok(points)
}

/// Group consecutive residues with the same secondary structure into segments.
///
/// Each segment carries the Cα centre-line points and a unit direction vector.
///
/// # Errors
///
/// - [`LodError::EmptyMolecule`] if `mol` has no atoms.
/// - [`LodError::NoChains`] if `mol` has no chain hierarchy.
pub fn compute_ss_segments(mol: &Molecule) -> Result<Vec<SecondaryStructureSegment>, LodError> {
    if mol.atoms.is_empty() {
        return Err(LodError::EmptyMolecule);
    }
    if mol.chains.is_empty() {
        return Err(LodError::NoChains);
    }

    let mut segments: Vec<SecondaryStructureSegment> = Vec::new();

    for chain in &mol.chains {
        if chain.residues.is_empty() {
            continue;
        }

        // Iterate residues, flushing a segment whenever the SS type changes.
        let mut seg_kind = chain.residues[0].secondary_structure;
        let mut seg_points: Vec<[f64; 3]> = Vec::new();
        let mut seg_start_seq = chain.residues[0].seq;
        let mut seg_end_seq = chain.residues[0].seq;

        for residue in &chain.residues {
            if residue.secondary_structure != seg_kind && !seg_points.is_empty() {
                // Flush current segment.
                let direction = compute_direction(&seg_points);
                segments.push(SecondaryStructureSegment {
                    kind: seg_kind,
                    chain_id: chain.id,
                    start_seq: seg_start_seq,
                    end_seq: seg_end_seq,
                    points: seg_points.clone(),
                    direction,
                });
                // Start a new segment.
                seg_kind = residue.secondary_structure;
                seg_points.clear();
                seg_start_seq = residue.seq;
            }

            // Append the Cα of this residue to the current segment.
            let ca_pos = residue.atom_indices.iter().find_map(|&idx| {
                let atom = mol.atoms.get(idx)?;
                if atom.name.trim() == "CA" {
                    Some(atom.position)
                } else {
                    None
                }
            });

            if let Some(pos) = ca_pos {
                seg_points.push(pos);
            }
            seg_end_seq = residue.seq;
        }

        // Flush the last segment.
        if !seg_points.is_empty() {
            let direction = compute_direction(&seg_points);
            segments.push(SecondaryStructureSegment {
                kind: seg_kind,
                chain_id: chain.id,
                start_seq: seg_start_seq,
                end_seq: seg_end_seq,
                points: seg_points,
                direction,
            });
        }
    }

    Ok(segments)
}

/// Group every `segment_size` residues (per chain) into a domain blob.
///
/// The blob centre is the mean of all atom positions in the group; the radius
/// is the maximum distance from the centre to any atom in the group.
///
/// # Errors
///
/// - [`LodError::EmptyMolecule`] if `mol` has no atoms.
/// - [`LodError::NoChains`] if `mol` has no chain hierarchy.
pub fn compute_domain_blobs(
    mol: &Molecule,
    segment_size: usize,
) -> Result<Vec<DomainBlob>, LodError> {
    if mol.atoms.is_empty() {
        return Err(LodError::EmptyMolecule);
    }
    if mol.chains.is_empty() {
        return Err(LodError::NoChains);
    }

    let effective_size = if segment_size == 0 { 1 } else { segment_size };
    let mut blobs: Vec<DomainBlob> = Vec::new();

    for chain in &mol.chains {
        // Split residues into chunks of `effective_size`.
        for chunk in chain.residues.chunks(effective_size) {
            if chunk.is_empty() {
                continue;
            }

            // Collect all atom positions for the chunk.
            let positions: Vec<[f64; 3]> = chunk
                .iter()
                .flat_map(|r| r.atom_indices.iter())
                .filter_map(|&idx| mol.atoms.get(idx).map(|a| a.position))
                .collect();

            if positions.is_empty() {
                continue;
            }

            let center = mean_position(&positions);
            let radius = positions
                .iter()
                .map(|p| euclidean_distance(p, &center))
                .fold(0.0_f64, f64::max);

            let start_seq = chunk[0].seq;
            let end_seq = chunk[chunk.len() - 1].seq;

            // Derive a color hint from the most common secondary structure.
            let color_hint = dominant_ss_color(chunk);

            blobs.push(DomainBlob {
                center,
                radius,
                chain_id: chain.id,
                start_seq,
                end_seq,
                atom_count: positions.len(),
                color_hint,
            });
        }
    }

    Ok(blobs)
}

/// Compute the 3-D convex hull of `points` via the gift-wrapping algorithm.
///
/// Returns a `ConvexHullData` with `chain_id` set to `'?'`. The caller
/// should update `chain_id` to the appropriate chain.
///
/// The implementation is O(n · h) where h is the number of hull faces.
/// It is intended for LOD pre-computation, not real-time use.
///
/// # Behaviour on degenerate input
///
/// - Fewer than 4 points: returns an empty hull (no triangles).
/// - All points coplanar: may return no triangles (gift wrapping degenerates).
#[must_use]
pub fn compute_convex_hull(points: &[[f64; 3]]) -> ConvexHullData {
    if points.len() < 4 {
        let center = if points.is_empty() {
            [0.0; 3]
        } else {
            mean_position(points)
        };
        return ConvexHullData {
            vertices: points.to_vec(),
            triangles: Vec::new(),
            center,
            chain_id: '?',
        };
    }

    let center = mean_position(points);
    let triangles = gift_wrap_hull(points, &center);

    ConvexHullData {
        vertices: points.to_vec(),
        triangles,
        center,
        chain_id: '?',
    }
}

/// Generate a `LodRepresentation` for `mol` at the requested `level`.
///
/// `FullAtom` is always valid (even for large molecules — the caller is
/// responsible for choosing a sensible level).  The protein-specific levels
/// (`CalphaTrace`, `SecondaryStructure`, `DomainBlob`, `ConvexHull`) require
/// the molecule to have chain information.
///
/// # Errors
///
/// Propagates errors from the underlying extraction functions.
pub fn generate_lod(
    mol: &Molecule,
    level: LodLevel,
    _config: &LodConfig,
) -> Result<LodRepresentation, LodError> {
    if mol.atoms.is_empty() {
        return Err(LodError::EmptyMolecule);
    }

    let atom_count = mol.atoms.len();

    match level {
        LodLevel::FullAtom => Ok(LodRepresentation {
            level,
            calpha_trace: None,
            ss_segments: None,
            domain_blobs: None,
            convex_hulls: None,
            atom_count,
            reduced_count: atom_count,
        }),

        LodLevel::CalphaTrace => {
            let trace = extract_calpha_trace(mol)?;
            let reduced_count = trace.len();
            Ok(LodRepresentation {
                level,
                calpha_trace: Some(trace),
                ss_segments: None,
                domain_blobs: None,
                convex_hulls: None,
                atom_count,
                reduced_count,
            })
        }

        LodLevel::SecondaryStructure => {
            let segments = compute_ss_segments(mol)?;
            let reduced_count = segments.iter().map(|s| s.points.len()).sum();
            Ok(LodRepresentation {
                level,
                calpha_trace: None,
                ss_segments: Some(segments),
                domain_blobs: None,
                convex_hulls: None,
                atom_count,
                reduced_count,
            })
        }

        LodLevel::DomainBlob => {
            let blobs = compute_domain_blobs(mol, 20)?;
            let reduced_count = blobs.len();
            Ok(LodRepresentation {
                level,
                calpha_trace: None,
                ss_segments: None,
                domain_blobs: Some(blobs),
                convex_hulls: None,
                atom_count,
                reduced_count,
            })
        }

        LodLevel::ConvexHull => {
            if mol.chains.is_empty() {
                return Err(LodError::NoChains);
            }
            let mut hulls: Vec<ConvexHullData> = Vec::new();
            for chain in &mol.chains {
                // Collect all atom positions for the chain.
                let positions: Vec<[f64; 3]> = chain
                    .residues
                    .iter()
                    .flat_map(|r| r.atom_indices.iter())
                    .filter_map(|&idx| mol.atoms.get(idx).map(|a| a.position))
                    .collect();

                if positions.is_empty() {
                    continue;
                }

                let mut hull = compute_convex_hull(&positions);
                hull.chain_id = chain.id;
                hulls.push(hull);
            }
            let reduced_count = hulls.iter().map(|h| h.vertices.len()).sum();
            Ok(LodRepresentation {
                level,
                calpha_trace: None,
                ss_segments: None,
                domain_blobs: None,
                convex_hulls: Some(hulls),
                atom_count,
                reduced_count,
            })
        }
    }
}

/// Automatically select the LOD level from `config` and generate the
/// representation for `mol`.
///
/// # Errors
///
/// Propagates errors from [`generate_lod`].
pub fn auto_lod(mol: &Molecule, config: &LodConfig) -> Result<LodRepresentation, LodError> {
    if mol.atoms.is_empty() {
        return Err(LodError::EmptyMolecule);
    }
    let level = select_lod_level(mol, config);
    generate_lod(mol, level, config)
}

/// Compute the reduction ratio: `reduced_count / atom_count`.
///
/// Returns `1.0` for an empty molecule (no reduction is possible).
///
/// ```
/// use nexcore_viz::lod::{LodLevel, LodRepresentation, reduction_ratio};
///
/// let repr = LodRepresentation {
///     level: LodLevel::CalphaTrace,
///     calpha_trace: None,
///     ss_segments: None,
///     domain_blobs: None,
///     convex_hulls: None,
///     atom_count: 1000,
///     reduced_count: 100,
/// };
/// assert!((reduction_ratio(&repr) - 0.1).abs() < 1e-10);
/// ```
#[must_use]
pub fn reduction_ratio(repr: &LodRepresentation) -> f64 {
    if repr.atom_count == 0 {
        return 1.0;
    }
    repr.reduced_count as f64 / repr.atom_count as f64
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Compute the mean of a slice of 3-D positions.
fn mean_position(points: &[[f64; 3]]) -> [f64; 3] {
    if points.is_empty() {
        return [0.0; 3];
    }
    let n = points.len() as f64;
    let mut sum = [0.0_f64; 3];
    for p in points {
        sum[0] += p[0];
        sum[1] += p[1];
        sum[2] += p[2];
    }
    [sum[0] / n, sum[1] / n, sum[2] / n]
}

/// Euclidean distance between two 3-D points.
fn euclidean_distance(a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let dx = a[0] - b[0];
    let dy = a[1] - b[1];
    let dz = a[2] - b[2];
    (dx * dx + dy * dy + dz * dz).sqrt()
}

/// Unit direction vector from the first to the last point in a sequence.
///
/// Returns `[0.0, 0.0, 1.0]` if the sequence has fewer than two points or if
/// the start equals the end.
fn compute_direction(points: &[[f64; 3]]) -> [f64; 3] {
    if points.len() < 2 {
        return [0.0, 0.0, 1.0];
    }
    let first = points[0];
    let last = points[points.len() - 1];
    let dx = last[0] - first[0];
    let dy = last[1] - first[1];
    let dz = last[2] - first[2];
    let len = (dx * dx + dy * dy + dz * dz).sqrt();
    if len < 1e-12 {
        [0.0, 0.0, 1.0]
    } else {
        [dx / len, dy / len, dz / len]
    }
}

/// Choose a color hint based on the most frequently occurring secondary
/// structure in a chunk of residues.
fn dominant_ss_color(residues: &[crate::molecular::Residue]) -> String {
    let mut helix = 0usize;
    let mut sheet = 0usize;
    let mut coil = 0usize;

    for r in residues {
        match r.secondary_structure {
            SecondaryStructure::Helix | SecondaryStructure::Helix310 | SecondaryStructure::HelixPi => helix += 1,
            SecondaryStructure::Sheet => sheet += 1,
            _ => coil += 1,
        }
    }

    if helix >= sheet && helix >= coil {
        "#FF4444".to_string() // red for helix
    } else if sheet >= coil {
        "#4444FF".to_string() // blue for sheet
    } else {
        "#44AA44".to_string() // green for coil/turn
    }
}

// ============================================================================
// Convex hull: gift wrapping in 3-D
// ============================================================================

/// Dot product of two 3-D vectors.
fn dot(a: [f64; 3], b: [f64; 3]) -> f64 {
    a[0] * b[0] + a[1] * b[1] + a[2] * b[2]
}

/// Cross product of two 3-D vectors.
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Subtract two 3-D vectors.
fn sub(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [a[0] - b[0], a[1] - b[1], a[2] - b[2]]
}

/// Compute the outward-facing normal of triangle (a, b, c) given an interior
/// point `interior`.  Returns the normal and whether it needed to be flipped.
fn outward_normal(a: [f64; 3], b: [f64; 3], c: [f64; 3], interior: [f64; 3]) -> [f64; 3] {
    let ab = sub(b, a);
    let ac = sub(c, a);
    let n = cross(ab, ac);
    // Ensure the normal points away from the interior.
    let to_interior = sub(interior, a);
    if dot(n, to_interior) > 0.0 {
        // Normal faces inward — flip it (swap b and c by negating).
        [-n[0], -n[1], -n[2]]
    } else {
        n
    }
}

/// Find the index of the point in `points` that maximises the dot product with
/// `direction`, starting from `hint`.
fn furthest_in_direction(points: &[[f64; 3]], direction: [f64; 3]) -> usize {
    let mut best = 0usize;
    let mut best_val = f64::NEG_INFINITY;
    for (i, p) in points.iter().enumerate() {
        let v = dot(*p, direction);
        if v > best_val {
            best_val = v;
            best = i;
        }
    }
    best
}

/// Find among all points the one that makes the triangle (edge_a, edge_b, candidate)
/// turn "most clockwise" when viewed from outside the hull.
///
/// Returns the index or `None` if all points are on the wrong side.
fn next_hull_point(
    points: &[[f64; 3]],
    edge_a: usize,
    edge_b: usize,
    face_normal: [f64; 3],
) -> Option<usize> {
    let pa = points[edge_a];
    let pb = points[edge_b];
    let edge_dir = sub(pb, pa);

    let mut best_idx: Option<usize> = None;
    let mut best_angle: f64 = f64::NEG_INFINITY;

    for (i, &pc) in points.iter().enumerate() {
        if i == edge_a || i == edge_b {
            continue;
        }
        // Normal of candidate triangle.
        let candidate_n = cross(edge_dir, sub(pc, pa));
        let len = (candidate_n[0] * candidate_n[0]
            + candidate_n[1] * candidate_n[1]
            + candidate_n[2] * candidate_n[2])
            .sqrt();
        if len < 1e-12 {
            continue; // Degenerate — skip collinear points.
        }
        let candidate_n_unit = [
            candidate_n[0] / len,
            candidate_n[1] / len,
            candidate_n[2] / len,
        ];
        // Prefer normals that are most "outward" relative to the current face.
        let angle = dot(candidate_n_unit, face_normal);
        if angle > best_angle {
            best_angle = angle;
            best_idx = Some(i);
        }
    }

    best_idx
}

/// Simple O(n · h) gift-wrapping algorithm for 3-D convex hulls.
///
/// Returns a list of triangles (index triples into `points`).
///
/// Limitations:
/// - Stops after 4 × n² iterations to bound worst-case runtime.
/// - Does not handle perfectly coplanar point sets gracefully.
fn gift_wrap_hull(points: &[[f64; 3]], interior: &[f64; 3]) -> Vec<[usize; 3]> {
    if points.len() < 4 {
        return Vec::new();
    }

    // Start from the bottommost point (minimum y).
    let start = points
        .iter()
        .enumerate()
        .min_by(|&(_, a), &(_, b)| a[1].partial_cmp(&b[1]).unwrap_or(std::cmp::Ordering::Equal))
        .map(|(i, _)| i)
        .unwrap_or(0);

    // Find a second point: furthest from start in the x-direction.
    let dir_x = [1.0_f64, 0.0, 0.0];
    let second = furthest_in_direction(points, dir_x);
    let second = if second == start {
        (start + 1) % points.len()
    } else {
        second
    };

    let mut triangles: Vec<[usize; 3]> = Vec::new();
    // Queue of oriented edges to process (edge_a, edge_b, outward_normal).
    let mut queue: Vec<(usize, usize, [f64; 3])> = Vec::new();
    // Set of processed edges to avoid infinite loops.
    let mut visited: std::collections::HashSet<(usize, usize)> = std::collections::HashSet::new();

    // Build the initial triangle using the start edge.
    let init_normal = [0.0_f64, -1.0, 0.0]; // pointing downward to start.
    if let Some(third) = next_hull_point(points, start, second, init_normal) {
        // Orient the triangle so the normal faces outward.
        let n = outward_normal(points[start], points[second], points[third], *interior);
        // If normal is pointing up (away from interior), use (start, second, third).
        // The winding already chosen by outward_normal; we just need to enqueue
        // the three edges in the correct orientation.
        let tri = if dot(n, [0.0, -1.0, 0.0]) >= 0.0 {
            [start, second, third]
        } else {
            [start, third, second]
        };
        triangles.push(tri);
        visited.insert((tri[0], tri[1]));
        visited.insert((tri[1], tri[2]));
        visited.insert((tri[2], tri[0]));

        // Enqueue the three reverse edges for gift-wrapping.
        let face_n = outward_normal(points[tri[0]], points[tri[1]], points[tri[2]], *interior);
        queue.push((tri[1], tri[0], face_n));
        queue.push((tri[2], tri[1], face_n));
        queue.push((tri[0], tri[2], face_n));
    } else {
        return Vec::new();
    }

    let max_iters = 4 * points.len() * points.len();
    let mut iters = 0usize;

    while let Some((ea, eb, face_normal)) = queue.pop() {
        iters += 1;
        if iters > max_iters {
            break;
        }
        if visited.contains(&(ea, eb)) {
            continue;
        }

        if let Some(ec) = next_hull_point(points, ea, eb, face_normal) {
            let tri = [ea, eb, ec];
            triangles.push(tri);
            visited.insert((tri[0], tri[1]));
            visited.insert((tri[1], tri[2]));
            visited.insert((tri[2], tri[0]));

            let new_normal =
                outward_normal(points[tri[0]], points[tri[1]], points[tri[2]], *interior);
            for &(na, nb) in &[(tri[1], tri[0]), (tri[2], tri[1]), (tri[0], tri[2])] {
                if !visited.contains(&(na, nb)) {
                    queue.push((na, nb, new_normal));
                }
            }
        }
    }

    triangles
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{Atom, Bond, BondOrder, Chain, Element, Molecule, Residue, SecondaryStructure};

    // -----------------------------------------------------------------------
    // Helpers
    // -----------------------------------------------------------------------

    fn atom_ca(id: u32, pos: [f64; 3], chain: char, seq: i32, rname: &str) -> Atom {
        Atom {
            id,
            element: Element::C,
            position: pos,
            charge: 0,
            name: "CA".to_string(),
            residue_name: Some(rname.to_string()),
            residue_seq: Some(seq),
            chain_id: Some(chain),
            b_factor: None,
        }
    }

    /// Build a simple protein with two chains (A and B), each with a few
    /// residues.  Chain A residues are all `Helix`; chain B residues are all `Coil`.
    fn simple_protein(helix_a: usize, coil_b: usize) -> Molecule {
        let mut mol = Molecule::new("Test");
        let mut atom_id = 1u32;
        let mut chains: Vec<Chain> = Vec::new();

        // Chain A — all Helix
        {
            let mut residues: Vec<Residue> = Vec::new();
            for i in 0..helix_a {
                let idx = mol.atoms.len();
                // i is at most a small test value; saturating cast is safe here.
                #[allow(clippy::cast_possible_truncation)]
                let seq = i as i32 + 1;
                let pos = [i as f64, 0.0, 0.0];
                mol.atoms.push(atom_ca(atom_id, pos, 'A', seq, "ALA"));
                atom_id += 1;
                residues.push(Residue {
                    name: "ALA".to_string(),
                    seq,
                    insertion_code: None,
                    atom_indices: vec![idx],
                    secondary_structure: SecondaryStructure::Helix,
                });
            }
            chains.push(Chain { id: 'A', residues });
        }

        // Chain B — all Coil
        {
            let mut residues: Vec<Residue> = Vec::new();
            for i in 0..coil_b {
                let idx = mol.atoms.len();
                #[allow(clippy::cast_possible_truncation)]
                let seq = i as i32 + 1;
                let pos = [i as f64, 10.0, 0.0];
                mol.atoms.push(atom_ca(atom_id, pos, 'B', seq, "GLY"));
                atom_id += 1;
                residues.push(Residue {
                    name: "GLY".to_string(),
                    seq,
                    insertion_code: None,
                    atom_indices: vec![idx],
                    secondary_structure: SecondaryStructure::Coil,
                });
            }
            chains.push(Chain { id: 'B', residues });
        }

        mol.chains = chains;
        mol
    }

    // -----------------------------------------------------------------------
    // LodLevel::from_atom_count thresholds
    // -----------------------------------------------------------------------

    #[test]
    fn lod_level_thresholds() {
        assert_eq!(LodLevel::from_atom_count(0), LodLevel::FullAtom);
        assert_eq!(LodLevel::from_atom_count(499), LodLevel::FullAtom);
        assert_eq!(LodLevel::from_atom_count(500), LodLevel::CalphaTrace);
        assert_eq!(LodLevel::from_atom_count(4_999), LodLevel::CalphaTrace);
        assert_eq!(LodLevel::from_atom_count(5_000), LodLevel::SecondaryStructure);
        assert_eq!(LodLevel::from_atom_count(19_999), LodLevel::SecondaryStructure);
        assert_eq!(LodLevel::from_atom_count(20_000), LodLevel::DomainBlob);
        assert_eq!(LodLevel::from_atom_count(49_999), LodLevel::DomainBlob);
        assert_eq!(LodLevel::from_atom_count(50_000), LodLevel::ConvexHull);
        assert_eq!(LodLevel::from_atom_count(1_000_000), LodLevel::ConvexHull);
    }

    // -----------------------------------------------------------------------
    // Detail index ordering
    // -----------------------------------------------------------------------

    #[test]
    fn detail_index_ordering() {
        assert_eq!(LodLevel::ConvexHull.detail_index(), 0);
        assert_eq!(LodLevel::DomainBlob.detail_index(), 1);
        assert_eq!(LodLevel::SecondaryStructure.detail_index(), 2);
        assert_eq!(LodLevel::CalphaTrace.detail_index(), 3);
        assert_eq!(LodLevel::FullAtom.detail_index(), 4);
    }

    // -----------------------------------------------------------------------
    // Empty molecule errors
    // -----------------------------------------------------------------------

    #[test]
    fn empty_molecule_auto_lod_error() {
        let mol = Molecule::new("empty");
        let result = auto_lod(&mol, &LodConfig::default());
        assert!(matches!(result, Err(LodError::EmptyMolecule)));
    }

    #[test]
    fn empty_molecule_extract_calpha_error() {
        let mol = Molecule::new("empty");
        assert!(matches!(extract_calpha_trace(&mol), Err(LodError::EmptyMolecule)));
    }

    // -----------------------------------------------------------------------
    // Small molecule → FullAtom
    // -----------------------------------------------------------------------

    #[test]
    fn small_molecule_returns_full_atom() -> Result<(), LodError> {
        let mut mol = Molecule::new("small");
        for i in 0..10u32 {
            mol.atoms.push(Atom::new(i + 1, Element::C, [f64::from(i), 0.0, 0.0]));
        }
        mol.bonds.push(Bond { atom1: 0, atom2: 1, order: BondOrder::Single });

        let repr = auto_lod(&mol, &LodConfig::default())?;
        assert_eq!(repr.level, LodLevel::FullAtom);
        assert_eq!(repr.atom_count, 10);
        assert_eq!(repr.reduced_count, 10);
        assert!(repr.calpha_trace.is_none());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Cα trace extraction
    // -----------------------------------------------------------------------

    #[test]
    fn calpha_trace_extraction() -> Result<(), LodError> {
        let mol = simple_protein(5, 3);
        let trace = extract_calpha_trace(&mol)?;
        // 5 residues in chain A + 3 in chain B = 8 Cα points.
        assert_eq!(trace.len(), 8);
        assert_eq!(trace[0].chain_id, 'A');
        assert_eq!(trace[0].residue_name, "ALA");
        assert_eq!(trace[5].chain_id, 'B');
        assert_eq!(trace[5].residue_name, "GLY");
        Ok(())
    }

    #[test]
    fn calpha_trace_no_chains_error() {
        let mut mol = Molecule::new("ligand");
        mol.atoms.push(Atom::new(1, Element::C, [0.0; 3]));
        assert!(matches!(extract_calpha_trace(&mol), Err(LodError::NoChains)));
    }

    #[test]
    fn calpha_trace_no_ca_atoms_error() {
        // Build a chain whose residue atoms are NOT named "CA".
        let mut mol = Molecule::new("no_ca");
        mol.atoms.push(Atom {
            id: 1,
            element: Element::N,
            position: [0.0; 3],
            charge: 0,
            name: "N".to_string(), // backbone N, not CA
            residue_name: Some("ALA".to_string()),
            residue_seq: Some(1),
            chain_id: Some('A'),
            b_factor: None,
        });
        mol.chains.push(Chain {
            id: 'A',
            residues: vec![Residue {
                name: "ALA".to_string(),
                seq: 1,
                insertion_code: None,
                atom_indices: vec![0],
                secondary_structure: SecondaryStructure::Coil,
            }],
        });
        assert!(matches!(extract_calpha_trace(&mol), Err(LodError::NoCalpha)));
    }

    // -----------------------------------------------------------------------
    // Secondary structure segment grouping
    // -----------------------------------------------------------------------

    #[test]
    fn ss_segment_grouping() -> Result<(), LodError> {
        // Chain A: 3 helix + 2 coil → 2 segments.
        let mut mol = simple_protein(3, 0);
        // Override last 2 residues to Coil.
        if let Some(chain) = mol.chains.first_mut() {
            if let Some(res) = chain.residues.get_mut(1) {
                res.secondary_structure = SecondaryStructure::Coil;
            }
            if let Some(res) = chain.residues.get_mut(2) {
                res.secondary_structure = SecondaryStructure::Coil;
            }
        }

        let segs = compute_ss_segments(&mol)?;
        // Residue 0 = Helix; residues 1-2 = Coil → 2 segments.
        assert_eq!(segs.len(), 2);
        assert_eq!(segs[0].kind, SecondaryStructure::Helix);
        assert_eq!(segs[1].kind, SecondaryStructure::Coil);
        assert_eq!(segs[0].chain_id, 'A');
        Ok(())
    }

    #[test]
    fn ss_segment_direction_unit_vector() -> Result<(), LodError> {
        let mol = simple_protein(5, 0);
        let segs = compute_ss_segments(&mol)?;
        assert!(!segs.is_empty());
        let d = segs[0].direction;
        let len = (d[0] * d[0] + d[1] * d[1] + d[2] * d[2]).sqrt();
        // Must be a unit vector (length ≈ 1.0).
        assert!((len - 1.0).abs() < 1e-9, "direction len={len}");
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Domain blob computation
    // -----------------------------------------------------------------------

    #[test]
    fn domain_blob_computation() -> Result<(), LodError> {
        let mol = simple_protein(10, 0);
        // Segment size 5 → 2 blobs from 10 residues.
        let blobs = compute_domain_blobs(&mol, 5)?;
        assert_eq!(blobs.len(), 2);
        assert_eq!(blobs[0].chain_id, 'A');
        assert!(blobs[0].radius >= 0.0);
        assert_eq!(blobs[0].atom_count, 5);
        Ok(())
    }

    #[test]
    fn domain_blob_no_chains_error() {
        let mut mol = Molecule::new("ligand");
        mol.atoms.push(Atom::new(1, Element::C, [0.0; 3]));
        assert!(matches!(compute_domain_blobs(&mol, 5), Err(LodError::NoChains)));
    }

    // -----------------------------------------------------------------------
    // Convex hull of tetrahedron
    // -----------------------------------------------------------------------

    #[test]
    fn convex_hull_tetrahedron() {
        // Regular tetrahedron vertices.
        let pts = vec![
            [1.0_f64, 1.0, 1.0],
            [-1.0, -1.0, 1.0],
            [-1.0, 1.0, -1.0],
            [1.0, -1.0, -1.0],
        ];
        let hull = compute_convex_hull(&pts);
        assert_eq!(hull.vertices.len(), 4);
        // Gift-wrapping should produce triangles for a tetrahedron.
        assert!(
            !hull.triangles.is_empty(),
            "tetrahedron hull must have triangles, got 0"
        );
        assert!(
            hull.triangles.len() <= 8,
            "tetrahedron hull should not exceed 8 triangles, got {}",
            hull.triangles.len()
        );
        // All triangle indices must be valid.
        for tri in &hull.triangles {
            for &idx in tri {
                assert!(idx < hull.vertices.len());
            }
        }
    }

    #[test]
    fn convex_hull_too_few_points() {
        let pts = vec![[0.0_f64; 3], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]];
        let hull = compute_convex_hull(&pts);
        assert!(hull.triangles.is_empty());
    }

    // -----------------------------------------------------------------------
    // generate_lod at each level
    // -----------------------------------------------------------------------

    #[test]
    fn generate_lod_full_atom() -> Result<(), LodError> {
        let mol = simple_protein(3, 2);
        let atom_count = mol.atoms.len();
        let repr = generate_lod(&mol, LodLevel::FullAtom, &LodConfig::default())?;
        assert_eq!(repr.level, LodLevel::FullAtom);
        assert_eq!(repr.reduced_count, atom_count);
        Ok(())
    }

    #[test]
    fn generate_lod_calpha_trace() -> Result<(), LodError> {
        let mol = simple_protein(5, 3);
        let repr = generate_lod(&mol, LodLevel::CalphaTrace, &LodConfig::default())?;
        assert_eq!(repr.level, LodLevel::CalphaTrace);
        assert!(repr.calpha_trace.is_some());
        assert_eq!(repr.reduced_count, 8);
        Ok(())
    }

    #[test]
    fn generate_lod_secondary_structure() -> Result<(), LodError> {
        let mol = simple_protein(6, 4);
        let repr = generate_lod(&mol, LodLevel::SecondaryStructure, &LodConfig::default())?;
        assert_eq!(repr.level, LodLevel::SecondaryStructure);
        assert!(repr.ss_segments.is_some());
        Ok(())
    }

    #[test]
    fn generate_lod_domain_blob() -> Result<(), LodError> {
        let mol = simple_protein(10, 5);
        let repr = generate_lod(&mol, LodLevel::DomainBlob, &LodConfig::default())?;
        assert_eq!(repr.level, LodLevel::DomainBlob);
        assert!(repr.domain_blobs.is_some());
        Ok(())
    }

    #[test]
    fn generate_lod_convex_hull() -> Result<(), LodError> {
        let mol = simple_protein(8, 4);
        let repr = generate_lod(&mol, LodLevel::ConvexHull, &LodConfig::default())?;
        assert_eq!(repr.level, LodLevel::ConvexHull);
        assert!(repr.convex_hulls.is_some());
        Ok(())
    }

    // -----------------------------------------------------------------------
    // auto_lod selection
    // -----------------------------------------------------------------------

    #[test]
    fn auto_lod_selects_correct_level() -> Result<(), LodError> {
        // 10 atoms → FullAtom (no chain hierarchy needed at FullAtom).
        let mut mol = Molecule::new("tiny");
        for i in 0..10u32 {
            mol.atoms.push(Atom::new(i + 1, Element::C, [f64::from(i), 0.0, 0.0]));
        }
        let repr = auto_lod(&mol, &LodConfig::default())?;
        assert_eq!(repr.level, LodLevel::FullAtom);
        Ok(())
    }

    // -----------------------------------------------------------------------
    // Reduction ratio
    // -----------------------------------------------------------------------

    #[test]
    fn reduction_ratio_calculation() {
        let repr = LodRepresentation {
            level: LodLevel::CalphaTrace,
            calpha_trace: None,
            ss_segments: None,
            domain_blobs: None,
            convex_hulls: None,
            atom_count: 1_000,
            reduced_count: 200,
        };
        let ratio = reduction_ratio(&repr);
        assert!((ratio - 0.2).abs() < 1e-10, "ratio={ratio}");
    }

    #[test]
    fn reduction_ratio_zero_atom_count() {
        let repr = LodRepresentation {
            level: LodLevel::FullAtom,
            calpha_trace: None,
            ss_segments: None,
            domain_blobs: None,
            convex_hulls: None,
            atom_count: 0,
            reduced_count: 0,
        };
        assert!((reduction_ratio(&repr) - 1.0).abs() < 1e-10);
    }

    // -----------------------------------------------------------------------
    // LodConfig custom thresholds
    // -----------------------------------------------------------------------

    #[test]
    fn lod_config_custom_thresholds() {
        let cfg = LodConfig {
            thresholds: [100, 500, 2_000, 10_000],
            transition_blend: 0.05,
        };
        assert_eq!(cfg.level_for(50), LodLevel::FullAtom);
        assert_eq!(cfg.level_for(100), LodLevel::CalphaTrace);
        assert_eq!(cfg.level_for(499), LodLevel::CalphaTrace);
        assert_eq!(cfg.level_for(500), LodLevel::SecondaryStructure);
        assert_eq!(cfg.level_for(2_000), LodLevel::DomainBlob);
        assert_eq!(cfg.level_for(10_000), LodLevel::ConvexHull);
    }

    // -----------------------------------------------------------------------
    // Display
    // -----------------------------------------------------------------------

    #[test]
    fn lod_level_display() {
        assert_eq!(LodLevel::FullAtom.to_string(), "FullAtom");
        assert_eq!(LodLevel::CalphaTrace.to_string(), "CalphaTrace");
        assert_eq!(LodLevel::SecondaryStructure.to_string(), "SecondaryStructure");
        assert_eq!(LodLevel::DomainBlob.to_string(), "DomainBlob");
        assert_eq!(LodLevel::ConvexHull.to_string(), "ConvexHull");
    }

    // -----------------------------------------------------------------------
    // LodError Display
    // -----------------------------------------------------------------------

    #[test]
    fn lod_error_display() {
        let e = LodError::InvalidLevel {
            requested: "FullAtom".to_string(),
            atom_count: 99_999,
        };
        let s = e.to_string();
        assert!(s.contains("FullAtom"));
        assert!(s.contains("99999"));
    }
}
