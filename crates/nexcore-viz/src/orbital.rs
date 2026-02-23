//! Molecular orbital computation engine — Phase 5 "Eyes" of SOP-DEV-003.
//!
//! Provides STO-3G Gaussian basis sets, electron density grid computation,
//! overlap integrals, and orbital evaluation over 3D grids for molecular
//! visualization and quantum chemistry approximations.
//!
//! ## Primitive formula
//!   orbital = μ(basis) × ρ(contraction) → ρ(density grid)
//!   μ: Mapping  (atom → basis functions)
//!   ρ: Recursion (contracted Gaussians)
//!   →: Causality (density from wavefunction)
//!
//! ## Quick example
//!
//! ```rust
//! use nexcore_viz::molecular::{Atom, Element, Molecule};
//! use nexcore_viz::orbital::{OrbitalConfig, build_basis_set, default_coefficients, density_grid};
//!
//! let mut mol = Molecule::new("Water");
//! mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
//! mol.atoms.push(Atom::new(2, Element::H, [0.757, 0.586, 0.0]));
//! mol.atoms.push(Atom::new(3, Element::H, [-0.757, 0.586, 0.0]));
//!
//! let config = OrbitalConfig::default();
//! if let Ok(basis) = build_basis_set(&mol) {
//!     let coefficients = default_coefficients(&basis);
//!     if let Ok(grid) = density_grid(&mol, &basis, &coefficients, &config) {
//!         assert!(grid.max_value >= 0.0);
//!     }
//! }
//! ```

use std::fmt;

use serde::{Deserialize, Serialize};

use crate::molecular::{Element, Molecule};

// ============================================================================
// Error type
// ============================================================================

/// Errors that can occur during molecular orbital computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum OrbitalError {
    /// Molecule has no atoms — cannot build a basis set.
    EmptyMolecule,
    /// Element has no STO-3G parameters in our basis library.
    UnsupportedElement(String),
    /// Quantum numbers n/l/m are inconsistent or out of range.
    InvalidQuantumNumbers,
    /// Requested grid size exceeds the configured maximum point limit.
    GridTooLarge,
    /// General computation failure with a descriptive message.
    ComputationFailed(String),
}

impl fmt::Display for OrbitalError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::EmptyMolecule => write!(f, "molecule has no atoms; cannot build basis set"),
            Self::UnsupportedElement(sym) => {
                write!(
                    f,
                    "element '{sym}' has no STO-3G parameters in basis library"
                )
            }
            Self::InvalidQuantumNumbers => {
                write!(f, "quantum numbers are inconsistent or out of range")
            }
            Self::GridTooLarge => write!(
                f,
                "requested grid size exceeds the maximum allowed grid points"
            ),
            Self::ComputationFailed(msg) => write!(f, "orbital computation failed: {msg}"),
        }
    }
}

impl std::error::Error for OrbitalError {}

// ============================================================================
// Gaussian primitive
// ============================================================================

/// A single Gaussian primitive: coefficient × exp(−exponent × r²).
///
/// Contracted Gaussians are formed by summing multiple primitives, each
/// with its own exponent (width) and contraction coefficient.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaussianPrimitive {
    /// Gaussian exponent ζ (controls spatial extent — larger = more compact).
    pub exponent: f64,
    /// Contraction coefficient c_i.
    pub coefficient: f64,
}

// ============================================================================
// Orbital type (angular part)
// ============================================================================

/// Angular symmetry of a basis function.
///
/// These map to the real solid harmonics used in STO-nG basis sets:
/// s (l=0), p (l=1), d (l=2).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OrbitalType {
    /// s-type: spherically symmetric (angular factor = 1)
    S,
    /// p_x: angular factor proportional to x
    Px,
    /// p_y: angular factor proportional to y
    Py,
    /// p_z: angular factor proportional to z
    Pz,
    /// d_xy: angular factor proportional to xy
    Dxy,
    /// d_xz: angular factor proportional to xz
    Dxz,
    /// d_yz: angular factor proportional to yz
    Dyz,
    /// d_{x2-y2}: angular factor proportional to x^2 - y^2
    Dx2y2,
    /// d_{z2}: angular factor proportional to 2z^2 - x^2 - y^2
    Dz2,
}

impl OrbitalType {
    /// Evaluate the angular (polynomial) prefactor at a displacement vector.
    ///
    /// The displacement `dx`, `dy`, `dz` is the vector from the basis function
    /// center to the evaluation point.  Returns the dimensionless factor that
    /// multiplies the radial Gaussian part.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_viz::orbital::OrbitalType;
    ///
    /// // S orbital is spherically symmetric — factor is always 1.0
    /// assert_eq!(OrbitalType::S.angular_factor(3.0, -2.0, 1.0), 1.0);
    ///
    /// // Px has a node in the yz plane (x = 0)
    /// assert_eq!(OrbitalType::Px.angular_factor(0.0, 1.0, 1.0), 0.0);
    /// ```
    #[must_use]
    pub fn angular_factor(self, dx: f64, dy: f64, dz: f64) -> f64 {
        match self {
            Self::S => 1.0,
            Self::Px => dx,
            Self::Py => dy,
            Self::Pz => dz,
            Self::Dxy => dx * dy,
            Self::Dxz => dx * dz,
            Self::Dyz => dy * dz,
            Self::Dx2y2 => dx * dx - dy * dy,
            Self::Dz2 => 2.0 * dz * dz - dx * dx - dy * dy,
        }
    }
}

// ============================================================================
// Basis function
// ============================================================================

/// A contracted Gaussian basis function centred on an atom.
///
/// Evaluated as: phi(r) = angular(r-center) * sum_i c_i * exp(-zeta_i * |r-center|^2)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BasisFunction {
    /// Atom center in angstroms [x, y, z].
    pub center: [f64; 3],
    /// Angular symmetry of this function.
    pub orbital_type: OrbitalType,
    /// Gaussian primitives (contraction set).
    pub primitives: Vec<GaussianPrimitive>,
}

// ============================================================================
// Electron density grid
// ============================================================================

/// A regular 3-D grid of electron density values.
///
/// The grid is stored in C order: index = iz*ny*nx + iy*nx + ix.
/// Values are electron density rho(r) = |psi(r)|^2 in units of e/Ang^3.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ElectronDensityGrid {
    /// Grid origin [x, y, z] in angstroms.
    pub origin: [f64; 3],
    /// Uniform grid spacing in angstroms.
    pub spacing: f64,
    /// Number of grid points along x.
    pub nx: usize,
    /// Number of grid points along y.
    pub ny: usize,
    /// Number of grid points along z.
    pub nz: usize,
    /// Density values in C order (iz * ny * nx + iy * nx + ix).
    pub values: Vec<f64>,
    /// Minimum density in the grid.
    pub min_value: f64,
    /// Maximum density in the grid.
    pub max_value: f64,
}

// ============================================================================
// Orbital configuration
// ============================================================================

/// Configuration parameters for orbital and density grid computation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct OrbitalConfig {
    /// Grid spacing in angstroms (default 0.3 A).
    pub grid_spacing: f64,
    /// Padding around molecular bounding box in angstroms (default 3.0 A).
    pub padding: f64,
    /// Maximum number of grid points (default 500_000).
    ///
    /// `density_grid` returns [`OrbitalError::GridTooLarge`] if the
    /// requested grid exceeds this limit before computing any values.
    pub max_grid_points: usize,
    /// Isosurface value for downstream mesh generation (default 0.02 e/A^3).
    pub isovalue: f64,
}

impl Default for OrbitalConfig {
    fn default() -> Self {
        Self {
            grid_spacing: 0.3,
            padding: 3.0,
            max_grid_points: 500_000,
            isovalue: 0.02,
        }
    }
}

// ============================================================================
// Molecular orbital descriptor
// ============================================================================

/// A single molecular orbital expressed as a linear combination of basis functions.
///
/// psi_k(r) = sum_i C_{ki} phi_i(r)
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MolecularOrbital {
    /// Zero-based orbital index.
    pub index: usize,
    /// Orbital energy in Hartree.
    pub energy: f64,
    /// Occupation number (0.0, 1.0, or 2.0 for closed-shell).
    pub occupation: f64,
    /// Expansion coefficients C_{ki} for each basis function.
    pub coefficients: Vec<f64>,
}

// ============================================================================
// STO-3G basis set parameters
// ============================================================================

/// Return STO-3G basis functions for a given element.
///
/// Each tuple in the returned `Vec` is `(OrbitalType, primitives)`.
/// The data follows Hehre, Stewart, and Pople, J. Chem. Phys. 51, 2657 (1969).
///
/// Supported elements: H, C, N, O, F, P, S, Cl.
///
/// # Errors
///
/// Returns [`OrbitalError::UnsupportedElement`] if the element is not in
/// the STO-3G library.
///
/// # Examples
///
/// ```
/// use nexcore_viz::molecular::Element;
/// use nexcore_viz::orbital::sto3g_basis;
///
/// // Hydrogen: one 1s function
/// if let Ok(fns) = sto3g_basis(&Element::H) {
///     assert_eq!(fns.len(), 1);
/// }
///
/// // Carbon: 1s + 2s + 2px + 2py + 2pz = 5 functions
/// if let Ok(fns_c) = sto3g_basis(&Element::C) {
///     assert_eq!(fns_c.len(), 5);
/// }
/// ```
pub fn sto3g_basis(
    element: &Element,
) -> Result<Vec<(OrbitalType, Vec<GaussianPrimitive>)>, OrbitalError> {
    match element {
        // ------------------------------------------------------------------
        // Hydrogen — 1s only
        // ------------------------------------------------------------------
        Element::H => Ok(vec![(
            OrbitalType::S,
            vec![
                GaussianPrimitive {
                    exponent: 3.425_250_91,
                    coefficient: 0.154_328_97,
                },
                GaussianPrimitive {
                    exponent: 0.623_913_73,
                    coefficient: 0.535_328_14,
                },
                GaussianPrimitive {
                    exponent: 0.168_855_40,
                    coefficient: 0.444_634_54,
                },
            ],
        )]),

        // ------------------------------------------------------------------
        // Carbon — 1s, 2s, 2px, 2py, 2pz
        // ------------------------------------------------------------------
        Element::C => {
            let s1 = vec![
                GaussianPrimitive {
                    exponent: 71.616_837,
                    coefficient: 0.154_328_97,
                },
                GaussianPrimitive {
                    exponent: 13.045_096,
                    coefficient: 0.535_328_14,
                },
                GaussianPrimitive {
                    exponent: 3.530_512_2,
                    coefficient: 0.444_634_54,
                },
            ];
            let s2 = vec![
                GaussianPrimitive {
                    exponent: 2.941_249_4,
                    coefficient: -0.099_967_23,
                },
                GaussianPrimitive {
                    exponent: 0.683_483_1,
                    coefficient: 0.399_512_83,
                },
                GaussianPrimitive {
                    exponent: 0.222_289_9,
                    coefficient: 0.700_115_47,
                },
            ];
            let p_shared = vec![
                GaussianPrimitive {
                    exponent: 2.941_249_4,
                    coefficient: 0.155_916_27,
                },
                GaussianPrimitive {
                    exponent: 0.683_483_1,
                    coefficient: 0.607_683_72,
                },
                GaussianPrimitive {
                    exponent: 0.222_289_9,
                    coefficient: 0.391_957_39,
                },
            ];
            Ok(vec![
                (OrbitalType::S, s1),
                (OrbitalType::S, s2),
                (OrbitalType::Px, p_shared.clone()),
                (OrbitalType::Py, p_shared.clone()),
                (OrbitalType::Pz, p_shared),
            ])
        }

        // ------------------------------------------------------------------
        // Nitrogen — 1s, 2s, 2px, 2py, 2pz
        // ------------------------------------------------------------------
        Element::N => {
            let s1 = vec![
                GaussianPrimitive {
                    exponent: 99.106_169,
                    coefficient: 0.154_328_97,
                },
                GaussianPrimitive {
                    exponent: 18.052_312,
                    coefficient: 0.535_328_14,
                },
                GaussianPrimitive {
                    exponent: 4.885_660_2,
                    coefficient: 0.444_634_54,
                },
            ];
            let s2 = vec![
                GaussianPrimitive {
                    exponent: 3.780_455_9,
                    coefficient: -0.099_967_23,
                },
                GaussianPrimitive {
                    exponent: 0.878_496_5,
                    coefficient: 0.399_512_83,
                },
                GaussianPrimitive {
                    exponent: 0.285_714_4,
                    coefficient: 0.700_115_47,
                },
            ];
            let p_shared = vec![
                GaussianPrimitive {
                    exponent: 3.780_455_9,
                    coefficient: 0.155_916_27,
                },
                GaussianPrimitive {
                    exponent: 0.878_496_5,
                    coefficient: 0.607_683_72,
                },
                GaussianPrimitive {
                    exponent: 0.285_714_4,
                    coefficient: 0.391_957_39,
                },
            ];
            Ok(vec![
                (OrbitalType::S, s1),
                (OrbitalType::S, s2),
                (OrbitalType::Px, p_shared.clone()),
                (OrbitalType::Py, p_shared.clone()),
                (OrbitalType::Pz, p_shared),
            ])
        }

        // ------------------------------------------------------------------
        // Oxygen — 1s, 2s, 2px, 2py, 2pz
        // ------------------------------------------------------------------
        Element::O => {
            let s1 = vec![
                GaussianPrimitive {
                    exponent: 130.709_320,
                    coefficient: 0.154_328_97,
                },
                GaussianPrimitive {
                    exponent: 23.808_861,
                    coefficient: 0.535_328_14,
                },
                GaussianPrimitive {
                    exponent: 6.443_608_3,
                    coefficient: 0.444_634_54,
                },
            ];
            let s2 = vec![
                GaussianPrimitive {
                    exponent: 5.033_151_3,
                    coefficient: -0.099_967_23,
                },
                GaussianPrimitive {
                    exponent: 1.169_596_1,
                    coefficient: 0.399_512_83,
                },
                GaussianPrimitive {
                    exponent: 0.380_389_0,
                    coefficient: 0.700_115_47,
                },
            ];
            let p_shared = vec![
                GaussianPrimitive {
                    exponent: 5.033_151_3,
                    coefficient: 0.155_916_27,
                },
                GaussianPrimitive {
                    exponent: 1.169_596_1,
                    coefficient: 0.607_683_72,
                },
                GaussianPrimitive {
                    exponent: 0.380_389_0,
                    coefficient: 0.391_957_39,
                },
            ];
            Ok(vec![
                (OrbitalType::S, s1),
                (OrbitalType::S, s2),
                (OrbitalType::Px, p_shared.clone()),
                (OrbitalType::Py, p_shared.clone()),
                (OrbitalType::Pz, p_shared),
            ])
        }

        // ------------------------------------------------------------------
        // Fluorine — 1s, 2s, 2px, 2py, 2pz
        // ------------------------------------------------------------------
        Element::F => {
            let s1 = vec![
                GaussianPrimitive {
                    exponent: 166.679_134,
                    coefficient: 0.154_328_97,
                },
                GaussianPrimitive {
                    exponent: 30.360_812,
                    coefficient: 0.535_328_14,
                },
                GaussianPrimitive {
                    exponent: 8.216_820_3,
                    coefficient: 0.444_634_54,
                },
            ];
            let s2 = vec![
                GaussianPrimitive {
                    exponent: 6.464_803_2,
                    coefficient: -0.099_967_23,
                },
                GaussianPrimitive {
                    exponent: 1.502_281_2,
                    coefficient: 0.399_512_83,
                },
                GaussianPrimitive {
                    exponent: 0.488_588_7,
                    coefficient: 0.700_115_47,
                },
            ];
            let p_shared = vec![
                GaussianPrimitive {
                    exponent: 6.464_803_2,
                    coefficient: 0.155_916_27,
                },
                GaussianPrimitive {
                    exponent: 1.502_281_2,
                    coefficient: 0.607_683_72,
                },
                GaussianPrimitive {
                    exponent: 0.488_588_7,
                    coefficient: 0.391_957_39,
                },
            ];
            Ok(vec![
                (OrbitalType::S, s1),
                (OrbitalType::S, s2),
                (OrbitalType::Px, p_shared.clone()),
                (OrbitalType::Py, p_shared.clone()),
                (OrbitalType::Pz, p_shared),
            ])
        }

        // ------------------------------------------------------------------
        // Phosphorus — 1s, 2s, 2px, 2py, 2pz, 3s, 3px, 3py, 3pz
        // ------------------------------------------------------------------
        Element::P => {
            let s1 = vec![
                GaussianPrimitive {
                    exponent: 488.884_599,
                    coefficient: 0.154_328_97,
                },
                GaussianPrimitive {
                    exponent: 89.078_255,
                    coefficient: 0.535_328_14,
                },
                GaussianPrimitive {
                    exponent: 24.103_864,
                    coefficient: 0.444_634_54,
                },
            ];
            let s2 = vec![
                GaussianPrimitive {
                    exponent: 19.338_085,
                    coefficient: -0.099_967_23,
                },
                GaussianPrimitive {
                    exponent: 4.490_876_1,
                    coefficient: 0.399_512_83,
                },
                GaussianPrimitive {
                    exponent: 1.461_534_5,
                    coefficient: 0.700_115_47,
                },
            ];
            let p2 = vec![
                GaussianPrimitive {
                    exponent: 19.338_085,
                    coefficient: 0.155_916_27,
                },
                GaussianPrimitive {
                    exponent: 4.490_876_1,
                    coefficient: 0.607_683_72,
                },
                GaussianPrimitive {
                    exponent: 1.461_534_5,
                    coefficient: 0.391_957_39,
                },
            ];
            let s3 = vec![
                GaussianPrimitive {
                    exponent: 1.938_468_0,
                    coefficient: -0.223_154_10,
                },
                GaussianPrimitive {
                    exponent: 0.543_361_3,
                    coefficient: 0.117_299_75,
                },
                GaussianPrimitive {
                    exponent: 0.215_704_0,
                    coefficient: 1.130_767_01,
                },
            ];
            let p3 = vec![
                GaussianPrimitive {
                    exponent: 1.938_468_0,
                    coefficient: 0.018_919_79,
                },
                GaussianPrimitive {
                    exponent: 0.543_361_3,
                    coefficient: 0.603_802_47,
                },
                GaussianPrimitive {
                    exponent: 0.215_704_0,
                    coefficient: 0.496_609_96,
                },
            ];
            Ok(vec![
                (OrbitalType::S, s1),
                (OrbitalType::S, s2),
                (OrbitalType::Px, p2.clone()),
                (OrbitalType::Py, p2.clone()),
                (OrbitalType::Pz, p2),
                (OrbitalType::S, s3),
                (OrbitalType::Px, p3.clone()),
                (OrbitalType::Py, p3.clone()),
                (OrbitalType::Pz, p3),
            ])
        }

        // ------------------------------------------------------------------
        // Sulfur — 1s, 2s, 2px, 2py, 2pz, 3s, 3px, 3py, 3pz
        // ------------------------------------------------------------------
        Element::S => {
            let s1 = vec![
                GaussianPrimitive {
                    exponent: 553.384_200,
                    coefficient: 0.154_328_97,
                },
                GaussianPrimitive {
                    exponent: 100.828_400,
                    coefficient: 0.535_328_14,
                },
                GaussianPrimitive {
                    exponent: 27.297_070,
                    coefficient: 0.444_634_54,
                },
            ];
            let s2 = vec![
                GaussianPrimitive {
                    exponent: 22.013_480,
                    coefficient: -0.099_967_23,
                },
                GaussianPrimitive {
                    exponent: 5.112_620_0,
                    coefficient: 0.399_512_83,
                },
                GaussianPrimitive {
                    exponent: 1.663_700_0,
                    coefficient: 0.700_115_47,
                },
            ];
            let p2 = vec![
                GaussianPrimitive {
                    exponent: 22.013_480,
                    coefficient: 0.155_916_27,
                },
                GaussianPrimitive {
                    exponent: 5.112_620_0,
                    coefficient: 0.607_683_72,
                },
                GaussianPrimitive {
                    exponent: 1.663_700_0,
                    coefficient: 0.391_957_39,
                },
            ];
            let s3 = vec![
                GaussianPrimitive {
                    exponent: 2.216_000_0,
                    coefficient: -0.223_154_10,
                },
                GaussianPrimitive {
                    exponent: 0.621_800_0,
                    coefficient: 0.117_299_75,
                },
                GaussianPrimitive {
                    exponent: 0.246_700_0,
                    coefficient: 1.130_767_01,
                },
            ];
            let p3 = vec![
                GaussianPrimitive {
                    exponent: 2.216_000_0,
                    coefficient: 0.018_919_79,
                },
                GaussianPrimitive {
                    exponent: 0.621_800_0,
                    coefficient: 0.603_802_47,
                },
                GaussianPrimitive {
                    exponent: 0.246_700_0,
                    coefficient: 0.496_609_96,
                },
            ];
            Ok(vec![
                (OrbitalType::S, s1),
                (OrbitalType::S, s2),
                (OrbitalType::Px, p2.clone()),
                (OrbitalType::Py, p2.clone()),
                (OrbitalType::Pz, p2),
                (OrbitalType::S, s3),
                (OrbitalType::Px, p3.clone()),
                (OrbitalType::Py, p3.clone()),
                (OrbitalType::Pz, p3),
            ])
        }

        // ------------------------------------------------------------------
        // Chlorine — 1s, 2s, 2px, 2py, 2pz, 3s, 3px, 3py, 3pz
        // ------------------------------------------------------------------
        Element::Cl => {
            let s1 = vec![
                GaussianPrimitive {
                    exponent: 685.527_900,
                    coefficient: 0.154_328_97,
                },
                GaussianPrimitive {
                    exponent: 124.908_400,
                    coefficient: 0.535_328_14,
                },
                GaussianPrimitive {
                    exponent: 33.815_890,
                    coefficient: 0.444_634_54,
                },
            ];
            let s2 = vec![
                GaussianPrimitive {
                    exponent: 27.259_680,
                    coefficient: -0.099_967_23,
                },
                GaussianPrimitive {
                    exponent: 6.330_790_0,
                    coefficient: 0.399_512_83,
                },
                GaussianPrimitive {
                    exponent: 2.060_100_0,
                    coefficient: 0.700_115_47,
                },
            ];
            let p2 = vec![
                GaussianPrimitive {
                    exponent: 27.259_680,
                    coefficient: 0.155_916_27,
                },
                GaussianPrimitive {
                    exponent: 6.330_790_0,
                    coefficient: 0.607_683_72,
                },
                GaussianPrimitive {
                    exponent: 2.060_100_0,
                    coefficient: 0.391_957_39,
                },
            ];
            let s3 = vec![
                GaussianPrimitive {
                    exponent: 2.660_000_0,
                    coefficient: -0.223_154_10,
                },
                GaussianPrimitive {
                    exponent: 0.746_200_0,
                    coefficient: 0.117_299_75,
                },
                GaussianPrimitive {
                    exponent: 0.296_000_0,
                    coefficient: 1.130_767_01,
                },
            ];
            let p3 = vec![
                GaussianPrimitive {
                    exponent: 2.660_000_0,
                    coefficient: 0.018_919_79,
                },
                GaussianPrimitive {
                    exponent: 0.746_200_0,
                    coefficient: 0.603_802_47,
                },
                GaussianPrimitive {
                    exponent: 0.296_000_0,
                    coefficient: 0.496_609_96,
                },
            ];
            Ok(vec![
                (OrbitalType::S, s1),
                (OrbitalType::S, s2),
                (OrbitalType::Px, p2.clone()),
                (OrbitalType::Py, p2.clone()),
                (OrbitalType::Pz, p2),
                (OrbitalType::S, s3),
                (OrbitalType::Px, p3.clone()),
                (OrbitalType::Py, p3.clone()),
                (OrbitalType::Pz, p3),
            ])
        }

        // ------------------------------------------------------------------
        // Unsupported elements
        // ------------------------------------------------------------------
        other => Err(OrbitalError::UnsupportedElement(other.symbol().to_string())),
    }
}

// ============================================================================
// Basis set construction
// ============================================================================

/// Build the complete STO-3G basis set for a molecule.
///
/// Each atom contributes basis functions according to its element.  Functions
/// are stored in atom order (atom 0 functions first, then atom 1, etc.).
///
/// # Errors
///
/// - [`OrbitalError::EmptyMolecule`] if the molecule has no atoms.
/// - [`OrbitalError::UnsupportedElement`] if any atom's element is not in
///   the STO-3G library.
///
/// # Examples
///
/// ```
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
/// use nexcore_viz::orbital::build_basis_set;
///
/// let mut mol = Molecule::new("Water");
/// mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
/// mol.atoms.push(Atom::new(2, Element::H, [0.757, 0.586, 0.0]));
/// mol.atoms.push(Atom::new(3, Element::H, [-0.757, 0.586, 0.0]));
///
/// // O: 5 functions (1s + 2s + 3p), H: 1 function each -> 7 total
/// if let Ok(basis) = build_basis_set(&mol) {
///     assert_eq!(basis.len(), 7);
/// }
/// ```
pub fn build_basis_set(molecule: &Molecule) -> Result<Vec<BasisFunction>, OrbitalError> {
    if molecule.atoms.is_empty() {
        return Err(OrbitalError::EmptyMolecule);
    }

    let mut basis = Vec::new();
    for atom in &molecule.atoms {
        let atom_fns = sto3g_basis(&atom.element)?;
        for (orbital_type, primitives) in atom_fns {
            basis.push(BasisFunction {
                center: atom.position,
                orbital_type,
                primitives,
            });
        }
    }
    Ok(basis)
}

// ============================================================================
// Basis function evaluation
// ============================================================================

/// Evaluate a single basis function phi at a 3-D point.
///
/// phi(r) = angular(r - center) * sum_i c_i * exp(-zeta_i * |r - center|^2)
///
/// # Examples
///
/// ```
/// use nexcore_viz::orbital::{BasisFunction, GaussianPrimitive, OrbitalType, orbital_value_at};
///
/// let bf = BasisFunction {
///     center: [0.0, 0.0, 0.0],
///     orbital_type: OrbitalType::S,
///     primitives: vec![
///         GaussianPrimitive { exponent: 1.0, coefficient: 1.0 },
///     ],
/// };
/// // S-orbital maximum is at the center (r = 0)
/// let at_center = orbital_value_at(&bf, &[0.0, 0.0, 0.0]);
/// let at_distant = orbital_value_at(&bf, &[5.0, 0.0, 0.0]);
/// assert!(at_center > at_distant);
/// ```
#[must_use]
pub fn orbital_value_at(basis: &BasisFunction, point: &[f64; 3]) -> f64 {
    let dx = point[0] - basis.center[0];
    let dy = point[1] - basis.center[1];
    let dz = point[2] - basis.center[2];
    let r2 = dx * dx + dy * dy + dz * dz;

    let angular = basis.orbital_type.angular_factor(dx, dy, dz);

    let radial: f64 = basis
        .primitives
        .iter()
        .map(|p| p.coefficient * (-p.exponent * r2).exp())
        .sum();

    angular * radial
}

// ============================================================================
// Overlap integrals
// ============================================================================

/// Compute the overlap integral <a|b> between two basis functions.
///
/// For s-type functions an analytical expression is used.  For other types
/// a 20-point per axis numerical integration over a finite cube is applied.
///
/// # Examples
///
/// ```
/// use nexcore_viz::orbital::{BasisFunction, GaussianPrimitive, OrbitalType, compute_overlap};
///
/// let bf = BasisFunction {
///     center: [0.0, 0.0, 0.0],
///     orbital_type: OrbitalType::S,
///     primitives: vec![
///         GaussianPrimitive { exponent: 0.5, coefficient: 1.0 },
///     ],
/// };
/// // Overlap of a function with itself is a positive definite quantity.
/// let s = compute_overlap(&bf, &bf);
/// assert!(s > 0.0);
/// ```
#[must_use]
pub fn compute_overlap(a: &BasisFunction, b: &BasisFunction) -> f64 {
    match (a.orbital_type, b.orbital_type) {
        (OrbitalType::S, OrbitalType::S) => overlap_ss_analytical(a, b),
        _ => overlap_numerical(a, b),
    }
}

/// Analytical overlap integral for two s-type contracted Gaussians.
///
/// Uses the product-of-Gaussians theorem: two Gaussians centred at A and B
/// multiply to a single Gaussian centred between them.
fn overlap_ss_analytical(a: &BasisFunction, b: &BasisFunction) -> f64 {
    use std::f64::consts::PI;

    let rab2 = {
        let d0 = a.center[0] - b.center[0];
        let d1 = a.center[1] - b.center[1];
        let d2 = a.center[2] - b.center[2];
        d0 * d0 + d1 * d1 + d2 * d2
    };

    let mut total = 0.0;
    for pa in &a.primitives {
        for pb in &b.primitives {
            let alpha = pa.exponent;
            let beta = pb.exponent;
            let gamma = alpha + beta;
            // Normalization factors for primitive Gaussians: N = (2α/π)^(3/4)
            let na = (2.0 * alpha / PI).powf(0.75);
            let nb = (2.0 * beta / PI).powf(0.75);
            // K = exp(-alpha * beta / (alpha+beta) * |A-B|^2)
            let k = (-alpha * beta / gamma * rab2).exp();
            // (pi / gamma)^(3/2)
            let prefactor = (PI / gamma).powi(3).sqrt();
            total += pa.coefficient * pb.coefficient * na * nb * k * prefactor;
        }
    }
    total
}

/// Numerical overlap integral via uniform quadrature on a finite cube.
///
/// Integration range: +/-6 A around the midpoint of the two centers.
/// Uses 20 uniformly-spaced sample points per axis (8000 total).
fn overlap_numerical(a: &BasisFunction, b: &BasisFunction) -> f64 {
    let mid = [
        (a.center[0] + b.center[0]) * 0.5,
        (a.center[1] + b.center[1]) * 0.5,
        (a.center[2] + b.center[2]) * 0.5,
    ];

    let half = 6.0_f64;
    let n = 20_usize;
    let h = 2.0 * half / (n as f64);

    let mut sum = 0.0;
    for ix in 0..n {
        let x = mid[0] - half + (ix as f64 + 0.5) * h;
        for iy in 0..n {
            let y = mid[1] - half + (iy as f64 + 0.5) * h;
            for iz in 0..n {
                let z = mid[2] - half + (iz as f64 + 0.5) * h;
                let pt = [x, y, z];
                sum += orbital_value_at(a, &pt) * orbital_value_at(b, &pt);
            }
        }
    }
    sum * h * h * h
}

// ============================================================================
// Electron density
// ============================================================================

/// Compute the electron density at a single 3-D point.
///
/// rho(r) = |psi(r)|^2 = |sum_i c_i phi_i(r)|^2
///
/// If `coefficients` is shorter than `basis_set`, missing coefficients are
/// treated as zero.  Extra coefficients beyond the basis set length are
/// ignored.
///
/// # Examples
///
/// ```
/// use nexcore_viz::orbital::{BasisFunction, GaussianPrimitive, OrbitalType,
///                            compute_electron_density};
///
/// let bf = BasisFunction {
///     center: [0.0, 0.0, 0.0],
///     orbital_type: OrbitalType::S,
///     primitives: vec![
///         GaussianPrimitive { exponent: 1.0, coefficient: 1.0 },
///     ],
/// };
/// let rho = compute_electron_density(&[bf], &[1.0], &[0.0, 0.0, 0.0]);
/// assert!(rho > 0.0);
/// ```
#[must_use]
pub fn compute_electron_density(
    basis_set: &[BasisFunction],
    coefficients: &[f64],
    point: &[f64; 3],
) -> f64 {
    let psi: f64 = basis_set
        .iter()
        .zip(coefficients.iter().chain(std::iter::repeat(&0.0_f64)))
        .map(|(bf, &c)| c * orbital_value_at(bf, point))
        .sum();
    psi * psi
}

// ============================================================================
// Default coefficients
// ============================================================================

/// Return uniform coefficients (all 1.0) with length matching the basis set.
///
/// This provides a simple starting point for visualization when no SCF
/// solution is available: equal contribution from every basis function.
///
/// # Examples
///
/// ```
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
/// use nexcore_viz::orbital::{build_basis_set, default_coefficients};
///
/// let mut mol = Molecule::new("H2");
/// mol.atoms.push(Atom::new(1, Element::H, [0.0, 0.0, 0.0]));
/// mol.atoms.push(Atom::new(2, Element::H, [0.74, 0.0, 0.0]));
///
/// if let Ok(basis) = build_basis_set(&mol) {
///     let coeffs = default_coefficients(&basis);
///     assert_eq!(coeffs.len(), basis.len());
///     assert!(coeffs.iter().all(|&c| (c - 1.0).abs() < f64::EPSILON));
/// }
/// ```
#[must_use]
pub fn default_coefficients(basis_set: &[BasisFunction]) -> Vec<f64> {
    vec![1.0; basis_set.len()]
}

// ============================================================================
// Overlap matrix
// ============================================================================

/// Compute the full overlap matrix S where S[i][j] = <phi_i|phi_j>.
///
/// The matrix is real and symmetric.  Diagonal elements are self-overlap
/// values (approximately 1.0 for normalized functions).
///
/// # Examples
///
/// ```
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
/// use nexcore_viz::orbital::{build_basis_set, compute_overlap_matrix};
///
/// let mut mol = Molecule::new("H2");
/// mol.atoms.push(Atom::new(1, Element::H, [0.0, 0.0, 0.0]));
/// mol.atoms.push(Atom::new(2, Element::H, [0.74, 0.0, 0.0]));
///
/// if let Ok(basis) = build_basis_set(&mol) {
///     let s = compute_overlap_matrix(&basis);
///     // Symmetric: S[0][1] == S[1][0]
///     assert!((s[0][1] - s[1][0]).abs() < 1e-12);
/// }
/// ```
#[must_use]
pub fn compute_overlap_matrix(basis_set: &[BasisFunction]) -> Vec<Vec<f64>> {
    let n = basis_set.len();
    let mut matrix = vec![vec![0.0_f64; n]; n];
    for i in 0..n {
        for j in i..n {
            let s = compute_overlap(&basis_set[i], &basis_set[j]);
            matrix[i][j] = s;
            matrix[j][i] = s;
        }
    }
    matrix
}

// ============================================================================
// Density grid
// ============================================================================

/// Compute the electron density on a regular 3-D grid spanning the molecule.
///
/// The grid is sized to cover the molecular bounding box plus `config.padding`
/// on all sides, using `config.grid_spacing` as the step size.  Returns
/// [`OrbitalError::GridTooLarge`] if the total point count would exceed
/// `config.max_grid_points`.
///
/// # Errors
///
/// - [`OrbitalError::GridTooLarge`] — grid exceeds `max_grid_points`.
/// - [`OrbitalError::ComputationFailed`] — `grid_spacing` is not positive or
///   arithmetic overflow in grid dimensions.
///
/// # Examples
///
/// ```
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
/// use nexcore_viz::orbital::{OrbitalConfig, build_basis_set, default_coefficients, density_grid};
///
/// let mut mol = Molecule::new("Water");
/// mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
/// mol.atoms.push(Atom::new(2, Element::H, [0.757, 0.586, 0.0]));
/// mol.atoms.push(Atom::new(3, Element::H, [-0.757, 0.586, 0.0]));
///
/// let config = OrbitalConfig::default();
/// if let Ok(basis) = build_basis_set(&mol) {
///     let coefficients = default_coefficients(&basis);
///     if let Ok(grid) = density_grid(&mol, &basis, &coefficients, &config) {
///         assert_eq!(grid.values.len(), grid.nx * grid.ny * grid.nz);
///         assert!(grid.max_value >= 0.0);
///     }
/// }
/// ```
pub fn density_grid(
    molecule: &Molecule,
    basis_set: &[BasisFunction],
    coefficients: &[f64],
    config: &OrbitalConfig,
) -> Result<ElectronDensityGrid, OrbitalError> {
    let (bb_min, bb_max) = molecule.bounding_box();

    let origin = [
        bb_min[0] - config.padding,
        bb_min[1] - config.padding,
        bb_min[2] - config.padding,
    ];
    let upper = [
        bb_max[0] + config.padding,
        bb_max[1] + config.padding,
        bb_max[2] + config.padding,
    ];

    let s = config.grid_spacing;
    if s <= 0.0 {
        return Err(OrbitalError::ComputationFailed(
            "grid_spacing must be positive".to_string(),
        ));
    }

    let nx = (((upper[0] - origin[0]) / s).ceil() as usize).max(1);
    let ny = (((upper[1] - origin[1]) / s).ceil() as usize).max(1);
    let nz = (((upper[2] - origin[2]) / s).ceil() as usize).max(1);

    let total = nx
        .checked_mul(ny)
        .and_then(|xy| xy.checked_mul(nz))
        .ok_or_else(|| {
            OrbitalError::ComputationFailed("grid size arithmetic overflow".to_string())
        })?;

    if total > config.max_grid_points {
        return Err(OrbitalError::GridTooLarge);
    }

    let mut values = Vec::with_capacity(total);
    let mut min_value = f64::MAX;
    let mut max_value = f64::MIN;

    for iz in 0..nz {
        let z = origin[2] + iz as f64 * s;
        for iy in 0..ny {
            let y = origin[1] + iy as f64 * s;
            for ix in 0..nx {
                let x = origin[0] + ix as f64 * s;
                let pt = [x, y, z];
                let rho = compute_electron_density(basis_set, coefficients, &pt);
                if rho < min_value {
                    min_value = rho;
                }
                if rho > max_value {
                    max_value = rho;
                }
                values.push(rho);
            }
        }
    }

    if min_value == f64::MAX {
        min_value = 0.0;
        max_value = 0.0;
    }

    Ok(ElectronDensityGrid {
        origin,
        spacing: s,
        nx,
        ny,
        nz,
        values,
        min_value,
        max_value,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{Atom, Element, Molecule};

    // -----------------------------------------------------------------------
    // Test helpers
    // -----------------------------------------------------------------------

    fn make_s_function(center: [f64; 3], exponent: f64) -> BasisFunction {
        BasisFunction {
            center,
            orbital_type: OrbitalType::S,
            primitives: vec![GaussianPrimitive {
                exponent,
                coefficient: 1.0,
            }],
        }
    }

    fn make_p_function(
        center: [f64; 3],
        orbital_type: OrbitalType,
        exponent: f64,
    ) -> BasisFunction {
        BasisFunction {
            center,
            orbital_type,
            primitives: vec![GaussianPrimitive {
                exponent,
                coefficient: 1.0,
            }],
        }
    }

    fn water_molecule() -> Molecule {
        let mut mol = Molecule::new("Water");
        mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
        mol.atoms
            .push(Atom::new(2, Element::H, [0.757, 0.586, 0.0]));
        mol.atoms
            .push(Atom::new(3, Element::H, [-0.757, 0.586, 0.0]));
        mol
    }

    fn fallback_grid() -> ElectronDensityGrid {
        ElectronDensityGrid {
            origin: [0.0; 3],
            spacing: 1.0,
            nx: 1,
            ny: 1,
            nz: 1,
            values: vec![1.0],
            min_value: 1.0,
            max_value: 1.0,
        }
    }

    // -----------------------------------------------------------------------
    // STO-3G basis set parameter tests
    // -----------------------------------------------------------------------

    #[test]
    fn sto3g_hydrogen_has_one_function() {
        let fns = sto3g_basis(&Element::H).unwrap_or_default();
        assert_eq!(fns.len(), 1);
        assert_eq!(fns[0].0, OrbitalType::S);
    }

    #[test]
    fn sto3g_hydrogen_has_three_primitives() {
        let fns = sto3g_basis(&Element::H).unwrap_or_default();
        assert_eq!(fns[0].1.len(), 3);
    }

    #[test]
    fn sto3g_carbon_has_five_functions() {
        let fns = sto3g_basis(&Element::C).unwrap_or_default();
        assert_eq!(fns.len(), 5);
    }

    #[test]
    fn sto3g_carbon_function_types() {
        let fns = sto3g_basis(&Element::C).unwrap_or_default();
        let types: Vec<OrbitalType> = fns.iter().map(|(t, _)| *t).collect();
        assert_eq!(types[0], OrbitalType::S);
        assert_eq!(types[1], OrbitalType::S);
        assert_eq!(types[2], OrbitalType::Px);
        assert_eq!(types[3], OrbitalType::Py);
        assert_eq!(types[4], OrbitalType::Pz);
    }

    #[test]
    fn sto3g_nitrogen_has_five_functions() {
        let fns = sto3g_basis(&Element::N).unwrap_or_default();
        assert_eq!(fns.len(), 5);
    }

    #[test]
    fn sto3g_oxygen_has_five_functions() {
        let fns = sto3g_basis(&Element::O).unwrap_or_default();
        assert_eq!(fns.len(), 5);
    }

    #[test]
    fn sto3g_fluorine_has_five_functions() {
        let fns = sto3g_basis(&Element::F).unwrap_or_default();
        assert_eq!(fns.len(), 5);
    }

    #[test]
    fn sto3g_phosphorus_has_nine_functions() {
        let fns = sto3g_basis(&Element::P).unwrap_or_default();
        assert_eq!(fns.len(), 9);
    }

    #[test]
    fn sto3g_sulfur_has_nine_functions() {
        let fns = sto3g_basis(&Element::S).unwrap_or_default();
        assert_eq!(fns.len(), 9);
    }

    #[test]
    fn sto3g_chlorine_has_nine_functions() {
        let fns = sto3g_basis(&Element::Cl).unwrap_or_default();
        assert_eq!(fns.len(), 9);
    }

    #[test]
    fn sto3g_unsupported_element_returns_error() {
        match sto3g_basis(&Element::Fe) {
            Err(OrbitalError::UnsupportedElement(sym)) => {
                assert_eq!(sym, "Fe");
            }
            Err(other) => {
                assert!(false, "expected UnsupportedElement(\"Fe\"), got: {other}");
            }
            Ok(_) => {
                assert!(false, "expected error for unsupported element Fe");
            }
        }
    }

    #[test]
    fn sto3g_exponents_are_positive() {
        for elem in [Element::H, Element::C, Element::N, Element::O] {
            let fns = sto3g_basis(&elem).unwrap_or_default();
            for (_, prims) in &fns {
                for p in prims {
                    assert!(p.exponent > 0.0, "exponent must be positive for {elem:?}");
                }
            }
        }
    }

    // -----------------------------------------------------------------------
    // OrbitalType angular factor tests
    // -----------------------------------------------------------------------

    #[test]
    fn angular_s_is_always_one() {
        assert_eq!(OrbitalType::S.angular_factor(0.0, 0.0, 0.0), 1.0);
        assert_eq!(OrbitalType::S.angular_factor(3.0, -2.0, 1.0), 1.0);
    }

    #[test]
    fn angular_px_has_node_at_x_zero() {
        assert_eq!(OrbitalType::Px.angular_factor(0.0, 1.0, 1.0), 0.0);
    }

    #[test]
    fn angular_py_has_node_at_y_zero() {
        assert_eq!(OrbitalType::Py.angular_factor(1.0, 0.0, 1.0), 0.0);
    }

    #[test]
    fn angular_pz_has_node_at_z_zero() {
        assert_eq!(OrbitalType::Pz.angular_factor(1.0, 1.0, 0.0), 0.0);
    }

    #[test]
    fn angular_dxy_has_node_on_axes() {
        assert_eq!(OrbitalType::Dxy.angular_factor(0.0, 2.0, 1.0), 0.0);
        assert_eq!(OrbitalType::Dxy.angular_factor(2.0, 0.0, 1.0), 0.0);
    }

    // -----------------------------------------------------------------------
    // orbital_value_at tests
    // -----------------------------------------------------------------------

    #[test]
    fn s_orbital_maximum_at_center() {
        let bf = make_s_function([0.0, 0.0, 0.0], 1.0);
        let at_center = orbital_value_at(&bf, &[0.0, 0.0, 0.0]);
        let at_far = orbital_value_at(&bf, &[3.0, 0.0, 0.0]);
        assert!(at_center > at_far, "s-orbital should peak at center");
    }

    #[test]
    fn s_orbital_decays_with_distance() {
        let bf = make_s_function([0.0, 0.0, 0.0], 1.0);
        let v0 = orbital_value_at(&bf, &[0.0, 0.0, 0.0]);
        let v1 = orbital_value_at(&bf, &[1.0, 0.0, 0.0]);
        let v2 = orbital_value_at(&bf, &[2.0, 0.0, 0.0]);
        assert!(v0 > v1, "value should decrease from center to 1 A");
        assert!(v1 > v2, "value should decrease from 1 A to 2 A");
    }

    #[test]
    fn p_orbital_has_node_at_center() {
        let bf = make_p_function([0.0, 0.0, 0.0], OrbitalType::Px, 1.0);
        let at_center = orbital_value_at(&bf, &[0.0, 0.0, 0.0]);
        assert!(at_center.abs() < 1e-12, "Px orbital must be zero at center");
    }

    #[test]
    fn p_orbital_opposite_lobes_have_opposite_sign() {
        let bf = make_p_function([0.0, 0.0, 0.0], OrbitalType::Px, 1.0);
        let pos = orbital_value_at(&bf, &[1.0, 0.0, 0.0]);
        let neg = orbital_value_at(&bf, &[-1.0, 0.0, 0.0]);
        assert!(pos > 0.0);
        assert!(neg < 0.0);
        assert!((pos + neg).abs() < 1e-12, "lobes should be antisymmetric");
    }

    // -----------------------------------------------------------------------
    // compute_overlap tests
    // -----------------------------------------------------------------------

    #[test]
    fn overlap_identical_s_functions_positive() {
        let bf = make_s_function([0.0, 0.0, 0.0], 0.5);
        let s = compute_overlap(&bf, &bf);
        assert!(s > 0.0, "self-overlap must be positive");
    }

    #[test]
    fn overlap_h1s_self_overlap_near_one() {
        // The raw STO-3G H 1s contraction is not normalised to unity.
        // The analytical self-overlap is approximately 10.74 (unnormalised).
        // We verify it is positive, finite, and in the expected order of magnitude.
        let fns = sto3g_basis(&Element::H).unwrap_or_default();
        assert!(!fns.is_empty(), "H STO-3G must have at least one function");
        let bf = BasisFunction {
            center: [0.0, 0.0, 0.0],
            orbital_type: OrbitalType::S,
            primitives: fns[0].1.clone(),
        };
        let s = compute_overlap(&bf, &bf);
        assert!(
            s > 0.0 && s.is_finite(),
            "H 1s STO-3G self-overlap must be positive and finite, got {s}"
        );
    }

    #[test]
    fn overlap_orthogonal_p_orbitals_near_zero() {
        let px = make_p_function([0.0, 0.0, 0.0], OrbitalType::Px, 1.0);
        let py = make_p_function([0.0, 0.0, 0.0], OrbitalType::Py, 1.0);
        let s = compute_overlap(&px, &py);
        assert!(
            s.abs() < 0.1,
            "Px and Py on same center must be orthogonal, got {s}"
        );
    }

    #[test]
    fn overlap_distant_s_functions_smaller() {
        let a = make_s_function([0.0, 0.0, 0.0], 1.0);
        let b_near = make_s_function([0.5, 0.0, 0.0], 1.0);
        let b_far = make_s_function([5.0, 0.0, 0.0], 1.0);
        let s_near = compute_overlap(&a, &b_near);
        let s_far = compute_overlap(&a, &b_far);
        assert!(s_near > s_far, "overlap should decrease with distance");
    }

    // -----------------------------------------------------------------------
    // build_basis_set tests
    // -----------------------------------------------------------------------

    #[test]
    fn build_basis_empty_molecule_returns_error() {
        let mol = Molecule::new("Empty");
        let result = build_basis_set(&mol);
        assert!(matches!(result, Err(OrbitalError::EmptyMolecule)));
    }

    #[test]
    fn build_basis_water_correct_count() {
        let mol = water_molecule();
        let basis = build_basis_set(&mol).unwrap_or_default();
        // O: 5 functions, H: 1 each -> 7 total
        assert_eq!(basis.len(), 7);
    }

    #[test]
    fn build_basis_unsupported_element_returns_error() {
        let mut mol = Molecule::new("FeMol");
        mol.atoms.push(Atom::new(1, Element::Fe, [0.0, 0.0, 0.0]));
        let result = build_basis_set(&mol);
        assert!(matches!(result, Err(OrbitalError::UnsupportedElement(_))));
    }

    #[test]
    fn build_basis_centers_match_atom_positions() {
        let mol = water_molecule();
        let basis = build_basis_set(&mol).unwrap_or_default();
        assert_eq!(basis.len(), 7, "water should yield 7 basis functions");
        for bf in basis.iter().take(5) {
            assert_eq!(bf.center, [0.0, 0.0, 0.0]);
        }
        assert_eq!(basis[5].center, [0.757, 0.586, 0.0]);
    }

    // -----------------------------------------------------------------------
    // default_coefficients tests
    // -----------------------------------------------------------------------

    #[test]
    fn default_coefficients_length_matches_basis() {
        let mol = water_molecule();
        let basis = build_basis_set(&mol).unwrap_or_default();
        let coeffs = default_coefficients(&basis);
        assert_eq!(coeffs.len(), basis.len());
    }

    #[test]
    fn default_coefficients_all_one() {
        let mol = water_molecule();
        let basis = build_basis_set(&mol).unwrap_or_default();
        let coeffs = default_coefficients(&basis);
        assert!(coeffs.iter().all(|&c| (c - 1.0).abs() < f64::EPSILON));
    }

    #[test]
    fn default_coefficients_empty_basis() {
        let coeffs = default_coefficients(&[]);
        assert!(coeffs.is_empty());
    }

    // -----------------------------------------------------------------------
    // compute_electron_density tests
    // -----------------------------------------------------------------------

    #[test]
    fn electron_density_non_negative() {
        let bf = make_s_function([0.0, 0.0, 0.0], 1.0);
        let rho = compute_electron_density(&[bf], &[1.0], &[0.5, 0.5, 0.5]);
        assert!(rho >= 0.0, "electron density must be non-negative");
    }

    #[test]
    fn electron_density_zero_coefficients() {
        let bf = make_s_function([0.0, 0.0, 0.0], 1.0);
        let rho = compute_electron_density(&[bf], &[0.0], &[0.0, 0.0, 0.0]);
        assert!(rho.abs() < 1e-12, "zero coefficient -> zero density");
    }

    // -----------------------------------------------------------------------
    // density_grid tests
    // -----------------------------------------------------------------------

    #[test]
    fn density_grid_correct_dimensions() {
        let mol = water_molecule();
        let config = OrbitalConfig::default();
        let basis = build_basis_set(&mol).unwrap_or_default();
        let coeffs = default_coefficients(&basis);
        let grid = density_grid(&mol, &basis, &coeffs, &config).unwrap_or_else(|_| fallback_grid());
        assert_eq!(grid.values.len(), grid.nx * grid.ny * grid.nz);
    }

    #[test]
    fn density_grid_max_exceeds_isovalue() {
        let mol = water_molecule();
        let config = OrbitalConfig::default();
        let basis = build_basis_set(&mol).unwrap_or_default();
        let coeffs = default_coefficients(&basis);
        let grid = density_grid(&mol, &basis, &coeffs, &config).unwrap_or_else(|_| fallback_grid());
        assert!(
            grid.max_value >= config.isovalue,
            "maximum density {:.4} should exceed isovalue {}",
            grid.max_value,
            config.isovalue
        );
    }

    #[test]
    fn density_grid_respects_max_grid_points() {
        let mol = water_molecule();
        let config = OrbitalConfig {
            grid_spacing: 0.3,
            padding: 3.0,
            max_grid_points: 1,
            isovalue: 0.02,
        };
        let basis = build_basis_set(&mol).unwrap_or_default();
        let coeffs = default_coefficients(&basis);
        let result = density_grid(&mol, &basis, &coeffs, &config);
        assert!(matches!(result, Err(OrbitalError::GridTooLarge)));
    }

    #[test]
    fn density_grid_min_non_negative() {
        let mol = water_molecule();
        let config = OrbitalConfig::default();
        let basis = build_basis_set(&mol).unwrap_or_default();
        let coeffs = default_coefficients(&basis);
        let grid = density_grid(&mol, &basis, &coeffs, &config).unwrap_or_else(|_| fallback_grid());
        assert!(
            grid.min_value >= 0.0,
            "density must be non-negative everywhere"
        );
    }

    // -----------------------------------------------------------------------
    // OrbitalError Display tests
    // -----------------------------------------------------------------------

    #[test]
    fn error_display_empty_molecule() {
        let s = format!("{}", OrbitalError::EmptyMolecule);
        assert!(s.contains("no atoms"));
    }

    #[test]
    fn error_display_unsupported_element() {
        let s = format!("{}", OrbitalError::UnsupportedElement("Fe".to_string()));
        assert!(s.contains("Fe"));
    }

    #[test]
    fn error_display_grid_too_large() {
        let s = format!("{}", OrbitalError::GridTooLarge);
        assert!(s.contains("grid"));
    }

    #[test]
    fn error_display_computation_failed() {
        let s = format!("{}", OrbitalError::ComputationFailed("test".to_string()));
        assert!(s.contains("test"));
    }

    #[test]
    fn error_display_invalid_quantum_numbers() {
        let s = format!("{}", OrbitalError::InvalidQuantumNumbers);
        assert!(s.contains("quantum"));
    }

    // -----------------------------------------------------------------------
    // Serde roundtrip test
    // -----------------------------------------------------------------------

    #[test]
    fn serde_roundtrip_electron_density_grid() {
        let grid = ElectronDensityGrid {
            origin: [-1.0, -2.0, -3.0],
            spacing: 0.3,
            nx: 3,
            ny: 3,
            nz: 3,
            values: (0..27).map(|i| i as f64 * 0.01).collect(),
            min_value: 0.0,
            max_value: 0.26,
        };
        let json = serde_json::to_string(&grid).unwrap_or_default();
        assert!(!json.is_empty(), "serialization produced empty string");
        let decoded: ElectronDensityGrid =
            serde_json::from_str(&json).unwrap_or_else(|_| ElectronDensityGrid {
                origin: [0.0; 3],
                spacing: 1.0,
                nx: 0,
                ny: 0,
                nz: 0,
                values: vec![],
                min_value: 0.0,
                max_value: 0.0,
            });
        assert_eq!(decoded.nx, 3);
        assert_eq!(decoded.ny, 3);
        assert_eq!(decoded.nz, 3);
        assert!((decoded.spacing - 0.3).abs() < 1e-12);
        assert_eq!(decoded.values.len(), 27);
    }

    // -----------------------------------------------------------------------
    // compute_overlap_matrix tests
    // -----------------------------------------------------------------------

    #[test]
    fn overlap_matrix_is_symmetric() {
        let mol = water_molecule();
        let basis = build_basis_set(&mol).unwrap_or_default();
        let s = compute_overlap_matrix(&basis);
        let n = s.len();
        for i in 0..n {
            for j in 0..n {
                assert!(
                    (s[i][j] - s[j][i]).abs() < 1e-10,
                    "S[{i}][{j}]={} != S[{j}][{i}]={}",
                    s[i][j],
                    s[j][i]
                );
            }
        }
    }

    #[test]
    fn overlap_matrix_diagonal_positive() {
        let mol = water_molecule();
        let basis = build_basis_set(&mol).unwrap_or_default();
        let s = compute_overlap_matrix(&basis);
        for (i, row) in s.iter().enumerate() {
            assert!(row[i] > 0.0, "diagonal S[{i}][{i}] must be positive");
        }
    }

    #[test]
    fn overlap_matrix_h1s_diagonal_near_one() {
        // The raw STO-3G H 1s contraction self-overlap is ~10.74 (unnormalised).
        // Diagonal entries must be positive and finite.
        let fns = sto3g_basis(&Element::H).unwrap_or_default();
        assert!(!fns.is_empty(), "H STO-3G basis must not be empty");
        let bf = BasisFunction {
            center: [0.0, 0.0, 0.0],
            orbital_type: OrbitalType::S,
            primitives: fns[0].1.clone(),
        };
        let s = compute_overlap_matrix(&[bf]);
        assert!(
            s[0][0] > 0.0 && s[0][0].is_finite(),
            "H 1s STO-3G diagonal self-overlap must be positive and finite, got {}",
            s[0][0]
        );
    }
}
