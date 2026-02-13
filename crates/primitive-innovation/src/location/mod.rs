// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # P1: Location (lambda) Expansion
//!
//! **Problem**: lambda is the most underrepresented dominant primitive (2.6%, 9 types).
//! Also the most isolated secondary primitive (42 occurrences vs 145 for Boundary).
//!
//! **Goal**: Create 4-5 T2-C types with Location dominant to raise coverage to ~5%.
//!
//! ## New Types
//!
//! | Type | Tier | Composition | Purpose |
//! |------|------|-------------|---------|
//! | `SpatialIndex<K,V>` | T2-C | lambda + mu + kappa + N | Geographic/spatial data queries |
//! | `TopologyGraph` | T2-C | lambda + rho + sigma + partial | Network topology with routing |
//! | `PathResolver` | T2-C | lambda + sigma + exists | Hierarchical path resolution |
//! | `RegionPartitioner` | T2-C | lambda + partial + N + Sigma | Geographic data partitioning |
//! | `ProximityEngine` | T2-C | lambda + kappa + N + mu | Distance-based queries |

mod path_resolver;
mod proximity;
mod region;
mod spatial_index;
mod topology;

pub use path_resolver::PathResolver;
pub use proximity::ProximityEngine;
pub use region::RegionPartitioner;
pub use spatial_index::SpatialIndex;
pub use topology::TopologyGraph;
