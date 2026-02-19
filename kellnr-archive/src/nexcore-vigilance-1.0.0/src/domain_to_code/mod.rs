//! # Domain-to-Code Pipeline
//!
//! Transforms domain knowledge into idiomatic Rust code through a typestate pipeline.
//!
//! ## Architecture
//!
//! ```text
//! Domain Text → [Extract] → Patterns → [Classify] → Languages → [Generate] → Rust Code
//! ```
//!
//! ## T1 Primitive Foundation
//!
//! Built on five irreducible primitives:
//! - **Sequence**: Pipeline stages flow in order
//! - **Mapping**: `From`/`Into` for pattern→code transformations
//! - **Recursion**: `DomainPattern` enum with `Box<Self>` for AST
//! - **State**: `Pipeline<S>` typestate pattern
//! - **Void**: `PhantomData<Stage>`, `Option<Classification>`
//!
//! ## Seven Universal Languages
//!
//! All domain patterns are classified into one or more of seven universal languages:
//!
//! | Language | Core Concept | Rust Mapping |
//! |----------|--------------|--------------|
//! | Risk | Probability, uncertainty | `f64`, `Result<T, E>` |
//! | Optimization | Objective, constraint | Iterators, `min`/`max` |
//! | Network | Node, edge, flow | Graph types |
//! | Information | Signal, entropy | `Vec<u8>`, channels |
//! | Resource | Capacity, allocation | Pool, Arc, limits |
//! | Emergence | Hierarchy, feedback | Nested enums |
//! | Adaptation | Learning, evolution | State machines |

pub mod extractor;
pub mod generator;
pub mod languages;
pub mod patterns;
pub mod pipeline;
pub mod primitives;

// Re-exports for convenience
pub use extractor::{ExtractedPattern, ExtractionContext, ExtractionError, PatternExtractor};
pub use generator::{
    CodeEmitter, GeneratedCode, GeneratorConfig, GeneratorError, RustCodeGenerator,
};
pub use languages::{DomainLanguage, LanguageClassification, LanguageClassifier};
pub use patterns::{
    CombinationSemantics, DomainPattern, PatternAst, PatternSemantics, TransformType,
};
pub use pipeline::{Classified, Extracted, Generated, Pipeline, PipelineError, Raw, Validated};
pub use primitives::{T1Mapping, T1Recursion, T1Sequence, T1State, T1Void};
