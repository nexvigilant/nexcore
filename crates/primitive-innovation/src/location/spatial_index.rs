// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # SpatialIndex<K, V>
//!
//! **Tier**: T2-C (lambda + mu + kappa + N)
//! **Dominant**: lambda (Location)
//!
//! A spatial data structure that indexes values by 2D coordinates,
//! supporting range queries and nearest-neighbor lookups.

use core::fmt;
use std::collections::BTreeMap;

/// A 2D coordinate in the spatial index.
///
/// ## Tier: T2-P (lambda + N)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Coord {
    /// X position.
    pub x: f64,
    /// Y position.
    pub y: f64,
}

impl Coord {
    /// Create a new coordinate.
    #[must_use]
    pub fn new(x: f64, y: f64) -> Self {
        Self { x, y }
    }

    /// Euclidean distance to another coordinate.
    #[must_use]
    pub fn distance_to(&self, other: &Self) -> f64 {
        ((self.x - other.x).powi(2) + (self.y - other.y).powi(2)).sqrt()
    }

    /// Manhattan distance to another coordinate.
    #[must_use]
    pub fn manhattan_distance(&self, other: &Self) -> f64 {
        (self.x - other.x).abs() + (self.y - other.y).abs()
    }
}

impl fmt::Display for Coord {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "({:.2}, {:.2})", self.x, self.y)
    }
}

/// An axis-aligned bounding box for range queries.
///
/// ## Tier: T2-C (lambda + partial + N)
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct BoundingBox {
    /// Minimum corner.
    pub min: Coord,
    /// Maximum corner.
    pub max: Coord,
}

impl BoundingBox {
    /// Create a bounding box from corners.
    #[must_use]
    pub fn new(min: Coord, max: Coord) -> Self {
        Self { min, max }
    }

    /// Check if a point is inside this box.
    #[must_use]
    pub fn contains(&self, point: &Coord) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Check if two boxes overlap.
    #[must_use]
    pub fn intersects(&self, other: &BoundingBox) -> bool {
        self.min.x <= other.max.x
            && self.max.x >= other.min.x
            && self.min.y <= other.max.y
            && self.max.y >= other.min.y
    }

    /// Area of the bounding box.
    #[must_use]
    pub fn area(&self) -> f64 {
        (self.max.x - self.min.x) * (self.max.y - self.min.y)
    }
}

/// An entry in the spatial index.
#[derive(Debug, Clone)]
struct SpatialEntry<V> {
    coord: Coord,
    value: V,
}

/// A spatial index mapping 2D coordinates to values.
///
/// Uses a grid-based approach for O(1) average lookups within cells.
///
/// ## Tier: T2-C (lambda + mu + kappa + N)
/// Dominant: lambda (Location)
#[derive(Debug, Clone)]
pub struct SpatialIndex<V> {
    /// Grid cell size.
    cell_size: f64,
    /// Grid cells indexed by (grid_x, grid_y).
    cells: BTreeMap<(i64, i64), Vec<SpatialEntry<V>>>,
    /// Total number of entries.
    count: usize,
}

impl<V: Clone + fmt::Debug> SpatialIndex<V> {
    /// Create a new spatial index with the given cell size.
    ///
    /// Smaller cell size = more precise queries but more memory.
    #[must_use]
    pub fn new(cell_size: f64) -> Self {
        let cell_size = if cell_size <= 0.0 { 1.0 } else { cell_size };
        Self {
            cell_size,
            cells: BTreeMap::new(),
            count: 0,
        }
    }

    /// Insert a value at a coordinate.
    pub fn insert(&mut self, coord: Coord, value: V) {
        let key = self.grid_key(&coord);
        self.cells
            .entry(key)
            .or_default()
            .push(SpatialEntry { coord, value });
        self.count = self.count.saturating_add(1);
    }

    /// Query all values within a bounding box.
    #[must_use]
    pub fn query_range(&self, bbox: &BoundingBox) -> Vec<(&Coord, &V)> {
        let min_key = self.grid_key(&bbox.min);
        let max_key = self.grid_key(&bbox.max);

        let mut results = Vec::new();

        for gx in min_key.0..=max_key.0 {
            for gy in min_key.1..=max_key.1 {
                if let Some(entries) = self.cells.get(&(gx, gy)) {
                    for entry in entries {
                        if bbox.contains(&entry.coord) {
                            results.push((&entry.coord, &entry.value));
                        }
                    }
                }
            }
        }

        results
    }

    /// Find the nearest value to a given coordinate.
    #[must_use]
    #[allow(
        clippy::arithmetic_side_effects,
        reason = "grid ring arithmetic: i64 range ops on bounded grid coordinates"
    )]
    #[allow(
        clippy::as_conversions,
        reason = "i64 to f64 for distance comparison; grid radius fits in f64"
    )]
    pub fn nearest(&self, target: &Coord) -> Option<(&Coord, &V)> {
        let mut best: Option<(&Coord, &V, f64)> = None;

        // Search expanding rings of grid cells
        for radius in 0..=self.search_radius() {
            let center = self.grid_key(target);

            for gx in (center.0 - radius)..=(center.0 + radius) {
                for gy in (center.1 - radius)..=(center.1 + radius) {
                    if let Some(entries) = self.cells.get(&(gx, gy)) {
                        for entry in entries {
                            let dist = target.distance_to(&entry.coord);
                            let is_closer = best.as_ref().is_none_or(|b| dist < b.2);
                            if is_closer {
                                best = Some((&entry.coord, &entry.value, dist));
                            }
                        }
                    }
                }
            }

            // If we found something and the next ring can't be closer, stop
            if let Some((_, _, best_dist)) = &best {
                let next_ring_min_dist = (radius as f64) * self.cell_size;
                if *best_dist <= next_ring_min_dist {
                    break;
                }
            }
        }

        best.map(|(coord, value, _)| (coord, value))
    }

    /// Find all values within a given radius of a point.
    #[must_use]
    pub fn within_radius(&self, center: &Coord, radius: f64) -> Vec<(&Coord, &V)> {
        let bbox = BoundingBox::new(
            Coord::new(center.x - radius, center.y - radius),
            Coord::new(center.x + radius, center.y + radius),
        );

        self.query_range(&bbox)
            .into_iter()
            .filter(|(coord, _)| center.distance_to(coord) <= radius)
            .collect()
    }

    /// Total entries in the index.
    #[must_use]
    pub fn len(&self) -> usize {
        self.count
    }

    /// Whether the index is empty.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.count == 0
    }

    /// Number of occupied grid cells.
    #[must_use]
    pub fn cell_count(&self) -> usize {
        self.cells.len()
    }

    /// Convert a coordinate to a grid cell key.
    #[allow(
        clippy::as_conversions,
        reason = "f64 floor to i64 for grid cell indexing; values are bounded by coordinate space"
    )]
    fn grid_key(&self, coord: &Coord) -> (i64, i64) {
        (
            (coord.x / self.cell_size).floor() as i64,
            (coord.y / self.cell_size).floor() as i64,
        )
    }

    /// Maximum search radius based on occupied cells.
    #[allow(
        clippy::as_conversions,
        reason = "usize to f64 for sqrt heuristic; result ceil back to i64 for ring count"
    )]
    fn search_radius(&self) -> i64 {
        // Heuristic: search up to sqrt(cell_count) rings
        let r = (self.cells.len() as f64).sqrt().ceil() as i64;
        r.max(3) // minimum 3 rings
    }
}

// ═══════════════════════════════════════════════════════════
// TESTS
// ═══════════════════════════════════════════════════════════

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_coord_distance() {
        let a = Coord::new(0.0, 0.0);
        let b = Coord::new(3.0, 4.0);
        let dist = a.distance_to(&b);
        assert!((dist - 5.0).abs() < 1e-10);
    }

    #[test]
    fn test_bounding_box_contains() {
        let bbox = BoundingBox::new(Coord::new(0.0, 0.0), Coord::new(10.0, 10.0));
        assert!(bbox.contains(&Coord::new(5.0, 5.0)));
        assert!(!bbox.contains(&Coord::new(11.0, 5.0)));
    }

    #[test]
    fn test_spatial_insert_and_query() {
        let mut index = SpatialIndex::new(10.0);
        index.insert(Coord::new(1.0, 1.0), "A");
        index.insert(Coord::new(5.0, 5.0), "B");
        index.insert(Coord::new(50.0, 50.0), "C");

        let bbox = BoundingBox::new(Coord::new(0.0, 0.0), Coord::new(10.0, 10.0));
        let results = index.query_range(&bbox);
        assert_eq!(results.len(), 2);
    }

    #[test]
    fn test_nearest() {
        let mut index = SpatialIndex::new(10.0);
        index.insert(Coord::new(1.0, 1.0), "close");
        index.insert(Coord::new(100.0, 100.0), "far");

        let target = Coord::new(2.0, 2.0);
        let nearest = index.nearest(&target);
        assert!(nearest.is_some());
        assert_eq!(*nearest.map(|(_, v)| v).unwrap_or(&""), "close");
    }

    #[test]
    fn test_within_radius() {
        let mut index = SpatialIndex::new(5.0);
        index.insert(Coord::new(0.0, 0.0), "origin");
        index.insert(Coord::new(3.0, 0.0), "near");
        index.insert(Coord::new(10.0, 0.0), "far");

        let results = index.within_radius(&Coord::new(0.0, 0.0), 5.0);
        assert_eq!(results.len(), 2); // origin + near
    }

    #[test]
    fn test_empty_index() {
        let index: SpatialIndex<&str> = SpatialIndex::new(1.0);
        assert!(index.is_empty());
        assert_eq!(index.len(), 0);
        assert!(index.nearest(&Coord::new(0.0, 0.0)).is_none());
    }
}
