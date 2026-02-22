//! Computational topology for molecular surface analysis and mesh characterization.
//!
//! Computes topological invariants (Euler characteristic, genus, Betti numbers),
//! mesh connectivity analysis, discrete curvature estimation (Gaussian and mean),
//! and verifies Gauss-Bonnet theorem for closed surfaces.
//!
//! Primitive formula: topology = N(euler_chi) × ∂(boundary) + ρ(connectivity)
//!
//! # Overview
//!
//! The module operates on [`TriangleMesh`] — a list of 3-D vertices and
//! index-triplet triangles with CCW winding.  From that input the pipeline:
//!
//! 1. Extracts unique edges and counts V, E, F to obtain χ = V − E + F.
//! 2. Builds full vertex/triangle adjacency via [`build_connectivity`].
//! 3. Identifies boundary edges (appear in exactly one triangle) and the
//!    loops they form.
//! 4. Derives genus g = (2C − χ − b) / 2 for C components, b boundary loops.
//! 5. Computes Betti numbers (β₀, β₁, β₂) from the Euler–Poincaré relation.
//! 6. Estimates Gaussian curvature K at each vertex via the angle-deficit
//!    method and mean curvature H via the cotangent-weight formula.
//! 7. Verifies the Gauss–Bonnet theorem: Σ K = 2π χ (5 % tolerance).
//!
//! All operations are O(V + E + F) and allocation-only — zero unsafe,
//! zero panics, all fallible paths return Result or Option.
//!
//! # Example
//!
//! ```
//! use nexcore_viz::topology::{TriangleMesh, compute_invariants};
//!
//! // Tetrahedron: 4 vertices, 4 faces, χ = 2
//! let mesh = TriangleMesh {
//!     vertices: vec![
//!         [0.0, 0.0, 0.0],
//!         [1.0, 0.0, 0.0],
//!         [0.5, 1.0, 0.0],
//!         [0.5, 0.5, 1.0],
//!     ],
//!     triangles: vec![
//!         [0, 1, 2],
//!         [0, 1, 3],
//!         [0, 2, 3],
//!         [1, 2, 3],
//!     ],
//! };
//! if let Ok(inv) = compute_invariants(&mesh) {
//!     assert_eq!(inv.euler_characteristic, 2);
//!     assert_eq!(inv.genus, 0);
//! }
//! ```

use serde::{Deserialize, Serialize};
use std::collections::{HashMap, HashSet, VecDeque};

// ============================================================================
// Public types
// ============================================================================

/// A triangle mesh for topological analysis.
///
/// Triangles are stored as CCW-wound vertex-index triplets.  All indices must
/// be valid positions into `vertices`; invalid indices are reported as
/// [`TopologyError::InvalidTriangle`].
///
/// # Example
///
/// ```
/// use nexcore_viz::topology::TriangleMesh;
///
/// let mesh = TriangleMesh {
///     vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
///     triangles: vec![[0, 1, 2]],
/// };
/// assert_eq!(mesh.vertex_count(), 3);
/// assert_eq!(mesh.triangle_count(), 1);
/// assert_eq!(mesh.edge_count(), 3);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TriangleMesh {
    /// 3-D vertex positions.
    pub vertices: Vec<[f64; 3]>,
    /// Triangle index triplets (CCW winding, 0-based into `vertices`).
    pub triangles: Vec<[usize; 3]>,
}

impl TriangleMesh {
    /// Number of vertices in the mesh.
    #[must_use]
    pub fn vertex_count(&self) -> usize {
        self.vertices.len()
    }

    /// Number of triangles in the mesh.
    #[must_use]
    pub fn triangle_count(&self) -> usize {
        self.triangles.len()
    }

    /// Number of unique undirected edges across all triangles.
    ///
    /// Computed fresh each call; cache the result if called in a hot path.
    #[must_use]
    pub fn edge_count(&self) -> usize {
        unique_edges(self).len()
    }
}

// ----------------------------------------------------------------------------

/// Topological invariants computed from a [`TriangleMesh`].
///
/// For a compact orientable surface without boundary the classical relations
/// hold:
/// - χ = V − E + F
/// - χ = 2C − 2g − b   (C components, g genus, b boundary loops)
/// - β₀ = C, β₁ = 2g + b, β₂ = C_closed
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TopologicalInvariants {
    /// Euler characteristic χ = V − E + F.
    pub euler_characteristic: i64,
    /// Genus (number of handles) for the orientable surface.
    pub genus: i64,
    /// β₀ — number of connected components.
    pub betti_0: usize,
    /// β₁ — number of independent loops (1-cycles).
    pub betti_1: usize,
    /// β₂ — number of enclosed voids (2-cycles / closed components).
    pub betti_2: usize,
    /// `true` if every edge is shared by exactly two triangles (no boundary).
    pub is_closed: bool,
    /// `true` if the mesh is orientable (simplified: `true` for manifold inputs).
    pub is_orientable: bool,
    /// Number of distinct boundary loops.
    pub boundary_loops: usize,
}

// ----------------------------------------------------------------------------

/// Discrete curvature estimates at a single mesh vertex.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct VertexCurvature {
    /// 0-based index into [`TriangleMesh::vertices`].
    pub vertex_index: usize,
    /// Gaussian curvature K (angle-deficit method).
    pub gaussian: f64,
    /// Mean curvature H (cotangent-weight formula).
    pub mean: f64,
    /// Larger principal curvature κ₁ = H + sqrt(H² − K).
    pub principal_max: f64,
    /// Smaller principal curvature κ₂ = H − sqrt(H² − K).
    pub principal_min: f64,
}

// ----------------------------------------------------------------------------

/// Adjacency and incidence information derived from a [`TriangleMesh`].
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MeshConnectivity {
    /// `vertex_neighbors[v]` — indices of vertices adjacent to `v`.
    pub vertex_neighbors: Vec<Vec<usize>>,
    /// `vertex_triangles[v]` — indices of triangles incident on `v`.
    pub vertex_triangles: Vec<Vec<usize>>,
    /// Vertices that lie on at least one boundary edge.
    pub boundary_vertices: Vec<usize>,
    /// Boundary edges as (min, max) ordered vertex-index pairs.
    pub boundary_edges: Vec<(usize, usize)>,
    /// Connected components — each entry is a sorted list of vertex indices.
    pub components: Vec<Vec<usize>>,
}

// ----------------------------------------------------------------------------

/// Result of verifying the Gauss–Bonnet theorem on a closed mesh.
///
/// For a closed surface: Σ K = 2π χ.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GaussBonnetResult {
    /// Sum of all vertex Gaussian curvatures.
    pub total_gaussian_curvature: f64,
    /// Theoretical value 2π × χ.
    pub expected_value: f64,
    /// |Σ K − expected| / |expected|  (0.0 when expected = 0 and sum = 0).
    pub relative_error: f64,
    /// `true` if `relative_error < 0.05`.
    pub passes: bool,
}

// ----------------------------------------------------------------------------

/// Errors that can arise during topological computation.
#[derive(Debug, Clone)]
pub enum TopologyError {
    /// Mesh has no vertices or no triangles.
    EmptyMesh,
    /// A triangle references a vertex index that is out of range.
    InvalidTriangle(String),
    /// A triangle is degenerate (zero area).
    DegenerateTriangle(usize),
}

impl std::fmt::Display for TopologyError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::EmptyMesh => write!(f, "mesh has no vertices or triangles"),
            Self::InvalidTriangle(msg) => write!(f, "invalid triangle: {msg}"),
            Self::DegenerateTriangle(idx) => {
                write!(f, "triangle {idx} is degenerate (zero area)")
            }
        }
    }
}

impl std::error::Error for TopologyError {}

// ============================================================================
// Public API — edges
// ============================================================================

/// Extract all unique undirected edges from a mesh.
///
/// Each edge is returned as `(min_idx, max_idx)`.  Duplicate edges across
/// triangles are deduplicated.
///
/// # Example
///
/// ```
/// use nexcore_viz::topology::{TriangleMesh, unique_edges};
///
/// let mesh = TriangleMesh {
///     vertices: vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],
///     triangles: vec![[0,1,2]],
/// };
/// let edges = unique_edges(&mesh);
/// assert_eq!(edges.len(), 3);
/// ```
#[must_use]
pub fn unique_edges(mesh: &TriangleMesh) -> Vec<(usize, usize)> {
    let mut seen: HashSet<(usize, usize)> = HashSet::new();
    for &[a, b, c] in &mesh.triangles {
        for &(u, v) in &[(a, b), (b, c), (c, a)] {
            let key = if u < v { (u, v) } else { (v, u) };
            seen.insert(key);
        }
    }
    let mut edges: Vec<(usize, usize)> = seen.into_iter().collect();
    edges.sort_unstable();
    edges
}

// ============================================================================
// Public API — Euler characteristic
// ============================================================================

/// Compute the Euler characteristic χ = V − E + F.
///
/// # Example
///
/// ```
/// use nexcore_viz::topology::{TriangleMesh, euler_characteristic};
///
/// // Tetrahedron: V=4, E=6, F=4 → χ=2
/// let mesh = TriangleMesh {
///     vertices: vec![
///         [0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[0.5,0.5,1.0],
///     ],
///     triangles: vec![[0,1,2],[0,1,3],[0,2,3],[1,2,3]],
/// };
/// assert_eq!(euler_characteristic(&mesh), 2);
/// ```
#[must_use]
pub fn euler_characteristic(mesh: &TriangleMesh) -> i64 {
    let v = mesh.vertex_count() as i64;
    let e = unique_edges(mesh).len() as i64;
    let f = mesh.triangle_count() as i64;
    v - e + f
}

// ============================================================================
// Public API — genus
// ============================================================================

/// Derive the genus from the Euler characteristic.
///
/// Uses χ = 2C − 2g − b  →  g = (2C − χ − b) / 2.
///
/// Returns 0 when the formula produces a negative result (can occur for
/// non-manifold or open meshes with complex topology).
///
/// # Arguments
///
/// * `chi`           — Euler characteristic.
/// * `components`    — Number of connected components (β₀).
/// * `boundary_loops` — Number of distinct boundary loops.
///
/// # Example
///
/// ```
/// use nexcore_viz::topology::compute_genus;
///
/// // Sphere: χ=2, 1 component, 0 boundary → g=0
/// assert_eq!(compute_genus(2, 1, 0), 0);
/// // Torus: χ=0, 1 component, 0 boundary → g=1
/// assert_eq!(compute_genus(0, 1, 0), 1);
/// ```
#[must_use]
pub fn compute_genus(chi: i64, components: usize, boundary_loops: usize) -> i64 {
    let c = components as i64;
    let b = boundary_loops as i64;
    let numerator = 2 * c - chi - b;
    if numerator <= 0 {
        return 0;
    }
    numerator / 2
}

// ============================================================================
// Public API — Betti numbers
// ============================================================================

/// Compute Betti numbers (β₀, β₁, β₂) from topological data.
///
/// Uses the Euler–Poincaré formula: χ = β₀ − β₁ + β₂.
///
/// * β₀ = `components`
/// * β₂ = number of components that are closed (no boundary)
/// * β₁ = β₀ + β₂ − χ
///
/// # Example
///
/// ```
/// use nexcore_viz::topology::compute_betti_numbers;
///
/// // Sphere: χ=2, 1 component, closed → (1, 0, 1)
/// assert_eq!(compute_betti_numbers(2, 1, true), (1, 0, 1));
/// // Torus: χ=0, 1 component, closed → (1, 2, 1)
/// assert_eq!(compute_betti_numbers(0, 1, true), (1, 2, 1));
/// ```
#[must_use]
pub fn compute_betti_numbers(chi: i64, components: usize, is_closed: bool) -> (usize, usize, usize) {
    let b0 = components;
    let b2 = if is_closed { components } else { 0_usize };
    // β₁ = β₀ + β₂ − χ  (Euler–Poincaré)
    let b1_signed = b0 as i64 + b2 as i64 - chi;
    let b1 = b1_signed.max(0) as usize;
    (b0, b1, b2)
}

// ============================================================================
// Public API — connectivity
// ============================================================================

/// Build vertex-to-vertex and vertex-to-triangle adjacency for a mesh.
///
/// Also identifies boundary edges (shared by exactly one triangle) and the
/// vertices that lie on them, and finds connected components via BFS.
///
/// # Errors
///
/// Returns [`TopologyError::EmptyMesh`] when the mesh has no vertices or no
/// triangles.  Returns [`TopologyError::InvalidTriangle`] if any triangle
/// references an out-of-range vertex index.
///
/// # Example
///
/// ```
/// use nexcore_viz::topology::{TriangleMesh, build_connectivity};
///
/// let mesh = TriangleMesh {
///     vertices: vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],
///     triangles: vec![[0,1,2]],
/// };
/// if let Ok(conn) = build_connectivity(&mesh) {
///     assert_eq!(conn.components.len(), 1);
///     // single triangle — all 3 edges are boundary
///     assert_eq!(conn.boundary_edges.len(), 3);
/// }
/// ```
pub fn build_connectivity(mesh: &TriangleMesh) -> Result<MeshConnectivity, TopologyError> {
    let v_count = mesh.vertex_count();
    if v_count == 0 || mesh.triangles.is_empty() {
        return Err(TopologyError::EmptyMesh);
    }

    // Validate triangle indices upfront.
    for (ti, &[a, b, c]) in mesh.triangles.iter().enumerate() {
        if a >= v_count || b >= v_count || c >= v_count {
            return Err(TopologyError::InvalidTriangle(format!(
                "triangle {ti} references vertex out of range [a={a}, b={b}, c={c}], \
                 vertex_count={v_count}"
            )));
        }
    }

    // --- vertex_neighbors and vertex_triangles ---
    let mut vertex_neighbors: Vec<HashSet<usize>> = vec![HashSet::new(); v_count];
    let mut vertex_triangles: Vec<Vec<usize>> = vec![Vec::new(); v_count];

    // Count how many triangles share each undirected edge.
    // Maps (min, max) → incident triangle count.
    let mut edge_count: HashMap<(usize, usize), usize> = HashMap::new();

    for (ti, &[a, b, c]) in mesh.triangles.iter().enumerate() {
        // Incidence
        vertex_triangles[a].push(ti);
        vertex_triangles[b].push(ti);
        vertex_triangles[c].push(ti);

        // Adjacency (symmetric)
        vertex_neighbors[a].insert(b);
        vertex_neighbors[a].insert(c);
        vertex_neighbors[b].insert(a);
        vertex_neighbors[b].insert(c);
        vertex_neighbors[c].insert(a);
        vertex_neighbors[c].insert(b);

        // Edge incidence count
        for &(u, v) in &[(a, b), (b, c), (c, a)] {
            let key = if u < v { (u, v) } else { (v, u) };
            *edge_count.entry(key).or_insert(0) += 1;
        }
    }

    // Convert neighbor sets to sorted vecs.
    let vertex_neighbors: Vec<Vec<usize>> = vertex_neighbors
        .into_iter()
        .map(|set| {
            let mut v: Vec<usize> = set.into_iter().collect();
            v.sort_unstable();
            v
        })
        .collect();

    // --- Boundary edges and vertices ---
    let mut boundary_edges: Vec<(usize, usize)> = edge_count
        .iter()
        .filter_map(|(&e, &cnt)| if cnt == 1 { Some(e) } else { None })
        .collect();
    boundary_edges.sort_unstable();

    let mut boundary_vertex_set: HashSet<usize> = HashSet::new();
    for &(u, v) in &boundary_edges {
        boundary_vertex_set.insert(u);
        boundary_vertex_set.insert(v);
    }
    let mut boundary_vertices: Vec<usize> = boundary_vertex_set.into_iter().collect();
    boundary_vertices.sort_unstable();

    // --- Connected components via BFS ---
    let mut visited = vec![false; v_count];
    let mut components: Vec<Vec<usize>> = Vec::new();

    for start in 0..v_count {
        if visited[start] {
            continue;
        }
        let mut component: Vec<usize> = Vec::new();
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(start);
        visited[start] = true;

        while let Some(v) = queue.pop_front() {
            component.push(v);
            for &nb in vertex_neighbors.get(v).map(Vec::as_slice).unwrap_or(&[]) {
                if !visited[nb] {
                    visited[nb] = true;
                    queue.push_back(nb);
                }
            }
        }
        component.sort_unstable();
        components.push(component);
    }

    Ok(MeshConnectivity {
        vertex_neighbors,
        vertex_triangles,
        boundary_vertices,
        boundary_edges,
        components,
    })
}

// ============================================================================
// Public API — full invariants
// ============================================================================

/// Compute all topological invariants for a mesh.
///
/// Orchestrates edge extraction, Euler characteristic, connectivity analysis,
/// genus, and Betti numbers.
///
/// # Errors
///
/// Propagates errors from [`build_connectivity`].
///
/// # Example
///
/// ```
/// use nexcore_viz::topology::{TriangleMesh, compute_invariants};
///
/// let mesh = TriangleMesh {
///     vertices: vec![
///         [0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[0.5,0.5,1.0],
///     ],
///     triangles: vec![[0,1,2],[0,1,3],[0,2,3],[1,2,3]],
/// };
/// if let Ok(inv) = compute_invariants(&mesh) {
///     assert_eq!(inv.euler_characteristic, 2);
///     assert_eq!(inv.genus, 0);
///     assert!(inv.is_closed);
/// }
/// ```
pub fn compute_invariants(mesh: &TriangleMesh) -> Result<TopologicalInvariants, TopologyError> {
    let conn = build_connectivity(mesh)?;
    let chi = euler_characteristic(mesh);
    let is_closed = conn.boundary_edges.is_empty();
    let boundary_loops = count_boundary_loops(&conn.boundary_edges);
    let components = conn.components.len();
    let genus = compute_genus(chi, components, boundary_loops);
    let (betti_0, betti_1, betti_2) = compute_betti_numbers(chi, components, is_closed);

    Ok(TopologicalInvariants {
        euler_characteristic: chi,
        genus,
        betti_0,
        betti_1,
        betti_2,
        is_closed,
        is_orientable: true, // manifold meshes are assumed orientable
        boundary_loops,
    })
}

// ============================================================================
// Public API — curvature
// ============================================================================

/// Compute Gaussian curvature at a vertex using the angle-deficit method.
///
/// * Interior vertex: K(v) = 2π − Σ θᵢ
/// * Boundary vertex: K(v) = π − Σ θᵢ
///
/// where θᵢ is the interior angle at `v` in each incident triangle.
///
/// # Example
///
/// ```
/// use nexcore_viz::topology::{TriangleMesh, build_connectivity, gaussian_curvature};
///
/// let mesh = TriangleMesh {
///     vertices: vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],
///     triangles: vec![[0,1,2]],
/// };
/// if let Ok(conn) = build_connectivity(&mesh) {
///     // Boundary vertex of a right triangle: K = π - π/2 = π/2
///     let k = gaussian_curvature(&mesh, 0, &conn);
///     let expected = std::f64::consts::FRAC_PI_2;
///     assert!((k - expected).abs() < 1e-6, "boundary vertex curvature");
/// }
/// ```
#[must_use]
pub fn gaussian_curvature(mesh: &TriangleMesh, vertex: usize, conn: &MeshConnectivity) -> f64 {
    let is_boundary = conn.boundary_vertices.binary_search(&vertex).is_ok();
    let base_angle = if is_boundary {
        std::f64::consts::PI
    } else {
        2.0 * std::f64::consts::PI
    };

    let angle_sum: f64 = conn
        .vertex_triangles
        .get(vertex)
        .map(Vec::as_slice)
        .unwrap_or(&[])
        .iter()
        .filter_map(|&ti| {
            let [a, b, c] = *mesh.triangles.get(ti)?;
            let v_pos = mesh.vertices.get(vertex)?;

            // Find the other two vertices of this triangle relative to `vertex`.
            let (p, q) = if a == vertex {
                (mesh.vertices.get(b)?, mesh.vertices.get(c)?)
            } else if b == vertex {
                (mesh.vertices.get(a)?, mesh.vertices.get(c)?)
            } else {
                (mesh.vertices.get(a)?, mesh.vertices.get(b)?)
            };

            Some(triangle_angle(v_pos, p, q))
        })
        .sum();

    base_angle - angle_sum
}

/// Compute mean curvature at a vertex using the cotangent-weight formula.
///
/// H = |Σᵢ (cot αᵢ + cot βᵢ)(vᵢ − v)| / (4 A)
///
/// where for each edge (v, vᵢ): αᵢ and βᵢ are the angles opposite that edge
/// in the two triangles sharing it, and A is the mixed Voronoi area.  When a
/// neighbouring triangle does not exist (boundary edge), only the single
/// cotangent is used.
///
/// # Example
///
/// ```
/// use nexcore_viz::topology::{TriangleMesh, build_connectivity, mean_curvature};
///
/// let mesh = TriangleMesh {
///     vertices: vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],
///     triangles: vec![[0,1,2]],
/// };
/// if let Ok(conn) = build_connectivity(&mesh) {
///     let h = mean_curvature(&mesh, 0, &conn);
///     assert!(h.is_finite(), "mean curvature must be finite");
/// }
/// ```
#[must_use]
pub fn mean_curvature(mesh: &TriangleMesh, vertex: usize, conn: &MeshConnectivity) -> f64 {
    let v_pos = match mesh.vertices.get(vertex) {
        Some(p) => *p,
        None => return 0.0,
    };

    let neighbors = conn
        .vertex_triangles
        .get(vertex)
        .map(Vec::as_slice)
        .unwrap_or(&[]);

    let mut cot_vec = [0.0_f64; 3]; // Laplace–Beltrami accumulator
    let mut area_sum = 0.0_f64;

    // For each edge (vertex, nb) accumulate cotangent weights.
    let mut edge_cots: HashMap<usize, f64> = HashMap::new();

    for &ti in neighbors {
        let &[a, b, c] = match mesh.triangles.get(ti) {
            Some(t) => t,
            None => continue,
        };

        // Identify the two neighbours of `vertex` in this triangle.
        let (nb1, nb2) = if a == vertex {
            (b, c)
        } else if b == vertex {
            (a, c)
        } else {
            (a, b)
        };

        let pos_nb1 = match mesh.vertices.get(nb1) {
            Some(p) => *p,
            None => continue,
        };
        let pos_nb2 = match mesh.vertices.get(nb2) {
            Some(p) => *p,
            None => continue,
        };

        // Angles opposite each edge in this triangle:
        // - opposite (vertex, nb1) → angle at nb2
        // - opposite (vertex, nb2) → angle at nb1
        let angle_opp_nb1 = triangle_angle(&pos_nb2, &v_pos, &pos_nb1);
        let angle_opp_nb2 = triangle_angle(&pos_nb1, &v_pos, &pos_nb2);

        *edge_cots.entry(nb1).or_insert(0.0) += cot(angle_opp_nb1);
        *edge_cots.entry(nb2).or_insert(0.0) += cot(angle_opp_nb2);

        // Voronoi area contribution from this triangle.
        area_sum += triangle_area(&v_pos, &pos_nb1, &pos_nb2);
    }

    if area_sum < 1e-14 {
        return 0.0;
    }

    // Laplace–Beltrami operator applied to position.
    for (&nb, &w) in &edge_cots {
        let pos_nb = match mesh.vertices.get(nb) {
            Some(p) => *p,
            None => continue,
        };
        let diff = [
            pos_nb[0] - v_pos[0],
            pos_nb[1] - v_pos[1],
            pos_nb[2] - v_pos[2],
        ];
        cot_vec[0] += w * diff[0];
        cot_vec[1] += w * diff[1];
        cot_vec[2] += w * diff[2];
    }

    // |H| = |Δx| / (4A)
    let len = (cot_vec[0] * cot_vec[0] + cot_vec[1] * cot_vec[1] + cot_vec[2] * cot_vec[2]).sqrt();
    len / (4.0 * area_sum)
}

/// Compute Gaussian and mean curvature at every vertex of a mesh.
///
/// Principal curvatures are κ₁ = H + sqrt(H² − K) and κ₂ = H − sqrt(H² − K).
/// The discriminant is clamped to ≥ 0 to avoid NaN from floating-point
/// rounding when H² ≈ K.
///
/// # Errors
///
/// Propagates errors from [`build_connectivity`].
///
/// # Example
///
/// ```
/// use nexcore_viz::topology::{TriangleMesh, compute_all_curvatures};
///
/// let mesh = TriangleMesh {
///     vertices: vec![[0.0,0.0,0.0],[1.0,0.0,0.0],[0.0,1.0,0.0]],
///     triangles: vec![[0,1,2]],
/// };
/// if let Ok(curvatures) = compute_all_curvatures(&mesh) {
///     assert_eq!(curvatures.len(), 3);
/// }
/// ```
pub fn compute_all_curvatures(
    mesh: &TriangleMesh,
) -> Result<Vec<VertexCurvature>, TopologyError> {
    let conn = build_connectivity(mesh)?;
    let mut result = Vec::with_capacity(mesh.vertex_count());

    for vi in 0..mesh.vertex_count() {
        let k = gaussian_curvature(mesh, vi, &conn);
        let h = mean_curvature(mesh, vi, &conn);
        let discriminant = (h * h - k).max(0.0);
        let sqrt_d = discriminant.sqrt();
        result.push(VertexCurvature {
            vertex_index: vi,
            gaussian: k,
            mean: h,
            principal_max: h + sqrt_d,
            principal_min: h - sqrt_d,
        });
    }

    Ok(result)
}

// ============================================================================
// Public API — Gauss–Bonnet
// ============================================================================

/// Verify the Gauss–Bonnet theorem: Σ K ≈ 2π χ.
///
/// Passes when the relative error is below 5 % — a practical tolerance for
/// the discrete angle-deficit approximation on coarse meshes.
///
/// When `expected_value` is zero (χ = 0) and the sum is also zero, the
/// result is marked as passing with `relative_error = 0.0`.
///
/// # Example
///
/// ```
/// use nexcore_viz::topology::{TriangleMesh, compute_all_curvatures, verify_gauss_bonnet};
///
/// let mesh = TriangleMesh {
///     vertices: vec![
///         [0.0,0.0,0.0],[1.0,0.0,0.0],[0.5,1.0,0.0],[0.5,0.5,1.0],
///     ],
///     triangles: vec![[0,1,2],[0,1,3],[0,2,3],[1,2,3]],
/// };
/// if let Ok(curvatures) = compute_all_curvatures(&mesh) {
///     let gb = verify_gauss_bonnet(&mesh, &curvatures);
///     assert!(gb.passes, "tetrahedron should satisfy Gauss–Bonnet (χ=2 → ΣK≈4π)");
/// }
/// ```
#[must_use]
pub fn verify_gauss_bonnet(mesh: &TriangleMesh, curvatures: &[VertexCurvature]) -> GaussBonnetResult {
    let total_gaussian_curvature: f64 = curvatures.iter().map(|c| c.gaussian).sum();
    let chi = euler_characteristic(mesh);
    let expected_value = 2.0 * std::f64::consts::PI * chi as f64;

    let relative_error = if expected_value.abs() < 1e-12 {
        if total_gaussian_curvature.abs() < 1e-12 {
            0.0
        } else {
            1.0 // expected=0 but sum≠0 → full error
        }
    } else {
        (total_gaussian_curvature - expected_value).abs() / expected_value.abs()
    };

    GaussBonnetResult {
        total_gaussian_curvature,
        expected_value,
        relative_error,
        passes: relative_error < 0.05,
    }
}

// ============================================================================
// Private helpers
// ============================================================================

/// Compute the interior angle at vertex `v` in a triangle with the other two
/// vertices at `a` and `b`.  Returns 0.0 for degenerate configurations.
fn triangle_angle(v: &[f64; 3], a: &[f64; 3], b: &[f64; 3]) -> f64 {
    let ea = [a[0] - v[0], a[1] - v[1], a[2] - v[2]];
    let eb = [b[0] - v[0], b[1] - v[1], b[2] - v[2]];

    let len_a = (ea[0] * ea[0] + ea[1] * ea[1] + ea[2] * ea[2]).sqrt();
    let len_b = (eb[0] * eb[0] + eb[1] * eb[1] + eb[2] * eb[2]).sqrt();

    if len_a < 1e-14 || len_b < 1e-14 {
        return 0.0;
    }

    let dot = ea[0] * eb[0] + ea[1] * eb[1] + ea[2] * eb[2];
    let cos_theta = (dot / (len_a * len_b)).clamp(-1.0, 1.0);
    cos_theta.acos()
}

/// Compute the area of triangle (a, b, c) via the cross-product magnitude.
fn triangle_area(a: &[f64; 3], b: &[f64; 3], c: &[f64; 3]) -> f64 {
    let ab = [b[0] - a[0], b[1] - a[1], b[2] - a[2]];
    let ac = [c[0] - a[0], c[1] - a[1], c[2] - a[2]];

    // Cross product ab × ac.
    let cx = ab[1] * ac[2] - ab[2] * ac[1];
    let cy = ab[2] * ac[0] - ab[0] * ac[2];
    let cz = ab[0] * ac[1] - ab[1] * ac[0];

    (cx * cx + cy * cy + cz * cz).sqrt() * 0.5
}

/// Return a unit vector in the direction of `v`.  Returns zero vector when
/// `v` has near-zero magnitude.
fn normalize(v: &[f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-14 {
        [0.0, 0.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Cotangent of an angle (cos / sin).  Clamped to ±1e6 to avoid blow-up.
fn cot(angle: f64) -> f64 {
    let s = angle.sin();
    if s.abs() < 1e-10 {
        return if angle.cos() >= 0.0 { 1e6 } else { -1e6 };
    }
    (angle.cos() / s).clamp(-1e6, 1e6)
}

/// Count distinct boundary loops by tracing chains of boundary edges.
///
/// A boundary loop is a connected cycle of boundary edges.  The algorithm
/// builds an adjacency list restricted to boundary edges and counts connected
/// components among boundary vertices.
fn count_boundary_loops(boundary_edges: &[(usize, usize)]) -> usize {
    if boundary_edges.is_empty() {
        return 0;
    }

    // Build adjacency among boundary vertices.
    let mut adj: HashMap<usize, Vec<usize>> = HashMap::new();
    for &(u, v) in boundary_edges {
        adj.entry(u).or_default().push(v);
        adj.entry(v).or_default().push(u);
    }

    let mut visited: HashSet<usize> = HashSet::new();
    let mut loops = 0_usize;

    for &start in adj.keys() {
        if visited.contains(&start) {
            continue;
        }
        let mut queue: VecDeque<usize> = VecDeque::new();
        queue.push_back(start);
        visited.insert(start);
        while let Some(v) = queue.pop_front() {
            for &nb in adj.get(&v).map(Vec::as_slice).unwrap_or(&[]) {
                if !visited.contains(&nb) {
                    visited.insert(nb);
                    queue.push_back(nb);
                }
            }
        }
        loops += 1;
    }

    loops
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    // --- Mesh builders -------------------------------------------------------

    /// Regular tetrahedron (vertices on unit sphere, scaled by 1/√3).
    ///
    /// V=4, E=6, F=4  →  χ = 2
    fn make_tetrahedron() -> TriangleMesh {
        TriangleMesh {
            vertices: vec![
                [1.0, 1.0, 1.0],
                [1.0, -1.0, -1.0],
                [-1.0, 1.0, -1.0],
                [-1.0, -1.0, 1.0],
            ],
            // CCW winding viewed from outside.
            triangles: vec![[0, 2, 1], [0, 1, 3], [0, 3, 2], [1, 2, 3]],
        }
    }

    /// Two triangles sharing an edge — a flat open quad.
    ///
    /// V=4, E=5, F=2  →  χ = 1
    fn make_open_plane() -> TriangleMesh {
        TriangleMesh {
            vertices: vec![
                [0.0, 0.0, 0.0],
                [1.0, 0.0, 0.0],
                [1.0, 1.0, 0.0],
                [0.0, 1.0, 0.0],
            ],
            triangles: vec![[0, 1, 2], [0, 2, 3]],
        }
    }

    // --- Euler characteristic ------------------------------------------------

    #[test]
    fn euler_tetrahedron() {
        // V=4, E=6, F=4 → χ=2
        let mesh = make_tetrahedron();
        assert_eq!(mesh.vertex_count(), 4);
        assert_eq!(mesh.edge_count(), 6);
        assert_eq!(mesh.triangle_count(), 4);
        assert_eq!(euler_characteristic(&mesh), 2);
    }

    #[test]
    fn euler_cube_surface() {
        // Triangulated cube surface: V=8, E=18, F=12 → χ=2
        let vertices = vec![
            [0.0, 0.0, 0.0],
            [1.0, 0.0, 0.0],
            [1.0, 1.0, 0.0],
            [0.0, 1.0, 0.0],
            [0.0, 0.0, 1.0],
            [1.0, 0.0, 1.0],
            [1.0, 1.0, 1.0],
            [0.0, 1.0, 1.0],
        ];
        // 6 faces × 2 triangles = 12 triangles total.
        let triangles = vec![
            [0, 1, 2], [0, 2, 3], // bottom  (z=0)
            [4, 6, 5], [4, 7, 6], // top     (z=1)
            [0, 1, 5], [0, 5, 4], // front   (y=0)
            [2, 3, 7], [2, 7, 6], // back    (y=1)
            [0, 3, 7], [0, 7, 4], // left    (x=0)
            [1, 2, 6], [1, 6, 5], // right   (x=1)
        ];
        let mesh = TriangleMesh { vertices, triangles };
        assert_eq!(euler_characteristic(&mesh), 2);
    }

    // --- Genus ---------------------------------------------------------------

    #[test]
    fn genus_sphere() {
        // χ=2, 1 component, 0 boundary → genus=0
        assert_eq!(compute_genus(2, 1, 0), 0);
    }

    #[test]
    fn genus_torus() {
        // χ=0, 1 component, 0 boundary → genus=1
        assert_eq!(compute_genus(0, 1, 0), 1);
    }

    // --- Betti numbers -------------------------------------------------------

    #[test]
    fn betti_sphere() {
        // Sphere: β₀=1, β₁=0, β₂=1
        let (b0, b1, b2) = compute_betti_numbers(2, 1, true);
        assert_eq!(b0, 1);
        assert_eq!(b1, 0);
        assert_eq!(b2, 1);
    }

    #[test]
    fn betti_torus() {
        // Torus: β₀=1, β₁=2, β₂=1
        let (b0, b1, b2) = compute_betti_numbers(0, 1, true);
        assert_eq!(b0, 1);
        assert_eq!(b1, 2);
        assert_eq!(b2, 1);
    }

    // --- Connectivity --------------------------------------------------------

    #[test]
    fn connectivity_triangle() {
        // Single triangle: 3 vertices, each with 2 neighbours.
        let mesh = TriangleMesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0], [0.0, 1.0, 0.0]],
            triangles: vec![[0, 1, 2]],
        };
        let result = build_connectivity(&mesh);
        assert!(result.is_ok(), "expected Ok from build_connectivity for single triangle");
        if let Ok(conn) = result {
            assert_eq!(conn.components.len(), 1);
            for vi in 0..3 {
                assert_eq!(
                    conn.vertex_neighbors[vi].len(),
                    2,
                    "vertex {vi} should have exactly 2 neighbours"
                );
            }
        }
    }

    // --- Boundary detection --------------------------------------------------

    #[test]
    fn boundary_detection_open_mesh() {
        let mesh = make_open_plane();
        let result = build_connectivity(&mesh);
        assert!(result.is_ok(), "expected Ok for open plane");
        if let Ok(conn) = result {
            // Open plane has 4 boundary edges (outer perimeter of the quad).
            assert!(
                !conn.boundary_edges.is_empty(),
                "open mesh must have boundary edges"
            );
            // Every vertex of the open quad lies on the boundary.
            assert_eq!(conn.boundary_vertices.len(), 4);
        }
    }

    // --- Gaussian curvature --------------------------------------------------

    #[test]
    fn gaussian_curvature_flat() {
        // Interior vertex of a flat fan: centre vertex 0 surrounded by 4
        // right-angle triangles forming a flat square.
        //
        //   4---3
        //   |\ /|
        //   | 0 |
        //   |/ \|
        //   1---2
        let mesh = TriangleMesh {
            vertices: vec![
                [0.0, 0.0, 0.0],   // 0 — centre
                [-1.0, -1.0, 0.0], // 1
                [1.0, -1.0, 0.0],  // 2
                [1.0, 1.0, 0.0],   // 3
                [-1.0, 1.0, 0.0],  // 4
            ],
            triangles: vec![[0, 1, 2], [0, 2, 3], [0, 3, 4], [0, 4, 1]],
        };
        let result = build_connectivity(&mesh);
        assert!(result.is_ok(), "expected Ok for flat fan mesh");
        if let Ok(conn) = result {
            let k = gaussian_curvature(&mesh, 0, &conn);
            // 4 right-angle triangles → angle sum at centre = 4 × π/2 = 2π → K ≈ 0
            assert!(
                k.abs() < 0.1,
                "planar interior vertex should have K≈0, got {k}"
            );
        }
    }

    #[test]
    fn gaussian_curvature_cone_tip() {
        // Cone apex: 3 equilateral triangles meeting at a point.
        // Each contributes a 60° angle → angle sum = 180° < 360° → K > 0.
        let h = 1.0_f64;
        let r = 1.0_f64;
        let mesh = TriangleMesh {
            vertices: vec![
                [0.0, 0.0, h],                            // 0 — apex
                [r, 0.0, 0.0],                            // 1
                [r * (-0.5_f64), r * 0.866_f64, 0.0],    // 2
                [r * (-0.5_f64), r * (-0.866_f64), 0.0], // 3
            ],
            triangles: vec![[0, 1, 2], [0, 2, 3], [0, 3, 1]],
        };
        let result = build_connectivity(&mesh);
        assert!(result.is_ok(), "expected Ok for cone mesh");
        if let Ok(conn) = result {
            let k = gaussian_curvature(&mesh, 0, &conn);
            assert!(k > 0.0, "cone tip should have positive Gaussian curvature, got {k}");
        }
    }

    // --- Gauss–Bonnet --------------------------------------------------------

    #[test]
    fn gauss_bonnet_tetrahedron() {
        // Tetrahedron: χ=2 → expected ΣK = 4π ≈ 12.566
        let mesh = make_tetrahedron();
        let result = compute_all_curvatures(&mesh);
        assert!(result.is_ok(), "expected Ok for tetrahedron curvatures");
        if let Ok(curvatures) = result {
            let gb = verify_gauss_bonnet(&mesh, &curvatures);
            assert!(
                gb.passes,
                "tetrahedron should satisfy Gauss–Bonnet: ΣK={:.4}, expected={:.4}, err={:.4}",
                gb.total_gaussian_curvature,
                gb.expected_value,
                gb.relative_error
            );
            let four_pi = 4.0 * std::f64::consts::PI;
            assert!(
                (gb.total_gaussian_curvature - four_pi).abs() < four_pi * 0.1,
                "ΣK should be near 4π={four_pi:.4}, got {:.4}",
                gb.total_gaussian_curvature
            );
        }
    }

    // --- Triangle area -------------------------------------------------------

    #[test]
    fn triangle_area_known() {
        // Right triangle with legs 3 and 4 → area = 6.0
        let a = [0.0, 0.0, 0.0];
        let b = [3.0, 0.0, 0.0];
        let c = [0.0, 4.0, 0.0];
        let area = triangle_area(&a, &b, &c);
        assert!(
            (area - 6.0).abs() < 1e-10,
            "right triangle 3-4 should have area 6.0, got {area}"
        );
    }

    // --- Additional coverage -------------------------------------------------

    #[test]
    fn compute_invariants_tetrahedron() {
        let mesh = make_tetrahedron();
        let result = compute_invariants(&mesh);
        assert!(result.is_ok(), "expected Ok for tetrahedron invariants");
        if let Ok(inv) = result {
            assert_eq!(inv.euler_characteristic, 2);
            assert_eq!(inv.genus, 0);
            assert!(inv.is_closed);
            assert!(inv.is_orientable);
            assert_eq!(inv.betti_0, 1);
            assert_eq!(inv.betti_1, 0);
            assert_eq!(inv.betti_2, 1);
        }
    }

    #[test]
    fn compute_invariants_open_plane() {
        let mesh = make_open_plane();
        let result = compute_invariants(&mesh);
        assert!(result.is_ok(), "expected Ok for open plane invariants");
        if let Ok(inv) = result {
            assert!(!inv.is_closed, "open plane should not be a closed surface");
            assert!(
                inv.boundary_loops > 0,
                "open plane must have at least one boundary loop"
            );
        }
    }

    #[test]
    fn empty_mesh_returns_error() {
        let mesh = TriangleMesh {
            vertices: vec![],
            triangles: vec![],
        };
        assert!(matches!(
            build_connectivity(&mesh),
            Err(TopologyError::EmptyMesh)
        ));
    }

    #[test]
    fn invalid_triangle_index_returns_error() {
        let mesh = TriangleMesh {
            vertices: vec![[0.0, 0.0, 0.0], [1.0, 0.0, 0.0]],
            triangles: vec![[0, 1, 99]], // vertex 99 does not exist
        };
        assert!(matches!(
            build_connectivity(&mesh),
            Err(TopologyError::InvalidTriangle(_))
        ));
    }

    #[test]
    fn unique_edges_deduplication() {
        // Two triangles sharing an edge: [0,1,2] and [0,2,3].
        // Edges: (0,1),(1,2),(0,2),(2,3),(0,3) → 5 unique edges.
        let mesh = make_open_plane();
        let edges = unique_edges(&mesh);
        assert_eq!(edges.len(), 5);
    }

    #[test]
    fn normalize_unit_vector() {
        let v = [3.0, 4.0, 0.0];
        let n = normalize(&v);
        let len = (n[0] * n[0] + n[1] * n[1] + n[2] * n[2]).sqrt();
        assert!((len - 1.0).abs() < 1e-12, "normalized vector should have length 1");
    }

    #[test]
    fn normalize_zero_vector_returns_zero() {
        let v = [0.0, 0.0, 0.0];
        let n = normalize(&v);
        assert_eq!(n, [0.0, 0.0, 0.0]);
    }

    #[test]
    fn all_curvatures_length_matches_vertex_count() {
        let mesh = make_tetrahedron();
        let result = compute_all_curvatures(&mesh);
        assert!(result.is_ok(), "expected Ok for tetrahedron curvatures");
        if let Ok(curvatures) = result {
            assert_eq!(curvatures.len(), mesh.vertex_count());
            for cv in &curvatures {
                assert!(cv.gaussian.is_finite());
                assert!(cv.mean.is_finite());
                assert!(cv.principal_max >= cv.principal_min);
            }
        }
    }
}
