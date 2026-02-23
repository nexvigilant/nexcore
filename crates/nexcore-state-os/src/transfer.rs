//! # Cross-Domain Transfer
//!
//! Maps between this crate's types and other domains.
//!
//! ## Established Transfer Confidence
//!
//! | Source Domain              | Target Domain            | Confidence | Bridge Primitive |
//! |----------------------------|--------------------------|------------|-----------------|
//! | OS kernel (15 syscall layers) | STOS 15-layer runtime | 0.89       | ς + → (State + Causality) |
//! | UTS (Universal Theory of State) | `nexcore-state-theory` | 0.95 | All 15 T1 primitives |
//! | Temporal scheduling (cron) | STOS-TM layer           | 0.82       | ν (Frequency) |
//! | Distributed systems (Raft) | STOS-LC location router | 0.78       | λ + π (Location + Persistence) |
//!
// Transfer mappings will be added as cross-domain bridges are identified.
