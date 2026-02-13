// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # ProximityEngine
//!
//! **Tier**: T2-C (lambda + kappa + N + mu)
//! **Dominant**: lambda (Location)
//!
//! Distance-based query engine supporting multiple distance metrics.
//! Maps entities to locations and answers "what's near X?" queries.

use core::fmt;
use std::collections::BTreeMap;

use super::spatial_index::Coord;

/// Distance metric for proximity calculations.
///
/// ## Tier: T2-P (kappa + N)
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum DistanceMetric {
    /// Straight-line distance (sqrt of sum of squared diffs).
    Euclidean,
    /// Sum of absolute differences per axis.
    Manhattan,
    /// Maximum absolute difference across axes.
    Chebyshev,
}

/// A proximity query result.
#[derive(Debug, Clone)]
pub struct ProximityResult<V> {
    /// The entity value.
    pub value: V,
    /// Its location.
    pub coord: Coord,
    /// Distance from query point.
    pub distance: f64,
}

/// Unique entity identifier.
pub type EntityId = u32;

/// Distance-based entity lookup engine.
///
/// ## Tier: T2-C (lambda + kappa + N + mu)
/// Dominant: lambda (Location)
#[derive(Debug, Clone)]
pub struct ProximityEngine<V> {
    /// Entities by ID.
    entities: BTreeMap<EntityId, (Coord, V)>,
    /// Default distance metric.
    metric: DistanceMetric,
    /// Next entity ID.
    next_id: EntityId,
}

impl<V: Clone + fmt::Debug> ProximityEngine<V> {
    /// Create a new proximity engine with the given metric.
    #[must_use]
    pub fn new(metric: DistanceMetric) -> Self {
        Self {
            entities: BTreeMap::new(),
            metric,
            next_id: 0,
        }
    }

    /// Register an entity at a location.
    pub fn register(&mut self, coord: Coord, value: V) -> EntityId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);
        self.entities.insert(id, (coord, value));
        id
    }

    /// Find the K nearest entities to a point.
    #[must_use]
    pub fn k_nearest(&self, target: &Coord, k: usize) -> Vec<ProximityResult<V>> {
        let mut distances: Vec<ProximityResult<V>> = self
            .entities
            .values()
            .map(|(coord, value)| ProximityResult {
                value: value.clone(),
                coord: *coord,
                distance: self.compute_distance(target, coord),
            })
            .collect();

        distances.sort_by(|a, b| {
            a.distance
                .partial_cmp(&b.distance)
                .unwrap_or(core::cmp::Ordering::Equal)
        });

        distances.truncate(k);
        distances
    }

    /// Find all entities within a given distance.
    #[must_use]
    pub fn within_distance(&self, target: &Coord, max_distance: f64) -> Vec<ProximityResult<V>> {
        self.entities
            .values()
            .filter_map(|(coord, value)| {
                let dist = self.compute_distance(target, coord);
                if dist <= max_distance {
                    Some(ProximityResult {
                        value: value.clone(),
                        coord: *coord,
                        distance: dist,
                    })
                } else {
                    None
                }
            })
            .collect()
    }

    /// Update an entity's location.
    pub fn relocate(&mut self, id: EntityId, new_coord: Coord) -> bool {
        if let Some(entry) = self.entities.get_mut(&id) {
            entry.0 = new_coord;
            true
        } else {
            false
        }
    }

    /// Remove an entity.
    pub fn remove(&mut self, id: EntityId) -> bool {
        self.entities.remove(&id).is_some()
    }

    /// Total entities.
    #[must_use]
    pub fn len(&self) -> usize {
        self.entities.len()
    }

    /// Whether empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.entities.is_empty()
    }

    /// Compute distance using the engine's metric.
    fn compute_distance(&self, a: &Coord, b: &Coord) -> f64 {
        match self.metric {
            DistanceMetric::Euclidean => a.distance_to(b),
            DistanceMetric::Manhattan => a.manhattan_distance(b),
            DistanceMetric::Chebyshev => {
                let dx = (a.x - b.x).abs();
                let dy = (a.y - b.y).abs();
                dx.max(dy)
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_k_nearest() {
        let mut engine = ProximityEngine::new(DistanceMetric::Euclidean);
        engine.register(Coord::new(0.0, 0.0), "origin");
        engine.register(Coord::new(1.0, 0.0), "close");
        engine.register(Coord::new(100.0, 100.0), "far");

        // Query from (0.1, 0.0): origin=0.1, close=0.9, far=~141
        let nearest = engine.k_nearest(&Coord::new(0.1, 0.0), 2);
        assert_eq!(nearest.len(), 2);
        assert_eq!(nearest[0].value, "origin");
        assert_eq!(nearest[1].value, "close");
    }

    #[test]
    fn test_within_distance() {
        let mut engine = ProximityEngine::new(DistanceMetric::Manhattan);
        engine.register(Coord::new(0.0, 0.0), "A");
        engine.register(Coord::new(2.0, 2.0), "B"); // Manhattan dist = 4
        engine.register(Coord::new(10.0, 10.0), "C"); // Manhattan dist = 20

        let near = engine.within_distance(&Coord::new(0.0, 0.0), 5.0);
        assert_eq!(near.len(), 2); // A and B
    }

    #[test]
    fn test_chebyshev() {
        let mut engine = ProximityEngine::new(DistanceMetric::Chebyshev);
        engine.register(Coord::new(3.0, 7.0), "point");

        let results = engine.k_nearest(&Coord::new(0.0, 0.0), 1);
        assert_eq!(results.len(), 1);
        // Chebyshev distance = max(|3-0|, |7-0|) = 7
        assert!((results[0].distance - 7.0).abs() < 1e-10);
    }

    #[test]
    fn test_relocate() {
        let mut engine = ProximityEngine::new(DistanceMetric::Euclidean);
        let id = engine.register(Coord::new(0.0, 0.0), "mobile");

        assert!(engine.relocate(id, Coord::new(50.0, 50.0)));

        let results = engine.k_nearest(&Coord::new(50.0, 50.0), 1);
        assert!((results[0].distance - 0.0).abs() < 1e-10);
    }
}
