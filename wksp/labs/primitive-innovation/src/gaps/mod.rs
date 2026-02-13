// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # P6-P8: Combination Space Fill
//!
//! **Problem**: ~27 of 105 possible T1 pairs are unexplored.
//! Some represent conceptually impossible combinations, but others
//! are genuinely novel compositions waiting to be discovered.
//!
//! ## Gap-Fill Types
//!
//! | Type | Missing Pair | Composition | Purpose |
//! |------|-------------|-------------|---------|
//! | `AbsenceRateDetector` | Void x Frequency | void + nu + kappa + N | Detect periodic missing data |
//! | `Tombstone` | Void x Persistence | void + pi + exists + irrev | Persistent deletion markers |
//! | `DampedOscillator` | Frequency x Recursion | nu + rho + N + partial | Recursive frequency convergence |

mod absence_rate;
mod damped_oscillator;
mod tombstone;

pub use absence_rate::AbsenceRateDetector;
pub use damped_oscillator::DampedOscillator;
pub use tombstone::Tombstone;
