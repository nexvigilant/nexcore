//! # GPU SDF Text Rendering
//!
//! Signed Distance Field (SDF) text rendering for massive label instancing.
//! Avoids creating individual geometry per character by using a texture atlas
//! of SDF glyphs and instanced quads.
//!
//! ## Architecture
//!
//! - **GlyphInstance**: Defines a single character to render.
//! - **TextLayout**: Helper for arranging text into lines and words.
//! - **WGSL Shader**: Implements the `smoothstep` anti-aliasing over the SDF texture.

use serde::{Deserialize, Serialize};

/// Error types for SDF text operations.
#[derive(Debug, Clone, PartialEq)]
pub enum SdfTextError {
    InvalidCharacter(char),
    AtlasFull,
    LayoutError(String),
}

impl std::fmt::Display for SdfTextError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidCharacter(c) => write!(f, "Invalid character: {}", c),
            Self::AtlasFull => write!(f, "SDF atlas is full"),
            Self::LayoutError(msg) => write!(f, "Layout error: {}", msg),
        }
    }
}

impl std::error::Error for SdfTextError {}

/// Represents a single character instance on the GPU.
/// Layout: `[pos_x, pos_y, pos_z, scale, uv_x, uv_y, uv_w, uv_h, color_r, color_g, color_b, color_a]`
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct GlyphInstance {
    pub position: [f32; 3],
    pub scale: f32,
    pub uv_rect: [f32; 4], // x, y, width, height in atlas
    pub color: [f32; 4],
}

impl GlyphInstance {
    pub fn new(position: [f32; 3], scale: f32, uv_rect: [f32; 4], color: [f32; 4]) -> Self {
        Self {
            position,
            scale,
            uv_rect,
            color,
        }
    }

    /// Pack into a raw f32 buffer for WebGPU.
    pub fn pack(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(&[
            self.position[0], self.position[1], self.position[2], self.scale,
            self.uv_rect[0], self.uv_rect[1], self.uv_rect[2], self.uv_rect[3],
            self.color[0], self.color[1], self.color[2], self.color[3],
        ]);
    }
}

/// A simple configuration for the SDF shader.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct SdfConfig {
    pub edge_value: f32, // typically 0.5
    pub smoothing: f32,  // typically 1.0 / pixel_range
}

impl Default for SdfConfig {
    fn default() -> Self {
        Self {
            edge_value: 0.5,
            smoothing: 0.05,
        }
    }
}

pub fn wgsl_sdf_text_shader() -> &'static str {
    r#"
struct GlyphInput {
    @location(0) position: vec3<f32>,
    @location(1) scale: f32,
    @location(2) uv_rect: vec4<f32>, // x, y, w, h
    @location(3) color: vec4<f32>,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    instance: GlyphInput,
) -> VertexOutput {
    // Generate quad: -1 to 1
    let x = f32(i32(in_vertex_index) % 2 * 2 - 1);
    let y = f32(i32(in_vertex_index) / 2 * 2 - 1);
    
    // UV generation: 0 to 1
    let u = f32(i32(in_vertex_index) % 2);
    let v = 1.0 - f32(i32(in_vertex_index) / 2);
    
    // Map to atlas UV rect
    let atlas_u = instance.uv_rect.x + u * instance.uv_rect.z;
    let atlas_v = instance.uv_rect.y + v * instance.uv_rect.w;
    
    var out: VertexOutput;
    out.clip_position = vec4<f32>(instance.position + vec3<f32>(x, y, 0.0) * instance.scale, 1.0);
    out.color = instance.color;
    out.uv = vec2<f32>(atlas_u, atlas_v);
    return out;
}

@group(0) @binding(0) var sdf_texture: texture_2d<f32>;
@group(0) @binding(1) var sdf_sampler: sampler;

struct SdfUniforms {
    edge_value: f32,
    smoothing: f32,
}
@group(0) @binding(2) var<uniform> uniforms: SdfUniforms;

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let distance = textureSample(sdf_texture, sdf_sampler, in.uv).r;
    let alpha = smoothstep(
        uniforms.edge_value - uniforms.smoothing,
        uniforms.edge_value + uniforms.smoothing,
        distance
    );
    
    if alpha < 0.01 { discard; }
    
    return vec4<f32>(in.color.rgb, in.color.a * alpha);
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_glyph_packing() {
        let mut buffer = Vec::new();
        let glyph = GlyphInstance::new(
            [1.0, 2.0, 3.0],
            0.5,
            [0.1, 0.2, 0.3, 0.4],
            [1.0, 0.0, 0.0, 1.0],
        );
        glyph.pack(&mut buffer);
        
        assert_eq!(buffer.len(), 12);
        assert_eq!(buffer[0], 1.0);
        assert_eq!(buffer[3], 0.5);
        assert_eq!(buffer[4], 0.1);
        assert_eq!(buffer[8], 1.0);
    }
    
    #[test]
    fn test_sdf_config_default() {
        let config = SdfConfig::default();
        assert_eq!(config.edge_value, 0.5);
        assert_eq!(config.smoothing, 0.05);
    }
    
    #[test]
    fn test_wgsl_shader_contains_smoothstep() {
        let shader = wgsl_sdf_text_shader();
        assert!(shader.contains("smoothstep"));
        assert!(shader.contains("textureSample"));
    }
}
