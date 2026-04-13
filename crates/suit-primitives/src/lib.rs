//! # Suit Primitives
//! Shared data structures with minimal dependencies.
#[derive(Debug, Clone, Copy, serde::Serialize, serde::Deserialize)]
pub struct Vector3 {
    pub x: f32,
    pub y: f32,
    pub z: f32,
}
