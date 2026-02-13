// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Primitive Innovation Lab
//!
//! Exploration crate for nexcore primitive ecosystem innovations.
//! Discovers new T2-C types, cross-domain bridges, and fills gaps
//! in the 105-pair T1 combination space.
//!
//! ## Organization
//!
//! | Module | Priority | Innovation |
//! |--------|----------|-----------|
//! | `location` | P1 | 5 lambda-dominant T2-C types (raises 2.6% -> ~5%) |
//! | `frequency` | P2 | 3 nu-dominant T2-C types (raises 3.4% -> ~4.5%) |
//! | `bridges` | P3-P5 | 3 cross-domain bridges (0 -> 3 built) |
//! | `gaps` | P6-P8 | 3 novel pair compositions (fills Void x Freq, Void x Persist, Freq x Recur) |
//! | `exploratory` | P10-P12 | 3 advanced bridges (quantum×stos, aggregate×cloud, transcriptase×dtree) |
//!
//! ## Primitive Pair Coverage
//!
//! Before: 78/105 pairs (74%)
//! After: 81/105 pairs (77%) — fills 3 previously unexplored combinations
//!
//! ## New Types Summary (17 total)
//!
//! | Type | Module | Tier | Dominant | Novel Pair |
//! |------|--------|------|----------|------------|
//! | `SpatialIndex<V>` | location | T2-C | lambda | — |
//! | `TopologyGraph` | location | T2-C | lambda | — |
//! | `PathResolver<V>` | location | T2-C | lambda | — |
//! | `RegionPartitioner` | location | T2-C | lambda | — |
//! | `ProximityEngine<V>` | location | T2-C | lambda | — |
//! | `AdaptivePoller` | frequency | T2-C | nu | — |
//! | `RetryStrategy` | frequency | T2-C | nu | — |
//! | `PeriodicMonitor` | frequency | T2-C | nu | — |
//! | `NeuroendocrineCoordinator` | bridges | T3 | causality | — |
//! | `EnergeticExecutor` | bridges | T2-C | causality | — |
//! | `SchemaImmuneSystem` | bridges | T3 | kappa | — |
//! | `AbsenceRateDetector` | gaps | T2-C | void | Void x Frequency |
//! | `Tombstone` | gaps | T2-C | void | Void x Persistence |
//! | `DampedOscillator` | gaps | T2-C | nu | Frequency x Recursion |
//! | `QuantumStateSpace` | exploratory | T3 | state | quantum × stos |
//! | `CloudResourceGraph` | exploratory | T2-C | sum | aggregate × cloud |
//! | `SchemaGuidedSplitter` | exploratory | T2-C | comparison | transcriptase × dtree |

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![allow(dead_code)]

/// P1: Location (lambda) primitive expansion — 5 new T2-C types.
pub mod location;

/// P2: Frequency (nu) primitive expansion — 3 new T2-C types.
pub mod frequency;

/// P3-P5: Cross-domain bridges — 3 bridge types.
pub mod bridges;

/// P6-P8: Combination space gap fill — 3 novel pair types.
pub mod gaps;

/// P10-P12: Exploratory advanced bridges — 3 high-complexity types.
pub mod exploratory;
