//! # Autonomous Documentation Generator
//!
//! FORGE-generated module for auto-creating CLAUDE.md files
//! by mining codebase primitives.
//!
//! ## Primitive Foundation
//!
//! | T1 Primitive | Application |
//! |--------------|-------------|
//! | sequence | Extraction pipeline |
//! | mapping | Source → Section transformation |
//! | state | Generator configuration |
//! | inference | Purpose derivation from patterns |

pub mod claude_md;

pub use claude_md::{ClaudeMdGenerator, GeneratorConfig, Section};
