//! Rendering pipeline: viewport → rasterizer → fragment → framebuffer

pub mod fragment;
pub mod framebuffer;
pub mod rasterizer;
pub mod viewport;

pub use framebuffer::Framebuffer;
pub use rasterizer::*;
pub use viewport::Viewport;
