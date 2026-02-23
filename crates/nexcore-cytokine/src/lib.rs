//! # NexVigilant Core — Cytokine - Typed Event Signaling
//!
//! Fire-and-forget signaling system based on immune system cytokine patterns.
//!
//! ## T1 Primitive Grounding
//!
//! | Concept | Primitive | Symbol | Role |
//! |---------|-----------|--------|------|
//! | Signal emission | Causality | → | Emitter causes receptor activation |
//! | Receptor binding | Mapping | μ | Typed matching of signal to handler |
//! | Amplification | Recursion | ρ | Cascade chains (one signal triggers many) |
//! | Concentration | Quantity | N | Signal strength/severity |
//! | Scope | Location | λ | Paracrine (local) vs Endocrine (systemic) |
//! | Duration | Persistence | π | Transient vs sustained signaling |
//! | State change | State | ς | Receptor state after binding |
//!
//! ## Cytokine Families
//!
//! | Family | Biological Function | Claude Code Analog |
//! |--------|--------------------|--------------------|
//! | IL (Interleukin) | Cell communication | Inter-component messaging |
//! | TNF (Tumor Necrosis) | Destroy threats | Block/terminate actions |
//! | IFN (Interferon) | Activate defenses | Amplify responses |
//! | TGF (Transforming) | Regulate growth | Modulate behavior |
//! | CSF (Colony Stimulating) | Spawn cells | Create subagents |
//!
//! ## Usage
//!
//! ```rust,ignore
//! use nexcore_cytokine::{Cytokine, CytokineFamily, Emitter, Receptor};
//!
//! // Emit a signal (fire-and-forget)
//! let signal = Cytokine::new(CytokineFamily::Il1, "threat_detected")
//!     .with_severity(Severity::High)
//!     .with_payload(json!({"file": "lib.rs", "line": 42}));
//!
//! emitter.emit(signal).await;
//!
//! // Receive signals (async handler)
//! receptor.on(CytokineFamily::Il1, |signal| async {
//!     println!("Received: {:?}", signal);
//! });
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]
#![warn(missing_docs)]

pub mod composites;
pub mod grounding;
pub mod neurotransmitter;
pub mod primitives;
pub mod pv_bridge;
pub mod transfer;

mod apoptosis;
mod bus;
mod cascade;
mod chemotaxis;
mod differentiation;
mod emitter;
mod endocytosis;
mod exocytosis;
mod mitosis;
mod nmd;
mod phagocytosis;
mod quorum;
mod receptor;
mod types;

pub use apoptosis::*;
pub use bus::*;
pub use cascade::*;
pub use chemotaxis::*;
pub use differentiation::*;
pub use emitter::*;
pub use endocytosis::*;
pub use exocytosis::*;
pub use mitosis::*;
pub use phagocytosis::*;
pub use quorum::*;
pub use receptor::*;
pub use types::*;

/// Prelude for common imports
pub mod prelude;
