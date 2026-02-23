// Copyright (c) 2026 Matthew Campion, PharmD; NexVigilant
// All Rights Reserved. See LICENSE file for details.

//! # Primitive Foundation
//!
//! Re-exports the T1 Lex Primitiva primitives most relevant to `nexcore-os`.
//!
//! ## Dominant Primitives for the OS Layer
//!
//! | Component     | Dominant Primitive | Rationale |
//! |---------------|--------------------|-----------|
//! | Boot sequence | σ (Sequence)       | Ordered causal phase transitions |
//! | Service mgr   | ς (State)          | Service lifecycle state machine |
//! | Security      | ∂ (Boundary)       | Threat detection perimeters |
//! | Vault         | π (Persistence)    | Encrypted secret persistence |
//! | IPC/EventBus  | → (Causality)      | Event emission causes handler activation |
//! | Users         | ∂ + κ              | Authentication boundary + comparison |
//! | Kernel (NexCoreOs) | Σ (Sum)       | Composition of all OS subsystems |

pub use nexcore_lex_primitiva::grounding::GroundsTo;
pub use nexcore_lex_primitiva::primitiva::LexPrimitiva;
pub use nexcore_lex_primitiva::primitiva::PrimitiveComposition;
pub use nexcore_lex_primitiva::tier::Tier;
