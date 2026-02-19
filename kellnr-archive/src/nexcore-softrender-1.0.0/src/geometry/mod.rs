//! Geometry: vertices, meshes, shape generators

pub mod mesh;
pub mod shapes;
pub mod vertex;

pub use mesh::{Mesh, Triangle};
pub use shapes::*;
pub use vertex::Vertex;
