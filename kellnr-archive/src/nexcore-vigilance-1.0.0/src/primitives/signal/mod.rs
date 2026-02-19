//! # Signal Primitives
//!
//! Universal signal detection decomposed to T1 Lex Primitiva.
//!
//! ## T1 Primitive Grounding
//!
//! | T1 Symbol | Name | Role in Signal | Rust Manifestation |
//! |:---------:|:-----|:---------------|:-------------------|
//! | **N** | Quantity | Observed/expected counts | `u64`, `Count` |
//! | **f** | Frequency | Rate of occurrence | `Frequency` |
//! | **kappa** | Comparison | O/E ratio | `Ratio`, `impl PartialOrd` |
//! | **partial** | Boundary | Threshold | `Threshold` |
//! | **exists** | Existence | Signal detected | `bool`, `Detected` |
//! | **sigma** | Sequence | Time ordering | `Timestamp` |
//! | **varsigma** | State | Lifecycle state | `State` |
//! | **mu** | Mapping | Transformation | `impl From<T>` |
//! | **emptyset** | Void | No signal | `Option<T>`, `None` |
//! | **arrow** | Causality | Association | `Association` |
//! | **pi** | Persistence | Storage | `impl Persist` |
//! | **Sigma** | Sum | Method selection | `enum Method` |
//! | **propto** | Irreversibility | State transitions | Typestate |
//! | **lambda** | Location | Source identity | `Source` |
//!
//! ## Tier Classification
//!
//! - **T1**: `Count`, `Timestamp` (raw primitives)
//! - **T2-P**: `Frequency`, `Ratio`, `Threshold`, `Source`, `Association`, `Method` (simple wrappers)
//! - **T2-C**: `Table`, `Interval`, `Signal`, `SignalLifecycle` (composites)
//! - **T3**: Domain-specific implementations
//!
//! ## Canonical Signal Grounding (∃ dominant @ 0.85)
//!
//! Signal detection fundamentally answers: "Does this association **exist**?"
//!
//! | T1 Symbol | Role | Rust Type |
//! |:---------:|:-----|:----------|
//! | **∃** | Existence determination (dominant) | `Detected` |
//! | **κ** | Observed vs expected comparison | `Ratio` |
//! | **N** | Numeric metric values | `Count`, `f64` |
//! | **∂** | Confidence interval bounds | `Interval`, `Threshold` |
//! | **Σ** | Method selection coproduct | `Method` |
//!
//! ## Cross-Domain Transfer
//!
//! | Domain | Instantiation |
//! |--------|---------------|
//! | Pharmacovigilance | PRR = drug-event disproportionality |
//! | Finance | Signal = price anomaly detection |
//! | Cybersecurity | Signal = intrusion pattern |
//! | Epidemiology | Signal = outbreak indicator |
//!
//! ## Example
//!
//! ```rust
//! use nexcore_vigilance::primitives::signal::{
//!     Count, Frequency, Ratio, Threshold, Detected,
//!     Table, compute_ratio, exceeds_threshold,
//! };
//!
//! // T1: Raw counts
//! let observed = Count::new(15);
//! let total_exposed = Count::new(115);
//! let background = Count::new(20);
//! let total_background = Count::new(10020);
//!
//! // T2-P: Frequencies (Option due to division-by-zero guard)
//! let freq_exposed = Frequency::from_count(observed, total_exposed).unwrap();
//! let freq_background = Frequency::from_count(background, total_background).unwrap();
//!
//! // T2-P: Ratio (comparison primitive, guarded)
//! let ratio = compute_ratio(freq_exposed, freq_background).unwrap();
//!
//! // T2-P: Threshold (boundary primitive)
//! let threshold = Threshold::new(2.0);
//!
//! // T1: Existence (signal detected?)
//! let detected = exceeds_threshold(ratio, threshold);
//! assert!(detected.is_signal());
//! ```

mod analytics;
mod atoms;
mod composites;
mod epidemiology;
mod quality;

pub use analytics::*;
pub use atoms::*;
pub use composites::*;
pub use epidemiology::*;
pub use quality::*;

#[cfg(test)]
mod tests;
