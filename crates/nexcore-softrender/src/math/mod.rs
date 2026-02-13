//! Math primitives: vectors, matrices, transforms, color
//!
//! Zero dependencies. Pure computation.

pub mod color;
pub mod mat;
pub mod transform;
pub mod vec;

// Re-export core types
pub use color::Color;
pub use mat::{Mat3, Mat4};
pub use transform::*;
pub use vec::{Vec2, Vec3, Vec4};
