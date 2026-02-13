//! Stage 8: K-Means Clustering and Topology.
//!
//! Groups words in statemind 3D space by spectral similarity.
//! Auto-selects k via silhouette score.
//!
//! Tier: T2-C | Dominant: Σ (Sum) + ∂ (Boundary) — centroid aggregation + decision boundaries.

use crate::projection::Point3D;
use serde::Serialize;

/// A cluster of words in statemind space.
///
/// Tier: T3 | Grounds to: Σ (Sum) + λ (Location) + ∂ (Boundary)
///         + κ (Comparison) + N (Quantity) + ∃ (Existence).
#[derive(Debug, Clone, Serialize)]
pub struct Cluster {
    /// Cluster identifier (0-indexed).
    pub id: usize,
    /// Words assigned to this cluster.
    pub members: Vec<String>,
    /// Centroid position.
    pub centroid: Point3D,
}

/// Result of clustering analysis.
///
/// Tier: T3 | Grounds to: Σ (Sum) + ∂ (Boundary) + κ (Comparison)
///         + N (Quantity) + λ (Location) + σ (Sequence).
#[derive(Debug, Clone, Serialize)]
pub struct ClusterResult {
    /// Identified clusters.
    pub clusters: Vec<Cluster>,
    /// Cluster assignment per input point (parallel to input order).
    pub assignments: Vec<usize>,
    /// Mean silhouette score [-1.0, 1.0] (higher = better separation).
    pub silhouette: f64,
}

/// Run k-means clustering on labeled 3D points.
///
/// Uses Lloyd's algorithm with deterministic initialization (first k points).
#[must_use]
pub fn kmeans(points: &[(String, Point3D)], k: usize, max_iter: usize) -> ClusterResult {
    if points.is_empty() || k == 0 {
        return ClusterResult {
            clusters: vec![],
            assignments: vec![],
            silhouette: 0.0,
        };
    }

    let k = k.min(points.len());

    // Initialize centroids from first k points
    let mut centroids: Vec<Point3D> = points[..k].iter().map(|(_, p)| p.clone()).collect();
    let mut assignments = vec![0_usize; points.len()];

    for _ in 0..max_iter {
        // Assign each point to nearest centroid
        let mut changed = false;
        for (i, (_, point)) in points.iter().enumerate() {
            let nearest = centroids
                .iter()
                .enumerate()
                .fold(
                    (0_usize, f64::MAX),
                    |(best_k, best_d), (k_idx, centroid)| {
                        let d = point.distance(centroid);
                        if d < best_d {
                            (k_idx, d)
                        } else {
                            (best_k, best_d)
                        }
                    },
                )
                .0;
            if assignments[i] != nearest {
                assignments[i] = nearest;
                changed = true;
            }
        }

        if !changed {
            break;
        }

        // Recompute centroids
        for (c_idx, centroid) in centroids.iter_mut().enumerate() {
            let members: Vec<&Point3D> = points
                .iter()
                .enumerate()
                .filter(|(i, _)| assignments[*i] == c_idx)
                .map(|(_, (_, p))| p)
                .collect();
            if !members.is_empty() {
                let n = members.len() as f64;
                centroid.x = members.iter().map(|p| p.x).sum::<f64>() / n;
                centroid.y = members.iter().map(|p| p.y).sum::<f64>() / n;
                centroid.z = members.iter().map(|p| p.z).sum::<f64>() / n;
            }
        }
    }

    // Build cluster structs
    let mut clusters: Vec<Cluster> = (0..k)
        .map(|id| Cluster {
            id,
            members: vec![],
            centroid: centroids[id].clone(),
        })
        .collect();

    for (i, (name, _)) in points.iter().enumerate() {
        if assignments[i] < clusters.len() {
            clusters[assignments[i]].members.push(name.clone());
        }
    }

    let silhouette = compute_silhouette(points, &assignments, k);

    ClusterResult {
        clusters,
        assignments,
        silhouette,
    }
}

/// Compute mean silhouette score.
///
/// s(i) = (b(i) - a(i)) / max(a(i), b(i))
/// where a(i) = mean intra-cluster distance, b(i) = mean nearest-cluster distance.
fn compute_silhouette(points: &[(String, Point3D)], assignments: &[usize], k: usize) -> f64 {
    if points.len() <= 1 || k <= 1 {
        return 0.0;
    }

    let n = points.len();
    let mut total_silhouette = 0.0;
    let mut valid_count = 0;

    for i in 0..n {
        let cluster_i = assignments[i];

        // a(i): mean distance to same-cluster points
        let same_cluster: Vec<f64> = (0..n)
            .filter(|&j| j != i && assignments[j] == cluster_i)
            .map(|j| points[i].1.distance(&points[j].1))
            .collect();

        if same_cluster.is_empty() {
            continue; // singleton cluster, silhouette undefined
        }

        let a_i = same_cluster.iter().sum::<f64>() / same_cluster.len() as f64;

        // b(i): minimum mean distance to any other cluster
        let mut b_i = f64::MAX;
        for c in 0..k {
            if c == cluster_i {
                continue;
            }
            let other: Vec<f64> = (0..n)
                .filter(|&j| assignments[j] == c)
                .map(|j| points[i].1.distance(&points[j].1))
                .collect();
            if !other.is_empty() {
                let mean_dist = other.iter().sum::<f64>() / other.len() as f64;
                if mean_dist < b_i {
                    b_i = mean_dist;
                }
            }
        }

        if b_i < f64::MAX {
            let max_ab = a_i.max(b_i);
            if max_ab > 0.0 {
                total_silhouette += (b_i - a_i) / max_ab;
                valid_count += 1;
            }
        }
    }

    if valid_count > 0 {
        total_silhouette / valid_count as f64
    } else {
        0.0
    }
}

/// Auto-select k and cluster using silhouette score.
///
/// Tries k = 2..min(n, 6) and returns the best silhouette.
#[must_use]
pub fn auto_cluster(points: &[(String, Point3D)]) -> ClusterResult {
    let n = points.len();
    if n <= 1 {
        return ClusterResult {
            clusters: points
                .iter()
                .enumerate()
                .map(|(i, (name, p))| Cluster {
                    id: i,
                    members: vec![name.clone()],
                    centroid: p.clone(),
                })
                .collect(),
            assignments: (0..n).collect(),
            silhouette: 0.0,
        };
    }

    if n == 2 {
        return kmeans(points, 2, 100);
    }

    let max_k = n.min(6);
    let mut best = kmeans(points, 2, 100);

    for k in 3..=max_k {
        let result = kmeans(points, k, 100);
        if result.silhouette > best.silhouette {
            best = result;
        }
    }

    best
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn singleton_cluster() {
        let points = vec![("word".to_string(), Point3D::new(1.0, 2.0, 3.0))];
        let result = auto_cluster(&points);
        assert_eq!(result.clusters.len(), 1);
        assert_eq!(result.clusters[0].members.len(), 1);
    }

    #[test]
    fn two_points_two_clusters() {
        let points = vec![
            ("a".to_string(), Point3D::new(0.0, 0.0, 0.0)),
            ("b".to_string(), Point3D::new(10.0, 10.0, 10.0)),
        ];
        let result = kmeans(&points, 2, 100);
        assert_eq!(result.clusters.len(), 2);
        // Points should be in different clusters
        assert_ne!(result.assignments[0], result.assignments[1]);
    }

    #[test]
    fn empty_input() {
        let points: Vec<(String, Point3D)> = vec![];
        let result = auto_cluster(&points);
        assert!(result.clusters.is_empty());
    }

    #[test]
    fn silhouette_in_range() {
        let points = vec![
            ("a".to_string(), Point3D::new(0.0, 0.0, 0.0)),
            ("b".to_string(), Point3D::new(0.1, 0.0, 0.0)),
            ("c".to_string(), Point3D::new(10.0, 10.0, 10.0)),
            ("d".to_string(), Point3D::new(10.1, 10.0, 10.0)),
        ];
        let result = kmeans(&points, 2, 100);
        assert!(
            result.silhouette >= -1.0 && result.silhouette <= 1.0,
            "Silhouette must be in [-1, 1], got {}",
            result.silhouette
        );
    }

    #[test]
    fn well_separated_high_silhouette() {
        let points = vec![
            ("a".to_string(), Point3D::new(0.0, 0.0, 0.0)),
            ("b".to_string(), Point3D::new(0.01, 0.01, 0.01)),
            ("c".to_string(), Point3D::new(100.0, 100.0, 100.0)),
            ("d".to_string(), Point3D::new(100.01, 100.01, 100.01)),
        ];
        let result = kmeans(&points, 2, 100);
        assert!(
            result.silhouette > 0.5,
            "Well-separated clusters should have high silhouette, got {}",
            result.silhouette
        );
    }
}
