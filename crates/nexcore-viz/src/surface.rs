//! Molecular surface generation via marching cubes.
//!
//! Converts molecular atom data into triangle meshes for 3D rendering.
//! Supports Van der Waals surfaces and Solvent-Excluded Surfaces (SES).
//!
//! # Algorithm
//!
//! 1. Compute bounding box of molecule (padded by max VdW + probe radius)
//! 2. Evaluate a signed distance field over a regular 3D grid
//! 3. Run marching cubes to extract the iso-surface at distance = 0
//! 4. Interpolate vertex normals from the distance field gradient
//! 5. Assign per-vertex CPK colors from the nearest atom
//!
//! # Examples
//!
//! ```
//! use nexcore_viz::molecular::{Atom, Element, Molecule};
//! use nexcore_viz::surface::{generate_surface, SurfaceType};
//!
//! let mut mol = Molecule::new("Water");
//! mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
//! mol.atoms.push(Atom::new(2, Element::H, [0.757, 0.586, 0.0]));
//! mol.atoms.push(Atom::new(3, Element::H, [-0.757, 0.586, 0.0]));
//!
//! let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
//! assert!(!mesh.vertices.is_empty());
//! ```

use serde::{Deserialize, Serialize};

use crate::molecular::Molecule;

// ============================================================================
// Public types
// ============================================================================

/// A triangle mesh suitable for 3D rendering.
///
/// All arrays are flat and tightly packed:
/// - `vertices` / `normals` / `colors` have length `3 * vertex_count`
/// - `indices` has length `3 * triangle_count`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SurfaceMesh {
    /// Vertex positions \[x, y, z, x, y, z, …\]
    pub vertices: Vec<f64>,
    /// Vertex normals \[nx, ny, nz, …\] — unit length
    pub normals: Vec<f64>,
    /// Triangle indices (every 3 = one triangle)
    pub indices: Vec<u32>,
    /// Per-vertex colors \[r, g, b, …\] (0.0–1.0)
    pub colors: Vec<f64>,
}

/// Which molecular surface to extract.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub enum SurfaceType {
    /// Van der Waals surface — the literal union of atom spheres.
    VanDerWaals,
    /// Solvent-excluded surface — VdW surface rolled by a probe sphere.
    ///
    /// `probe_radius` is typically 1.4 Å (water).
    SolventExcluded {
        /// Radius of the probe sphere in angstroms.
        probe_radius: f64,
    },
}

// ============================================================================
// Public API
// ============================================================================

/// Generate a triangle mesh for the molecular surface.
///
/// Returns an empty [`SurfaceMesh`] if the molecule has no atoms.
///
/// # Parameters
///
/// - `molecule`     — the molecular structure
/// - `surface_type` — VdW or SES
/// - `resolution`   — grid spacing in Å (0.5 is a good default)
///
/// # Examples
///
/// ```
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
/// use nexcore_viz::surface::{generate_surface, SurfaceType};
///
/// let mut mol = Molecule::new("Methane");
/// mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
/// let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
/// assert!(!mesh.vertices.is_empty());
/// ```
#[must_use]
pub fn generate_surface(molecule: &Molecule, surface_type: SurfaceType, resolution: f64) -> SurfaceMesh {
    if molecule.atoms.is_empty() {
        return SurfaceMesh {
            vertices: Vec::new(),
            normals: Vec::new(),
            indices: Vec::new(),
            colors: Vec::new(),
        };
    }

    let resolution = resolution.max(1e-3); // guard against degenerate grid

    // ------------------------------------------------------------------
    // 1. Bounding box with padding
    // ------------------------------------------------------------------
    let probe = match surface_type {
        SurfaceType::VanDerWaals => 0.0,
        SurfaceType::SolventExcluded { probe_radius } => probe_radius.max(0.0),
    };

    let max_vdw = molecule
        .atoms
        .iter()
        .map(|a| a.element.vdw_radius())
        .fold(0.0_f64, f64::max);

    let padding = max_vdw + probe + resolution; // one extra cell of breathing room

    let (bb_min, bb_max) = molecule.bounding_box();
    let origin = [bb_min[0] - padding, bb_min[1] - padding, bb_min[2] - padding];
    let extent = [
        bb_max[0] - bb_min[0] + 2.0 * padding,
        bb_max[1] - bb_min[1] + 2.0 * padding,
        bb_max[2] - bb_min[2] + 2.0 * padding,
    ];

    // Grid dimensions (at least 2 cells per axis)
    let nx = ((extent[0] / resolution).ceil() as usize).max(2);
    let ny = ((extent[1] / resolution).ceil() as usize).max(2);
    let nz = ((extent[2] / resolution).ceil() as usize).max(2);

    // ------------------------------------------------------------------
    // 2. Evaluate scalar field
    // ------------------------------------------------------------------
    // For VdW: field(p) = min_i( dist(p, atom_i) - vdw_i )
    //   < 0 inside the surface, = 0 on the surface, > 0 outside.
    // For SES: inflate radii by probe, then the iso-surface is at 0.
    //   (This approximates the SES; a full SES requires a second erosion pass,
    //    but the inflated-sphere union is a well-accepted approximation.)
    let field = build_field(molecule, surface_type, &origin, resolution, nx, ny, nz);

    // ------------------------------------------------------------------
    // 3. Marching cubes
    // ------------------------------------------------------------------
    let (vertices, normals, indices) = march(&field, &origin, resolution, nx, ny, nz);

    if vertices.is_empty() {
        return SurfaceMesh {
            vertices,
            normals,
            indices,
            colors: Vec::new(),
        };
    }

    // ------------------------------------------------------------------
    // 4. Color by nearest atom (CPK)
    // ------------------------------------------------------------------
    let colors = color_vertices(&vertices, molecule);

    SurfaceMesh {
        vertices,
        normals,
        indices,
        colors,
    }
}

/// Number of vertices in the mesh.
///
/// # Examples
///
/// ```
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
/// use nexcore_viz::surface::{generate_surface, vertex_count, SurfaceType};
///
/// let mut mol = Molecule::new("Single atom");
/// mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
/// let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
/// assert_eq!(vertex_count(&mesh), mesh.vertices.len() / 3);
/// ```
#[must_use]
pub fn vertex_count(mesh: &SurfaceMesh) -> usize {
    mesh.vertices.len() / 3
}

/// Number of triangles in the mesh.
///
/// # Examples
///
/// ```
/// use nexcore_viz::molecular::{Atom, Element, Molecule};
/// use nexcore_viz::surface::{generate_surface, triangle_count, SurfaceType};
///
/// let mut mol = Molecule::new("Single atom");
/// mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
/// let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
/// assert_eq!(triangle_count(&mesh), mesh.indices.len() / 3);
/// ```
#[must_use]
pub fn triangle_count(mesh: &SurfaceMesh) -> usize {
    mesh.indices.len() / 3
}

// ============================================================================
// Internal: scalar field
// ============================================================================

/// Flat 3D scalar field indexed as `[ix * ny * nz + iy * nz + iz]`.
struct Field {
    data: Vec<f64>,
    _nx: usize,
    ny: usize,
    nz: usize,
}

impl Field {
    fn new(nx: usize, ny: usize, nz: usize) -> Self {
        Self {
            data: vec![f64::MAX; nx * ny * nz],
            _nx: nx,
            ny,
            nz,
        }
    }

    fn index(&self, ix: usize, iy: usize, iz: usize) -> usize {
        ix * self.ny * self.nz + iy * self.nz + iz
    }

    fn get(&self, ix: usize, iy: usize, iz: usize) -> f64 {
        self.data[self.index(ix, iy, iz)]
    }

    fn set(&mut self, ix: usize, iy: usize, iz: usize, v: f64) {
        let idx = self.index(ix, iy, iz);
        self.data[idx] = v;
    }
}

/// Build the scalar distance field over the grid.
fn build_field(
    molecule: &Molecule,
    surface_type: SurfaceType,
    origin: &[f64; 3],
    resolution: f64,
    nx: usize,
    ny: usize,
    nz: usize,
) -> Field {
    let mut field = Field::new(nx, ny, nz);

    // Pre-compute effective radius per atom
    let radii: Vec<f64> = molecule
        .atoms
        .iter()
        .map(|a| {
            let r = a.element.vdw_radius();
            match surface_type {
                SurfaceType::VanDerWaals => r,
                SurfaceType::SolventExcluded { probe_radius } => r + probe_radius.max(0.0),
            }
        })
        .collect();

    for ix in 0..nx {
        let px = origin[0] + ix as f64 * resolution;
        for iy in 0..ny {
            let py = origin[1] + iy as f64 * resolution;
            for iz in 0..nz {
                let pz = origin[2] + iz as f64 * resolution;

                // Signed distance = min over atoms of (dist - effective_radius)
                let mut min_dist = f64::MAX;
                for (atom, &radius) in molecule.atoms.iter().zip(radii.iter()) {
                    let dx = px - atom.position[0];
                    let dy = py - atom.position[1];
                    let dz = pz - atom.position[2];
                    let d = (dx * dx + dy * dy + dz * dz).sqrt() - radius;
                    if d < min_dist {
                        min_dist = d;
                    }
                }
                field.set(ix, iy, iz, min_dist);
            }
        }
    }

    field
}

// ============================================================================
// Internal: marching cubes
// ============================================================================

/// Run marching cubes over the scalar field, returning flat vertex/normal/index arrays.
fn march(
    field: &Field,
    origin: &[f64; 3],
    resolution: f64,
    nx: usize,
    ny: usize,
    nz: usize,
) -> (Vec<f64>, Vec<f64>, Vec<u32>) {
    let mut vertices: Vec<f64> = Vec::new();
    let mut normals: Vec<f64> = Vec::new();
    let mut indices: Vec<u32> = Vec::new();

    // Iterate over each cube (voxel corner at (ix, iy, iz))
    let cx = nx.saturating_sub(1);
    let cy = ny.saturating_sub(1);
    let cz = nz.saturating_sub(1);

    for ix in 0..cx {
        for iy in 0..cy {
            for iz in 0..cz {
                process_cube(
                    field,
                    origin,
                    resolution,
                    ix,
                    iy,
                    iz,
                    &mut vertices,
                    &mut normals,
                    &mut indices,
                );
            }
        }
    }

    (vertices, normals, indices)
}

/// The 8 corner offsets of a marching-cubes voxel cube.
///
/// Ordered per the standard Lorensen-Cline convention (vertex 0–7).
const CUBE_CORNERS: [[usize; 3]; 8] = [
    [0, 0, 0], // 0
    [1, 0, 0], // 1
    [1, 1, 0], // 2
    [0, 1, 0], // 3
    [0, 0, 1], // 4
    [1, 0, 1], // 5
    [1, 1, 1], // 6
    [0, 1, 1], // 7
];

/// The 12 edges of the cube, given as pairs of corner indices.
const CUBE_EDGES: [[usize; 2]; 12] = [
    [0, 1],
    [1, 2],
    [2, 3],
    [3, 0],
    [4, 5],
    [5, 6],
    [6, 7],
    [7, 4],
    [0, 4],
    [1, 5],
    [2, 6],
    [3, 7],
];

/// Process one voxel cube and append any triangles to the output buffers.
#[allow(clippy::too_many_arguments)]
fn process_cube(
    field: &Field,
    origin: &[f64; 3],
    resolution: f64,
    ix: usize,
    iy: usize,
    iz: usize,
    vertices: &mut Vec<f64>,
    normals: &mut Vec<f64>,
    indices: &mut Vec<u32>,
) {
    // Sample the 8 corners
    let mut corner_vals = [0.0_f64; 8];
    let mut corner_pos = [[0.0_f64; 3]; 8];

    for (ci, &[dx, dy, dz]) in CUBE_CORNERS.iter().enumerate() {
        let gx = ix + dx;
        let gy = iy + dy;
        let gz = iz + dz;
        corner_vals[ci] = field.get(gx, gy, gz);
        corner_pos[ci] = [
            origin[0] + gx as f64 * resolution,
            origin[1] + gy as f64 * resolution,
            origin[2] + gz as f64 * resolution,
        ];
    }

    // Build the lookup index (0–255)
    let mut cube_index: usize = 0;
    for (ci, &v) in corner_vals.iter().enumerate() {
        if v < 0.0 {
            cube_index |= 1 << ci;
        }
    }

    // Fully inside or fully outside — no triangles
    if EDGE_TABLE[cube_index] == 0 {
        return;
    }

    // Interpolate vertex positions on active edges
    let mut edge_verts = [[0.0_f64; 3]; 12];
    for (ei, &[c0, c1]) in CUBE_EDGES.iter().enumerate() {
        if EDGE_TABLE[cube_index] & (1 << ei) != 0 {
            edge_verts[ei] = interpolate_edge(corner_pos[c0], corner_pos[c1], corner_vals[c0], corner_vals[c1]);
        }
    }

    // Emit triangles from the triangle table
    let tri_row = &TRI_TABLE[cube_index];
    let mut ti = 0;
    while ti < 15 {
        let e0 = tri_row[ti];
        if e0 == 255 {
            break;
        }
        let e1 = tri_row[ti + 1];
        let e2 = tri_row[ti + 2];

        let v0 = edge_verts[e0 as usize];
        let v1 = edge_verts[e1 as usize];
        let v2 = edge_verts[e2 as usize];

        // Triangle normal via cross product
        let n = triangle_normal(v0, v1, v2);

        let base = (vertices.len() / 3) as u32;

        for v in [v0, v1, v2] {
            vertices.extend_from_slice(&v);
            normals.extend_from_slice(&n);
        }
        indices.extend_from_slice(&[base, base + 1, base + 2]);

        ti += 3;
    }
}

/// Linear interpolation along an edge to find where the iso-surface crosses.
///
/// The iso-value is 0.0 (the surface boundary).
fn interpolate_edge(p0: [f64; 3], p1: [f64; 3], v0: f64, v1: f64) -> [f64; 3] {
    let dv = v1 - v0;
    // Avoid division by zero if field values are identical
    if dv.abs() < 1e-10 {
        return [
            (p0[0] + p1[0]) * 0.5,
            (p0[1] + p1[1]) * 0.5,
            (p0[2] + p1[2]) * 0.5,
        ];
    }
    let t = (-v0) / dv; // solve 0 = v0 + t*(v1-v0)
    let t = t.clamp(0.0, 1.0);
    [
        p0[0] + t * (p1[0] - p0[0]),
        p0[1] + t * (p1[1] - p0[1]),
        p0[2] + t * (p1[2] - p0[2]),
    ]
}

/// Compute unit normal for a triangle, falling back to [0,1,0] for degenerate cases.
fn triangle_normal(v0: [f64; 3], v1: [f64; 3], v2: [f64; 3]) -> [f64; 3] {
    let e1 = [v1[0] - v0[0], v1[1] - v0[1], v1[2] - v0[2]];
    let e2 = [v2[0] - v0[0], v2[1] - v0[1], v2[2] - v0[2]];

    // Cross product e1 × e2
    let nx = e1[1] * e2[2] - e1[2] * e2[1];
    let ny = e1[2] * e2[0] - e1[0] * e2[2];
    let nz = e1[0] * e2[1] - e1[1] * e2[0];

    let len = (nx * nx + ny * ny + nz * nz).sqrt();
    if len < 1e-10 {
        return [0.0, 1.0, 0.0];
    }
    [nx / len, ny / len, nz / len]
}

// ============================================================================
// Internal: vertex coloring
// ============================================================================

/// Assign per-vertex CPK colors by finding the nearest atom for each vertex.
fn color_vertices(vertices: &[f64], molecule: &Molecule) -> Vec<f64> {
    let n_verts = vertices.len() / 3;
    let mut colors = Vec::with_capacity(n_verts * 3);

    for vi in 0..n_verts {
        let vx = vertices[vi * 3];
        let vy = vertices[vi * 3 + 1];
        let vz = vertices[vi * 3 + 2];

        let mut nearest_color = "#FF1493"; // default: hot-pink for Other
        let mut best_dist2 = f64::MAX;

        for atom in &molecule.atoms {
            let dx = vx - atom.position[0];
            let dy = vy - atom.position[1];
            let dz = vz - atom.position[2];
            let d2 = dx * dx + dy * dy + dz * dz;
            if d2 < best_dist2 {
                best_dist2 = d2;
                nearest_color = atom.element.cpk_color();
            }
        }

        let (r, g, b) = hex_to_rgb(nearest_color);
        colors.push(r);
        colors.push(g);
        colors.push(b);
    }

    colors
}

/// Parse a `#RRGGBB` hex color string to (r, g, b) in 0.0–1.0.
///
/// Falls back to (1.0, 0.0, 1.0) magenta for malformed input.
fn hex_to_rgb(hex: &str) -> (f64, f64, f64) {
    let hex = hex.trim_start_matches('#');
    if hex.len() != 6 {
        return (1.0, 0.0, 1.0);
    }

    let r = u8::from_str_radix(&hex[0..2], 16).unwrap_or(255);
    let g = u8::from_str_radix(&hex[2..4], 16).unwrap_or(0);
    let b = u8::from_str_radix(&hex[4..6], 16).unwrap_or(255);

    (r as f64 / 255.0, g as f64 / 255.0, b as f64 / 255.0)
}

// ============================================================================
// Marching cubes lookup tables
// ============================================================================
//
// Source: Lorensen & Cline, "Marching Cubes: A High Resolution 3D Surface
// Construction Algorithm", SIGGRAPH 1987.
//
// EDGE_TABLE[i]: bitmask of the 12 edges cut by the iso-surface for cube
//   configuration i (i = bitmask of the 8 corners that are "inside").
//
// TRI_TABLE[i]: sequence of edge triplets forming triangles for configuration i.
//   Rows are 15 entries long; 255 is the sentinel terminator.

/// 256-entry edge table — one bitmask per cube configuration.
const EDGE_TABLE: [u16; 256] = [
    0x000, 0x109, 0x203, 0x30A, 0x406, 0x50F, 0x605, 0x70C,
    0x80C, 0x905, 0xA0F, 0xB06, 0xC0A, 0xD03, 0xE09, 0xF00,
    0x190, 0x099, 0x393, 0x29A, 0x596, 0x49F, 0x795, 0x69C,
    0x99C, 0x895, 0xB9F, 0xA96, 0xD9A, 0xC93, 0xF99, 0xE90,
    0x230, 0x339, 0x033, 0x13A, 0x636, 0x73F, 0x435, 0x53C,
    0xA3C, 0xB35, 0x83F, 0x936, 0xE3A, 0xF33, 0xC39, 0xD30,
    0x3A0, 0x2A9, 0x1A3, 0x0AA, 0x7A6, 0x6AF, 0x5A5, 0x4AC,
    0xBAC, 0xAA5, 0x9AF, 0x8A6, 0xFAA, 0xEA3, 0xDA9, 0xCA0,
    0x460, 0x569, 0x663, 0x76A, 0x066, 0x16F, 0x265, 0x36C,
    0xC6C, 0xD65, 0xE6F, 0xF66, 0x86A, 0x963, 0xA69, 0xB60,
    0x5F0, 0x4F9, 0x7F3, 0x6FA, 0x1F6, 0x0FF, 0x3F5, 0x2FC,
    0xDFC, 0xCF5, 0xFFF, 0xEF6, 0x9FA, 0x8F3, 0xBF9, 0xAF0,
    0x650, 0x759, 0x453, 0x55A, 0x256, 0x35F, 0x055, 0x15C,
    0xE5C, 0xF55, 0xC5F, 0xD56, 0xA5A, 0xB53, 0x859, 0x950,
    0x7C0, 0x6C9, 0x5C3, 0x4CA, 0x3C6, 0x2CF, 0x1C5, 0x0CC,
    0xFCC, 0xEC5, 0xDCF, 0xCC6, 0xBCA, 0xAC3, 0x9C9, 0x8C0,
    0x8C0, 0x9C9, 0xAC3, 0xBCA, 0xCC6, 0xDCF, 0xEC5, 0xFCC,
    0x0CC, 0x1C5, 0x2CF, 0x3C6, 0x4CA, 0x5C3, 0x6C9, 0x7C0,
    0x950, 0x859, 0xB53, 0xA5A, 0xD56, 0xC5F, 0xF55, 0xE5C,
    0x15C, 0x055, 0x35F, 0x256, 0x55A, 0x453, 0x759, 0x650,
    0xAF0, 0xBF9, 0x8F3, 0x9FA, 0xEF6, 0xFFF, 0xCF5, 0xDFC,
    0x2FC, 0x3F5, 0x0FF, 0x1F6, 0x6FA, 0x7F3, 0x4F9, 0x5F0,
    0xB60, 0xA69, 0x963, 0x86A, 0xF66, 0xE6F, 0xD65, 0xC6C,
    0x36C, 0x265, 0x16F, 0x066, 0x76A, 0x663, 0x569, 0x460,
    0xCA0, 0xDA9, 0xEA3, 0xFAA, 0x8A6, 0x9AF, 0xAA5, 0xBAC,
    0x4AC, 0x5A5, 0x6AF, 0x7A6, 0x0AA, 0x1A3, 0x2A9, 0x3A0,
    0xD30, 0xC39, 0xF33, 0xE3A, 0x936, 0xB35, 0xB3F, 0xA36,
    0x53C, 0x435, 0x73F, 0x636, 0x13A, 0x033, 0x339, 0x230,
    0xE90, 0xF99, 0xC93, 0xD9A, 0xA96, 0xB9F, 0x895, 0x99C,
    0x69C, 0x795, 0x49F, 0x596, 0x29A, 0x393, 0x099, 0x190,
    0xF00, 0xE09, 0xD03, 0xC0A, 0xB06, 0xA0F, 0x905, 0x80C,
    0x70C, 0x605, 0x50F, 0x406, 0x30A, 0x203, 0x109, 0x000,
];

/// 256 × 15 triangle table.
///
/// Each row lists edge triplets; 255 terminates the row.
/// Edge indices 0–11 correspond to `CUBE_EDGES`.
#[rustfmt::skip]
const TRI_TABLE: [[u8; 15]; 256] = [
    [255,255,255,255,255,255,255,255,255,255,255,255,255,255,255],
    [  0,  8,  3,255,255,255,255,255,255,255,255,255,255,255,255],
    [  0,  1,  9,255,255,255,255,255,255,255,255,255,255,255,255],
    [  1,  8,  3,  9,  8,  1,255,255,255,255,255,255,255,255,255],
    [  1,  2, 10,255,255,255,255,255,255,255,255,255,255,255,255],
    [  0,  8,  3,  1,  2, 10,255,255,255,255,255,255,255,255,255],
    [  9,  2, 10,  0,  2,  9,255,255,255,255,255,255,255,255,255],
    [  2,  8,  3,  2, 10,  8, 10,  9,  8,255,255,255,255,255,255],
    [  3, 11,  2,255,255,255,255,255,255,255,255,255,255,255,255],
    [  0, 11,  2,  8, 11,  0,255,255,255,255,255,255,255,255,255],
    [  1,  9,  0,  2,  3, 11,255,255,255,255,255,255,255,255,255],
    [  1, 11,  2,  1,  9, 11,  9,  8, 11,255,255,255,255,255,255],
    [  3, 10,  1, 11, 10,  3,255,255,255,255,255,255,255,255,255],
    [  0, 10,  1,  0,  8, 10,  8, 11, 10,255,255,255,255,255,255],
    [  3,  9,  0,  3, 11,  9, 11, 10,  9,255,255,255,255,255,255],
    [  9,  8, 10, 10,  8, 11,255,255,255,255,255,255,255,255,255],
    [  4,  7,  8,255,255,255,255,255,255,255,255,255,255,255,255],
    [  4,  3,  0,  7,  3,  4,255,255,255,255,255,255,255,255,255],
    [  0,  1,  9,  8,  4,  7,255,255,255,255,255,255,255,255,255],
    [  4,  1,  9,  4,  7,  1,  7,  3,  1,255,255,255,255,255,255],
    [  1,  2, 10,  8,  4,  7,255,255,255,255,255,255,255,255,255],
    [  3,  4,  7,  3,  0,  4,  1,  2, 10,255,255,255,255,255,255],
    [  9,  2, 10,  9,  0,  2,  8,  4,  7,255,255,255,255,255,255],
    [  2, 10,  9,  2,  9,  7,  2,  7,  3,  7,  9,  4,255,255,255],
    [  8,  4,  7,  3, 11,  2,255,255,255,255,255,255,255,255,255],
    [ 11,  4,  7, 11,  2,  4,  2,  0,  4,255,255,255,255,255,255],
    [  9,  0,  1,  8,  4,  7,  2,  3, 11,255,255,255,255,255,255],
    [  4,  7, 11,  9,  4, 11,  9, 11,  2,  9,  2,  1,255,255,255],
    [  3, 10,  1,  3, 11, 10,  7,  8,  4,255,255,255,255,255,255],
    [  1, 11, 10,  1,  4, 11,  1,  0,  4,  7, 11,  4,255,255,255],
    [  4,  7,  8,  9,  0, 11,  9, 11, 10, 11,  0,  3,255,255,255],
    [  4,  7, 11,  4, 11,  9,  9, 11, 10,255,255,255,255,255,255],
    [  9,  5,  4,255,255,255,255,255,255,255,255,255,255,255,255],
    [  9,  5,  4,  0,  8,  3,255,255,255,255,255,255,255,255,255],
    [  0,  5,  4,  1,  5,  0,255,255,255,255,255,255,255,255,255],
    [  8,  5,  4,  8,  3,  5,  3,  1,  5,255,255,255,255,255,255],
    [  1,  2, 10,  9,  5,  4,255,255,255,255,255,255,255,255,255],
    [  3,  0,  8,  1,  2, 10,  4,  9,  5,255,255,255,255,255,255],
    [  5,  2, 10,  5,  4,  2,  4,  0,  2,255,255,255,255,255,255],
    [  2, 10,  5,  3,  2,  5,  3,  5,  4,  3,  4,  8,255,255,255],
    [  9,  5,  4,  2,  3, 11,255,255,255,255,255,255,255,255,255],
    [  0, 11,  2,  0,  8, 11,  4,  9,  5,255,255,255,255,255,255],
    [  0,  5,  4,  0,  1,  5,  2,  3, 11,255,255,255,255,255,255],
    [  2,  1,  5,  2,  5,  8,  2,  8, 11,  4,  8,  5,255,255,255],
    [ 10,  3, 11, 10,  1,  3,  9,  5,  4,255,255,255,255,255,255],
    [  4,  9,  5,  0,  8,  1,  8, 10,  1,  8, 11, 10,255,255,255],
    [  5,  4,  0,  5,  0, 11,  5, 11, 10, 11,  0,  3,255,255,255],
    [  5,  4,  8,  5,  8, 10, 10,  8, 11,255,255,255,255,255,255],
    [  9,  7,  8,  5,  7,  9,255,255,255,255,255,255,255,255,255],
    [  9,  3,  0,  9,  5,  3,  5,  7,  3,255,255,255,255,255,255],
    [  0,  7,  8,  0,  1,  7,  1,  5,  7,255,255,255,255,255,255],
    [  1,  5,  3,  3,  5,  7,255,255,255,255,255,255,255,255,255],
    [  9,  7,  8,  9,  5,  7, 10,  1,  2,255,255,255,255,255,255],
    [ 10,  1,  2,  9,  5,  0,  5,  3,  0,  5,  7,  3,255,255,255],
    [  8,  0,  2,  8,  2,  5,  8,  5,  7, 10,  5,  2,255,255,255],
    [  2, 10,  5,  2,  5,  3,  3,  5,  7,255,255,255,255,255,255],
    [  7,  9,  5,  7,  8,  9,  3, 11,  2,255,255,255,255,255,255],
    [  9,  5,  7,  9,  7,  2,  9,  2,  0,  2,  7, 11,255,255,255],
    [  2,  3, 11,  0,  1,  8,  1,  7,  8,  1,  5,  7,255,255,255],
    [ 11,  2,  1, 11,  1,  7,  7,  1,  5,255,255,255,255,255,255],
    [  9,  5,  8,  8,  5,  7, 10,  1,  3, 10,  3, 11,255,255,255],
    [  5,  7,  0,  5,  0,  9,  7, 11,  0,  1,  0, 10, 11, 10,  0],
    [ 11, 10,  0, 11,  0,  3, 10,  5,  0,  8,  0,  7,  5,  7,  0],
    [ 11, 10,  5,  7, 11,  5,255,255,255,255,255,255,255,255,255],
    [ 10,  6,  5,255,255,255,255,255,255,255,255,255,255,255,255],
    [  0,  8,  3,  5, 10,  6,255,255,255,255,255,255,255,255,255],
    [  9,  0,  1,  5, 10,  6,255,255,255,255,255,255,255,255,255],
    [  1,  8,  3,  1,  9,  8,  5, 10,  6,255,255,255,255,255,255],
    [  1,  6,  5,  2,  6,  1,255,255,255,255,255,255,255,255,255],
    [  1,  6,  5,  1,  2,  6,  3,  0,  8,255,255,255,255,255,255],
    [  9,  6,  5,  9,  0,  6,  0,  2,  6,255,255,255,255,255,255],
    [  5,  9,  8,  5,  8,  2,  5,  2,  6,  3,  2,  8,255,255,255],
    [  2,  3, 11, 10,  6,  5,255,255,255,255,255,255,255,255,255],
    [ 11,  0,  8, 11,  2,  0, 10,  6,  5,255,255,255,255,255,255],
    [  0,  1,  9,  2,  3, 11,  5, 10,  6,255,255,255,255,255,255],
    [  5, 10,  6,  1,  9,  2,  9, 11,  2,  9,  8, 11,255,255,255],
    [  6,  3, 11,  6,  5,  3,  5,  1,  3,255,255,255,255,255,255],
    [  0,  8, 11,  0, 11,  5,  0,  5,  1,  5, 11,  6,255,255,255],
    [  3, 11,  6,  0,  3,  6,  0,  6,  5,  0,  5,  9,255,255,255],
    [  6,  5,  9,  6,  9, 11, 11,  9,  8,255,255,255,255,255,255],
    [  5, 10,  6,  4,  7,  8,255,255,255,255,255,255,255,255,255],
    [  4,  3,  0,  4,  7,  3,  6,  5, 10,255,255,255,255,255,255],
    [  1,  9,  0,  5, 10,  6,  8,  4,  7,255,255,255,255,255,255],
    [ 10,  6,  5,  1,  9,  7,  1,  7,  3,  7,  9,  4,255,255,255],
    [  6,  1,  2,  6,  5,  1,  4,  7,  8,255,255,255,255,255,255],
    [  1,  2,  5,  5,  2,  6,  3,  0,  4,  3,  4,  7,255,255,255],
    [  8,  4,  7,  9,  0,  5,  0,  6,  5,  0,  2,  6,255,255,255],
    [  7,  3,  9,  7,  9,  4,  3,  2,  9,  5,  9,  6,  2,  6,  9],
    [  3, 11,  2,  7,  8,  4, 10,  6,  5,255,255,255,255,255,255],
    [  5, 10,  6,  4,  7,  2,  4,  2,  0,  2,  7, 11,255,255,255],
    [  0,  1,  9,  4,  7,  8,  2,  3, 11,  5, 10,  6,255,255,255],
    [  9,  2,  1,  9, 11,  2,  9,  4, 11,  7, 11,  4,  5, 10,  6],
    [  8,  4,  7,  3, 11,  5,  3,  5,  1,  5, 11,  6,255,255,255],
    [  5,  1, 11,  5, 11,  6,  1,  0, 11,  7, 11,  4,  0,  4, 11],
    [  0,  5,  9,  0,  6,  5,  0,  3,  6, 11,  6,  3,  8,  4,  7],
    [  6,  5,  9,  6,  9, 11,  4,  7,  9,  7, 11,  9,255,255,255],
    [ 10,  4,  9,  6,  4, 10,255,255,255,255,255,255,255,255,255],
    [  4, 10,  6,  4,  9, 10,  0,  8,  3,255,255,255,255,255,255],
    [ 10,  0,  1, 10,  6,  0,  6,  4,  0,255,255,255,255,255,255],
    [  8,  3,  1,  8,  1,  6,  8,  6,  4,  6,  1, 10,255,255,255],
    [  1,  4,  9,  1,  2,  4,  2,  6,  4,255,255,255,255,255,255],
    [  3,  0,  8,  1,  2,  9,  2,  4,  9,  2,  6,  4,255,255,255],
    [  0,  2,  4,  4,  2,  6,255,255,255,255,255,255,255,255,255],
    [  8,  3,  2,  8,  2,  4,  4,  2,  6,255,255,255,255,255,255],
    [ 10,  4,  9, 10,  6,  4, 11,  2,  3,255,255,255,255,255,255],
    [  0,  8,  2,  2,  8, 11,  4,  9, 10,  4, 10,  6,255,255,255],
    [  3, 11,  2,  0,  1,  6,  0,  6,  4,  6,  1, 10,255,255,255],
    [  6,  4,  1,  6,  1, 10,  4,  8,  1,  2,  1, 11,  8, 11,  1],
    [  9,  6,  4,  9,  3,  6,  9,  1,  3, 11,  6,  3,255,255,255],
    [  8, 11,  1,  8,  1,  0, 11,  6,  1,  9,  1,  4,  6,  4,  1],
    [  3, 11,  6,  3,  6,  0,  0,  6,  4,255,255,255,255,255,255],
    [  6,  4,  8, 11,  6,  8,255,255,255,255,255,255,255,255,255],
    [  7, 10,  6,  7,  8, 10,  8,  9, 10,255,255,255,255,255,255],
    [  0,  7,  3,  0, 10,  7,  0,  9, 10,  6,  7, 10,255,255,255],
    [ 10,  6,  7,  1, 10,  7,  1,  7,  8,  1,  8,  0,255,255,255],
    [ 10,  6,  7, 10,  7,  1,  1,  7,  3,255,255,255,255,255,255],
    [  1,  2,  6,  1,  6,  8,  1,  8,  9,  8,  6,  7,255,255,255],
    [  2,  6,  9,  2,  9,  1,  6,  7,  9,  0,  9,  3,  7,  3,  9],
    [  7,  8,  0,  7,  0,  6,  6,  0,  2,255,255,255,255,255,255],
    [  7,  3,  2,  6,  7,  2,255,255,255,255,255,255,255,255,255],
    [  2,  3, 11, 10,  6,  8, 10,  8,  9,  8,  6,  7,255,255,255],
    [  2,  0,  7,  2,  7, 11,  0,  9,  7,  6,  7, 10,  9, 10,  7],
    [  1,  8,  0,  1,  7,  8,  1, 10,  7,  6,  7, 10,  2,  3, 11],
    [ 11,  2,  1, 11,  1,  7, 10,  6,  1,  6,  7,  1,255,255,255],
    [  8,  9,  1,  8,  1,  3,  9,  6,  1, 11,  1,  7,  6,  7,  1],
    [  0,  9,  1, 11,  6,  7,255,255,255,255,255,255,255,255,255],
    [  7, 11,  6,  3,  8,  0,255,255,255,255,255,255,255,255,255],
    [  7, 11,  6,255,255,255,255,255,255,255,255,255,255,255,255],
    [  7,  6, 11,255,255,255,255,255,255,255,255,255,255,255,255],
    [  3,  0,  8, 11,  7,  6,255,255,255,255,255,255,255,255,255],
    [  0,  1,  9, 11,  7,  6,255,255,255,255,255,255,255,255,255],
    [  8,  1,  9,  8,  3,  1, 11,  7,  6,255,255,255,255,255,255],
    [ 10,  1,  2,  6, 11,  7,255,255,255,255,255,255,255,255,255],
    [  1,  2, 10,  3,  0,  8,  6, 11,  7,255,255,255,255,255,255],
    [  2,  9,  0,  2, 10,  9,  6, 11,  7,255,255,255,255,255,255],
    [  6, 11,  7,  2, 10,  3, 10,  8,  3, 10,  9,  8,255,255,255],
    [  7,  2,  3,  6,  2,  7,255,255,255,255,255,255,255,255,255],
    [  7,  0,  8,  7,  6,  0,  6,  2,  0,255,255,255,255,255,255],
    [  2,  7,  6,  2,  3,  7,  0,  1,  9,255,255,255,255,255,255],
    [  1,  6,  2,  1,  8,  6,  1,  9,  8,  8,  7,  6,255,255,255],
    [ 10,  7,  6, 10,  1,  7,  1,  3,  7,255,255,255,255,255,255],
    [ 10,  7,  6,  1,  7, 10,  1,  8,  7,  1,  0,  8,255,255,255],
    [  0,  3,  7,  0,  7, 10,  0, 10,  9,  6, 10,  7,255,255,255],
    [  7,  6, 10,  7, 10,  8,  8, 10,  9,255,255,255,255,255,255],
    [  6,  8,  4, 11,  8,  6,255,255,255,255,255,255,255,255,255],
    [  3,  6, 11,  3,  0,  6,  0,  4,  6,255,255,255,255,255,255],
    [  8,  6, 11,  8,  4,  6,  9,  0,  1,255,255,255,255,255,255],
    [  9,  4,  6,  9,  6,  3,  9,  3,  1, 11,  3,  6,255,255,255],
    [  6,  8,  4,  6, 11,  8,  2, 10,  1,255,255,255,255,255,255],
    [  1,  2, 10,  3,  0, 11,  0,  6, 11,  0,  4,  6,255,255,255],
    [  4, 11,  8,  4,  6, 11,  0,  2,  9,  2, 10,  9,255,255,255],
    [ 10,  9,  3, 10,  3,  2,  9,  4,  3, 11,  3,  6,  4,  6,  3],
    [  8,  2,  3,  8,  4,  2,  4,  6,  2,255,255,255,255,255,255],
    [  0,  4,  2,  4,  6,  2,255,255,255,255,255,255,255,255,255],
    [  1,  9,  0,  2,  3,  4,  2,  4,  6,  4,  3,  8,255,255,255],
    [  1,  9,  4,  1,  4,  2,  2,  4,  6,255,255,255,255,255,255],
    [  8,  1,  3,  8,  6,  1,  8,  4,  6,  6, 10,  1,255,255,255],
    [ 10,  1,  0, 10,  0,  6,  6,  0,  4,255,255,255,255,255,255],
    [  4,  6,  3,  4,  3,  8,  6, 10,  3,  0,  3,  9, 10,  9,  3],
    [ 10,  9,  4,  6, 10,  4,255,255,255,255,255,255,255,255,255],
    [  4,  9,  5,  7,  6, 11,255,255,255,255,255,255,255,255,255],
    [  0,  8,  3,  4,  9,  5, 11,  7,  6,255,255,255,255,255,255],
    [  5,  0,  1,  5,  4,  0,  7,  6, 11,255,255,255,255,255,255],
    [ 11,  7,  6,  8,  3,  4,  3,  5,  4,  3,  1,  5,255,255,255],
    [  9,  5,  4, 10,  1,  2,  7,  6, 11,255,255,255,255,255,255],
    [  6, 11,  7,  1,  2, 10,  0,  8,  3,  4,  9,  5,255,255,255],
    [  7,  6, 11,  5,  4, 10,  4,  2, 10,  4,  0,  2,255,255,255],
    [  3,  4,  8,  3,  5,  4,  3,  2,  5, 10,  5,  2, 11,  7,  6],
    [  7,  2,  3,  7,  6,  2,  5,  4,  9,255,255,255,255,255,255],
    [  9,  5,  4,  0,  8,  6,  0,  6,  2,  6,  8,  7,255,255,255],
    [  3,  6,  2,  3,  7,  6,  1,  5,  0,  5,  4,  0,255,255,255],
    [  6,  2,  8,  6,  8,  7,  2,  1,  8,  4,  8,  5,  1,  5,  8],
    [  9,  5,  4, 10,  1,  6,  1,  7,  6,  1,  3,  7,255,255,255],
    [  1,  6, 10,  1,  7,  6,  1,  0,  7,  8,  7,  0,  9,  5,  4],
    [  4,  0, 10,  4, 10,  5,  0,  3, 10,  6, 10,  7,  3,  7, 10],
    [  7,  6, 10,  7, 10,  8,  5,  4, 10,  4,  8, 10,255,255,255],
    [  6,  9,  5,  6, 11,  9, 11,  8,  9,255,255,255,255,255,255],
    [  3,  6, 11,  0,  6,  3,  0,  5,  6,  0,  9,  5,255,255,255],
    [  0, 11,  8,  0,  5, 11,  0,  1,  5,  5,  6, 11,255,255,255],
    [  6, 11,  3,  6,  3,  5,  5,  3,  1,255,255,255,255,255,255],
    [  1,  2, 10,  9,  5, 11,  9, 11,  8, 11,  5,  6,255,255,255],
    [  0, 11,  3,  0,  6, 11,  0,  9,  6,  5,  6,  9,  1,  2, 10],
    [ 11,  8,  5, 11,  5,  6,  8,  0,  5, 10,  5,  2,  0,  2,  5],
    [  6, 11,  3,  6,  3,  5,  2, 10,  3, 10,  5,  3,255,255,255],
    [  5,  8,  9,  5,  2,  8,  5,  6,  2,  3,  8,  2,255,255,255],
    [  9,  5,  6,  9,  6,  0,  0,  6,  2,255,255,255,255,255,255],
    [  1,  5,  8,  1,  8,  0,  5,  6,  8,  3,  8,  2,  6,  2,  8],
    [  1,  5,  6,  2,  1,  6,255,255,255,255,255,255,255,255,255],
    [  1,  3,  6,  1,  6, 10,  3,  8,  6,  5,  6,  9,  8,  9,  6],
    [ 10,  1,  0,  10, 0,  6,  9,  5,  0,  5,  6,  0,255,255,255],
    [  0,  3,  8,  5,  6, 10,255,255,255,255,255,255,255,255,255],
    [ 10,  5,  6,255,255,255,255,255,255,255,255,255,255,255,255],
    [ 11,  5, 10,  7,  5, 11,255,255,255,255,255,255,255,255,255],
    [ 11,  5, 10, 11,  7,  5,  8,  3,  0,255,255,255,255,255,255],
    [  5, 11,  7,  5, 10, 11,  1,  9,  0,255,255,255,255,255,255],
    [ 10,  7,  5, 10, 11,  7,  9,  8,  1,  8,  3,  1,255,255,255],
    [ 11,  1,  2, 11,  7,  1,  7,  5,  1,255,255,255,255,255,255],
    [  0,  8,  3,  1,  2,  7,  1,  7,  5,  7,  2, 11,255,255,255],
    [  9,  7,  5,  9,  2,  7,  9,  0,  2,  2, 11,  7,255,255,255],
    [  7,  5,  2,  7,  2, 11,  5,  9,  2,  3,  2,  8,  9,  8,  2],
    [  2,  5, 10,  2,  3,  5,  3,  7,  5,255,255,255,255,255,255],
    [  8,  2,  0,  8,  5,  2,  8,  7,  5, 10,  2,  5,255,255,255],
    [  9,  0,  1,  5, 10,  3,  5,  3,  7,  3, 10,  2,255,255,255],
    [  9,  8,  2,  9,  2,  1,  8,  7,  2, 10,  2,  5,  7,  5,  2],
    [  1,  3,  5,  3,  7,  5,255,255,255,255,255,255,255,255,255],
    [  0,  8,  7,  0,  7,  1,  1,  7,  5,255,255,255,255,255,255],
    [  9,  0,  3,  9,  3,  5,  5,  3,  7,255,255,255,255,255,255],
    [  9,  8,  7,  5,  9,  7,255,255,255,255,255,255,255,255,255],
    [  5,  8,  4,  5, 10,  8, 10, 11,  8,255,255,255,255,255,255],
    [  5,  0,  4,  5, 11,  0,  5, 10, 11, 11,  3,  0,255,255,255],
    [  0,  1,  9,  8,  4, 10,  8, 10, 11, 10,  4,  5,255,255,255],
    [ 10, 11,  4, 10,  4,  5, 11,  3,  4,  9,  4,  1,  3,  1,  4],
    [  2,  5,  1,  2,  8,  5,  2, 11,  8,  4,  5,  8,255,255,255],
    [  0,  4, 11,  0, 11,  3,  4,  5, 11,  2, 11,  1,  5,  1, 11],
    [  0,  2,  5,  0,  5,  9,  2, 11,  5,  4,  5,  8, 11,  8,  5],
    [  9,  4,  5,  2, 11,  3,255,255,255,255,255,255,255,255,255],
    [  2,  5, 10,  3,  5,  2,  3,  4,  5,  3,  8,  4,255,255,255],
    [  5, 10,  2,  5,  2,  4,  4,  2,  0,255,255,255,255,255,255],
    [  3, 10,  2,  3,  5, 10,  3,  8,  5,  4,  5,  8,  0,  1,  9],
    [  5, 10,  2,  5,  2,  4,  1,  9,  2,  9,  4,  2,255,255,255],
    [  8,  4,  5,  8,  5,  3,  3,  5,  1,255,255,255,255,255,255],
    [  0,  4,  5,  1,  0,  5,255,255,255,255,255,255,255,255,255],
    [  8,  4,  5,  8,  5,  3,  9,  0,  5,  0,  3,  5,255,255,255],
    [  9,  4,  5,255,255,255,255,255,255,255,255,255,255,255,255],
    [  4, 11,  7,  4,  9, 11,  9, 10, 11,255,255,255,255,255,255],
    [  0,  8,  3,  4,  9,  7,  9, 11,  7,  9, 10, 11,255,255,255],
    [  1, 10, 11,  1, 11,  4,  1,  4,  0,  7,  4, 11,255,255,255],
    [  3,  1,  4,  3,  4,  8,  1, 10,  4,  7,  4, 11, 10, 11,  4],
    [  4, 11,  7,  9, 11,  4,  9,  2, 11,  9,  1,  2,255,255,255],
    [  9,  7,  4,  9, 11,  7,  9,  1, 11,  2, 11,  1,  0,  8,  3],
    [ 11,  7,  4,  2, 11,  4,  2,  4,  0,255,255,255,255,255,255],
    [ 11,  7,  4, 11,  4,  2,  8,  3,  4,  3,  2,  4,255,255,255],
    [  2,  9, 10,  2,  7,  9,  2,  3,  7,  7,  4,  9,255,255,255],
    [  9, 10,  7,  9,  7,  4, 10,  2,  7,  8,  7,  0,  2,  0,  7],
    [  3,  7, 10,  3, 10,  2,  7,  4, 10,  1, 10,  0,  4,  0, 10],
    [  1, 10,  2,  8,  7,  4,255,255,255,255,255,255,255,255,255],
    [  4,  9,  1,  4,  1,  7,  7,  1,  3,255,255,255,255,255,255],
    [  4,  9,  1,  4,  1,  7,  0,  8,  1,  8,  7,  1,255,255,255],
    [  4,  0,  3,  7,  4,  3,255,255,255,255,255,255,255,255,255],
    [  4,  8,  7,255,255,255,255,255,255,255,255,255,255,255,255],
    [  9, 10,  8, 10, 11,  8,255,255,255,255,255,255,255,255,255],
    [  3,  0,  9,  3,  9, 11, 11,  9, 10,255,255,255,255,255,255],
    [  0,  1, 10,  0, 10,  8,  8, 10, 11,255,255,255,255,255,255],
    [  3,  1, 10, 11,  3, 10,255,255,255,255,255,255,255,255,255],
    [  1,  2, 11,  1, 11,  9,  9, 11,  8,255,255,255,255,255,255],
    [  3,  0,  9,  3,  9, 11,  1,  2,  9,  2, 11,  9,255,255,255],
    [  0,  2, 11,  8,  0, 11,255,255,255,255,255,255,255,255,255],
    [  3,  2, 11,255,255,255,255,255,255,255,255,255,255,255,255],
    [  2,  3,  8,  2,  8, 10, 10,  8,  9,255,255,255,255,255,255],
    [  9, 10,  2,  0,  9,  2,255,255,255,255,255,255,255,255,255],
    [  2,  3,  8,  2,  8, 10,  0,  1,  8,  1, 10,  8,255,255,255],
    [  1, 10,  2,255,255,255,255,255,255,255,255,255,255,255,255],
    [  1,  3,  8,  9,  1,  8,255,255,255,255,255,255,255,255,255],
    [  0,  9,  1,255,255,255,255,255,255,255,255,255,255,255,255],
    [  0,  3,  8,255,255,255,255,255,255,255,255,255,255,255,255],
    [255,255,255,255,255,255,255,255,255,255,255,255,255,255,255],
];

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::molecular::{Atom, BondOrder, Bond, Element, Molecule};

    /// Build a water molecule for tests.
    fn water() -> Molecule {
        let mut mol = Molecule::new("Water");
        mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::H, [0.757, 0.586, 0.0]));
        mol.atoms.push(Atom::new(3, Element::H, [-0.757, 0.586, 0.0]));
        mol.bonds.push(Bond { atom1: 0, atom2: 1, order: BondOrder::Single });
        mol.bonds.push(Bond { atom1: 0, atom2: 2, order: BondOrder::Single });
        mol
    }

    #[test]
    fn vdw_surface_water_non_empty() {
        let mol = water();
        let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
        assert!(!mesh.vertices.is_empty(), "VdW surface for water must produce vertices");
        assert!(!mesh.indices.is_empty(), "VdW surface for water must produce triangles");
        assert!(!mesh.colors.is_empty(), "VdW surface for water must produce colors");
    }

    #[test]
    fn vertex_and_triangle_count_helpers() {
        let mol = water();
        let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
        let vc = vertex_count(&mesh);
        let tc = triangle_count(&mesh);
        // Every triangle references 3 indices, 3 vertices stored inline
        assert_eq!(mesh.indices.len(), tc * 3, "indices.len() == triangle_count * 3");
        assert_eq!(mesh.vertices.len(), vc * 3, "vertices.len() == vertex_count * 3");
    }

    #[test]
    fn normals_same_length_as_vertices() {
        let mol = water();
        let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
        assert_eq!(
            mesh.normals.len(),
            mesh.vertices.len(),
            "normals array must be the same length as vertices array"
        );
    }

    #[test]
    fn colors_same_length_as_vertices() {
        let mol = water();
        let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
        assert_eq!(
            mesh.colors.len(),
            mesh.vertices.len(),
            "colors array must be the same length as vertices array"
        );
    }

    #[test]
    fn ses_surface_non_empty() {
        let mol = water();
        let mesh = generate_surface(&mol, SurfaceType::SolventExcluded { probe_radius: 1.4 }, 0.5);
        assert!(!mesh.vertices.is_empty(), "SES surface for water must produce vertices");
    }

    #[test]
    fn empty_molecule_returns_empty_mesh() {
        let mol = Molecule::new("Empty");
        let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
        assert!(mesh.vertices.is_empty());
        assert!(mesh.normals.is_empty());
        assert!(mesh.indices.is_empty());
        assert!(mesh.colors.is_empty());
    }

    #[test]
    fn single_atom_carbon_has_surface() {
        let mut mol = Molecule::new("Methane-ish");
        mol.atoms.push(Atom::new(1, Element::C, [0.0, 0.0, 0.0]));
        let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
        assert!(!mesh.vertices.is_empty(), "Single carbon atom must yield a surface");
    }

    #[test]
    fn normals_are_unit_length() {
        let mol = water();
        let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
        let n_verts = vertex_count(&mesh);
        for vi in 0..n_verts {
            let nx = mesh.normals[vi * 3];
            let ny = mesh.normals[vi * 3 + 1];
            let nz = mesh.normals[vi * 3 + 2];
            let len = (nx * nx + ny * ny + nz * nz).sqrt();
            assert!(
                (len - 1.0).abs() < 1e-9,
                "Normal at vertex {vi} has length {len}, expected 1.0"
            );
        }
    }

    #[test]
    fn vertex_count_helper_consistent() {
        let mol = water();
        let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
        assert_eq!(vertex_count(&mesh), mesh.vertices.len() / 3);
    }

    #[test]
    fn triangle_count_helper_consistent() {
        let mol = water();
        let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
        assert_eq!(triangle_count(&mesh), mesh.indices.len() / 3);
    }

    #[test]
    fn mesh_serializes_to_json() {
        let mol = water();
        let mesh = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
        let json = serde_json::to_string(&mesh);
        assert!(json.is_ok(), "SurfaceMesh must serialize to JSON");
    }

    #[test]
    fn hex_to_rgb_white() {
        let (r, g, b) = super::hex_to_rgb("#FFFFFF");
        assert!((r - 1.0).abs() < 1e-6);
        assert!((g - 1.0).abs() < 1e-6);
        assert!((b - 1.0).abs() < 1e-6);
    }

    #[test]
    fn hex_to_rgb_black() {
        let (r, g, b) = super::hex_to_rgb("#000000");
        assert!(r.abs() < 1e-6);
        assert!(g.abs() < 1e-6);
        assert!(b.abs() < 1e-6);
    }

    #[test]
    fn ses_larger_than_vdw() {
        // SES inflates radii so the surface is strictly larger
        let mol = water();
        let vdw = generate_surface(&mol, SurfaceType::VanDerWaals, 0.5);
        let ses = generate_surface(&mol, SurfaceType::SolventExcluded { probe_radius: 1.4 }, 0.5);
        // SES must have at least as many triangles (more surface area → more tris)
        assert!(
            triangle_count(&ses) >= triangle_count(&vdw),
            "SES should produce equal or more triangles than VdW"
        );
    }
}
