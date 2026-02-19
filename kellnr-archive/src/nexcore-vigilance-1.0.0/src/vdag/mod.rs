//! # Validated SMART-DAG (VDAG)
//!
//! Meta-methodology integrating SMART goal validation, DAG-based execution,
//! CTVP validation, and CEP/SECI learning loops.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use nexcore_vigilance::vdag::prelude::*;
//!
//! fn example() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create a validated goal
//!     let goal = SmartGoal::builder()
//!         .raw("Build authentication system")
//!         .specific("Implement OAuth2 + JWT authentication")
//!         .measurable("100% of endpoints protected")
//!         .achievable("Team has auth experience")
//!         .relevant("Security requirement for launch")
//!         .time_bound("2 weeks")
//!         .build()?;
//!
//!     // Generate execution DAG
//!     let dag = Pipeline::from_goal(goal)?;
//!
//!     // Execute with validation
//!     let result = dag.execute()?;
//!
//!     // Check reality gradient
//!     println!("Reality Score: {:.2} ({})",
//!         result.reality_gradient.score,
//!         result.reality_gradient.interpretation()
//!     );
//!     Ok(())
//! }
//! ```
//!
//! ## Pipeline Phases
//!
//! | Phase | Description | Frameworks |
//! |-------|-------------|------------|
//! | 1 | Goal Validation | SMART + CTVP Phase 0 |
//! | 2 | DAG Generation | Decomposition + DAG |
//! | 3 | Execution | CTVP Phases 1-3 |
//! | 4 | Reflection | Learning Loops |
//! | 5 | Socialization | SECI Closure |
//!
//! ## Core Principle
//!
//! > "Mock testing is testing theater. But so is mock goal-setting—
//! > SMART without validation just validates your optimism."

#![warn(missing_docs)]

pub mod learning;
pub mod node;
pub mod pipeline;
pub mod reality;
pub mod smart;

/// Prelude for convenient imports
pub mod prelude {
    pub use super::learning::{LearningLoop, LoopResult, LoopType, Pattern};
    pub use super::node::{Node, NodeBuilder, NodeId, NodeStatus};
    pub use super::pipeline::{ExecutionResult, Pipeline, PipelinePhase, PipelineState};
    pub use super::reality::{Interpretation, RealityGradient};
    pub use super::smart::{SmartDimension, SmartGoal, SmartValidation, Variable};
}

/// VDAG Framework version
pub const VERSION: &str = env!("CARGO_PKG_VERSION");

/// Default Reality Gradient threshold for blocking
pub const BLOCKING_THRESHOLD: f64 = 0.20;

/// Default Reality Gradient threshold for warning
pub const WARNING_THRESHOLD: f64 = 0.50;

/// Triple-loop trigger (every N executions)
pub const TRIPLE_LOOP_INTERVAL: u32 = 5;

/// Failure rate threshold for double-loop escalation
pub const DOUBLE_LOOP_FAILURE_RATE: f64 = 0.20;
