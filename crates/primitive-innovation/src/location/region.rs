// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # RegionPartitioner
//!
//! **Tier**: T2-C (lambda + partial + N + Sigma)
//! **Dominant**: lambda (Location)
//!
//! Partitions a coordinate space into named regions with aggregate statistics.

use core::fmt;
use std::collections::BTreeMap;

use super::spatial_index::Coord;

/// A unique region identifier.
pub type RegionId = u32;

/// A rectangular region in 2D space.
///
/// ## Tier: T2-C (lambda + partial + N)
#[derive(Debug, Clone)]
pub struct Region {
    /// Unique identifier.
    pub id: RegionId,
    /// Human-readable name.
    pub name: String,
    /// Lower-left corner.
    pub min: Coord,
    /// Upper-right corner.
    pub max: Coord,
}

impl Region {
    /// Check if a point falls within this region.
    #[must_use]
    pub fn contains(&self, point: &Coord) -> bool {
        point.x >= self.min.x
            && point.x <= self.max.x
            && point.y >= self.min.y
            && point.y <= self.max.y
    }

    /// Area of the region.
    #[must_use]
    pub fn area(&self) -> f64 {
        (self.max.x - self.min.x) * (self.max.y - self.min.y)
    }

    /// Center point of the region.
    #[must_use]
    pub fn center(&self) -> Coord {
        Coord::new(
            (self.min.x + self.max.x) / 2.0,
            (self.min.y + self.max.y) / 2.0,
        )
    }
}

/// Aggregate statistics for a region.
#[derive(Debug, Clone, Default)]
pub struct RegionStats {
    /// Number of data points in the region.
    pub count: usize,
    /// Density (count / area).
    pub density: f64,
    /// Sum of values.
    pub value_sum: f64,
    /// Mean value.
    pub value_mean: f64,
}

/// Partitions space into named regions with aggregate operations.
///
/// ## Tier: T2-C (lambda + partial + N + Sigma)
/// Dominant: lambda (Location)
#[derive(Debug, Clone)]
pub struct RegionPartitioner {
    /// Regions by ID.
    regions: BTreeMap<RegionId, Region>,
    /// Data points per region.
    data: BTreeMap<RegionId, Vec<(Coord, f64)>>,
    /// Next region ID.
    next_id: RegionId,
}

impl RegionPartitioner {
    /// Create a new partitioner.
    #[must_use]
    pub fn new() -> Self {
        Self {
            regions: BTreeMap::new(),
            data: BTreeMap::new(),
            next_id: 0,
        }
    }

    /// Define a region.
    pub fn add_region(&mut self, name: impl Into<String>, min: Coord, max: Coord) -> RegionId {
        let id = self.next_id;
        self.next_id = self.next_id.saturating_add(1);

        self.regions.insert(
            id,
            Region {
                id,
                name: name.into(),
                min,
                max,
            },
        );
        self.data.entry(id).or_default();

        id
    }

    /// Insert a data point — auto-assigned to matching region(s).
    /// Returns the region IDs the point was assigned to.
    pub fn insert(&mut self, coord: Coord, value: f64) -> Vec<RegionId> {
        let mut assigned = Vec::new();

        for (id, region) in &self.regions {
            if region.contains(&coord) {
                assigned.push(*id);
            }
        }

        for id in &assigned {
            self.data.entry(*id).or_default().push((coord, value));
        }

        assigned
    }

    /// Get statistics for a region.
    #[must_use]
    pub fn stats(&self, region_id: RegionId) -> Option<RegionStats> {
        let region = self.regions.get(&region_id)?;
        let points = self.data.get(&region_id)?;

        let count = points.len();
        let area = region.area();
        #[allow(
            clippy::as_conversions,
            reason = "usize to f64 for density calculation; count fits in f64"
        )]
        let density = if area > 0.0 { count as f64 / area } else { 0.0 };
        let value_sum: f64 = points.iter().map(|(_, v)| v).sum();
        #[allow(
            clippy::as_conversions,
            reason = "usize to f64 for mean calculation; count fits in f64"
        )]
        let value_mean = if count > 0 {
            value_sum / count as f64
        } else {
            0.0
        };

        Some(RegionStats {
            count,
            density,
            value_sum,
            value_mean,
        })
    }

    /// Get region by ID.
    #[must_use]
    pub fn region(&self, id: RegionId) -> Option<&Region> {
        self.regions.get(&id)
    }

    /// Find which region a point belongs to.
    #[must_use]
    pub fn locate(&self, point: &Coord) -> Vec<RegionId> {
        self.regions
            .iter()
            .filter(|(_, r)| r.contains(point))
            .map(|(id, _)| *id)
            .collect()
    }

    /// Total number of regions.
    #[must_use]
    pub fn region_count(&self) -> usize {
        self.regions.len()
    }

    /// Rank regions by data density (descending).
    #[must_use]
    pub fn rank_by_density(&self) -> Vec<(RegionId, f64)> {
        let mut ranked: Vec<(RegionId, f64)> = self
            .regions
            .keys()
            .filter_map(|id| self.stats(*id).map(|s| (*id, s.density)))
            .collect();

        ranked.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap_or(core::cmp::Ordering::Equal));
        ranked
    }
}

impl Default for RegionPartitioner {
    fn default() -> Self {
        Self::new()
    }
}

impl fmt::Display for RegionPartitioner {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "RegionPartitioner({} regions)", self.region_count())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_region_insert_and_stats() {
        let mut part = RegionPartitioner::new();
        let r1 = part.add_region("north", Coord::new(0.0, 50.0), Coord::new(100.0, 100.0));
        let r2 = part.add_region("south", Coord::new(0.0, 0.0), Coord::new(100.0, 50.0));

        part.insert(Coord::new(10.0, 80.0), 5.0); // north
        part.insert(Coord::new(20.0, 90.0), 3.0); // north
        part.insert(Coord::new(50.0, 10.0), 8.0); // south

        let north_stats = part.stats(r1);
        assert!(north_stats.is_some());
        let ns = north_stats.unwrap_or_default();
        assert_eq!(ns.count, 2);
        assert!((ns.value_mean - 4.0).abs() < 1e-10);

        let south_stats = part.stats(r2);
        assert!(south_stats.is_some());
        assert_eq!(south_stats.unwrap_or_default().count, 1);
    }

    #[test]
    fn test_locate_point() {
        let mut part = RegionPartitioner::new();
        part.add_region("box", Coord::new(0.0, 0.0), Coord::new(10.0, 10.0));

        let inside = part.locate(&Coord::new(5.0, 5.0));
        assert_eq!(inside.len(), 1);

        let outside = part.locate(&Coord::new(50.0, 50.0));
        assert!(outside.is_empty());
    }

    #[test]
    fn test_rank_by_density() {
        let mut part = RegionPartitioner::new();
        let small = part.add_region("small", Coord::new(0.0, 0.0), Coord::new(1.0, 1.0));
        let big = part.add_region("big", Coord::new(0.0, 0.0), Coord::new(100.0, 100.0));

        // Same point count but different areas
        part.insert(Coord::new(0.5, 0.5), 1.0);

        let ranked = part.rank_by_density();
        assert_eq!(ranked.len(), 2);
        // small region should rank higher (same count, smaller area = higher density)
        assert_eq!(ranked[0].0, small);
        let _ = big; // used only for ranking comparison
    }
}
