//! Server functions for Education Machine
//!
//! Each function calls localhost:3030 with reqwest, falling back to mock data.

pub mod get_learner;
pub mod get_subjects;
pub mod get_lessons;
pub mod submit_assessment;
pub mod schedule_review;

pub use get_learner::*;
pub use get_subjects::*;
pub use get_lessons::*;
pub use submit_assessment::*;
pub use schedule_review::*;
