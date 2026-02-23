//! Persistence computation via matrix reduction over Z/2Z.

use crate::diagram::{PersistenceDiagram, PersistencePoint};
use crate::simplex::SimplicialComplex;
use std::collections::HashMap;

/// Compute the persistence diagram from a filtered simplicial complex.
///
/// Uses the standard column-reduction algorithm (boundary matrix over Z/2Z).
/// The complex must be sorted by filtration value before calling this function;
/// [`SimplicialComplex::sort_by_filtration`] achieves this.
///
/// # Example
/// ```
/// use stem_topology::{DistanceMatrix, vietoris_rips, compute_persistence};
///
/// let dm = DistanceMatrix::new(vec![
///     vec![0.0, 1.0, 1.0],
///     vec![1.0, 0.0, 1.0],
///     vec![1.0, 1.0, 0.0],
/// ]);
/// let complex = vietoris_rips(&dm, 2, 2.0);
/// let diagram = compute_persistence(&complex);
/// assert!(!diagram.points.is_empty());
/// ```
pub fn compute_persistence(complex: &SimplicialComplex) -> PersistenceDiagram {
    let n = complex.simplices.len();
    let mut diagram = PersistenceDiagram::new();

    if n == 0 {
        return diagram;
    }

    // Map each simplex (by sorted vertex list) to its column index.
    let index_map: HashMap<Vec<usize>, usize> = complex
        .simplices
        .iter()
        .enumerate()
        .map(|(i, s)| (s.vertices.clone(), i))
        .collect();

    // Build boundary columns: each column = sorted list of face indices.
    let mut columns: Vec<Vec<usize>> = Vec::with_capacity(n);
    for simplex in &complex.simplices {
        let mut col = Vec::new();
        if simplex.dimension() > 0 {
            for skip in 0..simplex.vertices.len() {
                let face: Vec<usize> = simplex
                    .vertices
                    .iter()
                    .enumerate()
                    .filter(|&(i, _)| i != skip)
                    .map(|(_, v)| *v)
                    .collect();
                if let Some(&idx) = index_map.get(&face) {
                    col.push(idx);
                }
            }
            col.sort_unstable();
        }
        columns.push(col);
    }

    // Column reduction: standard persistence algorithm (Z/2Z).
    // pivot_col[low_value] = column index that owns that pivot row.
    let mut pivot_col: HashMap<usize, usize> = HashMap::new();

    for j in 0..n {
        loop {
            let low_j = columns[j].last().copied();
            match low_j {
                None => break,
                Some(l) => {
                    if let Some(&other) = pivot_col.get(&l) {
                        let other_col = columns[other].clone();
                        xor_columns(&mut columns[j], &other_col);
                    } else {
                        pivot_col.insert(l, j);
                        break;
                    }
                }
            }
        }
    }

    // Build reverse map: column_index -> pivot_row_index.
    // pivot_col maps low_row -> col; we need col -> low_row.
    let col_to_pivot: HashMap<usize, usize> =
        pivot_col.iter().map(|(&row, &col)| (col, row)).collect();

    // Collect pairing information.
    let mut paired = vec![false; n];

    for j in 0..n {
        if let Some(&i) = col_to_pivot.get(&j) {
            let birth = complex.simplices[i].filtration_value;
            let death = complex.simplices[j].filtration_value;
            let dim = complex.simplices[i].dimension();
            if (death - birth).abs() > f64::EPSILON {
                diagram.add_point(PersistencePoint::new(birth, death, dim));
            }
            paired[i] = true;
            paired[j] = true;
        }
    }

    // Unpaired simplices with zero boundary → essential (infinite) features.
    for (i, &is_paired) in paired.iter().enumerate() {
        if !is_paired && columns[i].is_empty() {
            let birth = complex.simplices[i].filtration_value;
            let dim = complex.simplices[i].dimension();
            diagram.add_point(PersistencePoint::new(birth, f64::INFINITY, dim));
        }
    }

    diagram
}

/// XOR two sorted column vectors (Z/2Z addition of boundary chains).
fn xor_columns(a: &mut Vec<usize>, b: &[usize]) {
    let mut result = Vec::new();
    let (mut i, mut j) = (0, 0);
    while i < a.len() && j < b.len() {
        match a[i].cmp(&b[j]) {
            std::cmp::Ordering::Less => {
                result.push(a[i]);
                i += 1;
            }
            std::cmp::Ordering::Greater => {
                result.push(b[j]);
                j += 1;
            }
            std::cmp::Ordering::Equal => {
                // Cancel in Z/2Z
                i += 1;
                j += 1;
            }
        }
    }
    result.extend_from_slice(&a[i..]);
    result.extend_from_slice(&b[j..]);
    *a = result;
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filtration::{DistanceMatrix, vietoris_rips};

    fn triangle_complex() -> SimplicialComplex {
        let dm = DistanceMatrix::new(vec![
            vec![0.0, 1.0, 1.0],
            vec![1.0, 0.0, 1.0],
            vec![1.0, 1.0, 0.0],
        ]);
        vietoris_rips(&dm, 2, 2.0)
    }

    #[test]
    fn persistence_on_triangle_not_empty() {
        let complex = triangle_complex();
        let diagram = compute_persistence(&complex);
        assert!(!diagram.points.is_empty());
    }

    #[test]
    fn persistence_empty_complex() {
        let complex = SimplicialComplex::new();
        let diagram = compute_persistence(&complex);
        assert!(diagram.points.is_empty());
    }

    #[test]
    fn persistence_triangle_has_betti0_component() {
        let complex = triangle_complex();
        let diagram = compute_persistence(&complex);
        let dim0 = diagram.points_of_dim(0);
        assert!(!dim0.is_empty(), "expected Betti-0 points");
    }

    #[test]
    fn xor_columns_cancel() {
        let mut a = vec![0usize, 2, 4];
        let b = vec![0usize, 2, 4];
        xor_columns(&mut a, &b);
        assert!(a.is_empty(), "XOR of identical columns should be empty");
    }

    #[test]
    fn xor_columns_symmetric_difference() {
        let mut a = vec![0usize, 1, 3];
        let b = vec![1usize, 2];
        xor_columns(&mut a, &b);
        assert_eq!(a, vec![0, 2, 3]);
    }
}
