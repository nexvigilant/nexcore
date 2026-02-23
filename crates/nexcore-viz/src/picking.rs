//! # GPU Picking Pipeline
//!
//! Provides the data structures and WebGPU WGSL shaders required to implement
//! a pixel-perfect GPU picking pipeline. It uses the `decode_pick_color`
//! and `encode_pick_color` logic from the `instancing` module to map unique
//! instance IDs to RGBA colors and back.
//!
//! ## Architecture
//!
//! 1. Render all interactive instances to an offscreen texture, replacing
//!    their visual color with their encoded `u32` ID.
//! 2. Read back the pixel under the cursor (or a region).
//! 3. Decode the RGBA float value back to a `u32` instance ID.

#[allow(unused_imports)]
use crate::instancing::{decode_pick_color, encode_pick_color};
use serde::{Deserialize, Serialize};

/// Represents a pick request at specific viewport coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PickRequest {
    pub x: u32,
    pub y: u32,
}

/// Represents the result of a pick operation.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct PickResult {
    pub x: u32,
    pub y: u32,
    pub instance_id: Option<u32>,
    pub depth: f32,
}

/// WGSL shader for the picking pass vertex step.
/// Typically reuses the instancing vertex shader but outputs the ID color.
pub fn wgsl_picking_vertex_shader() -> &'static str {
    r#"
struct InstanceInput {
    @location(0) position: vec3<f32>,
    @location(1) scale: f32,
    @location(2) color: vec4<f32>,
    @location(3) instance_id: u32,
    @location(4) metadata: u32,
};

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pick_color: vec4<f32>,
};

// Extracted from instancing::encode_pick_color logic
fn encode_id(id: u32) -> vec4<f32> {
    let r = f32((id >> 16u) & 0xFFu) / 255.0;
    let g = f32((id >> 8u) & 0xFFu) / 255.0;
    let b = f32(id & 0xFFu) / 255.0;
    let a = f32((id >> 24u) & 0xFFu) / 255.0;
    return vec4<f32>(r, g, b, a);
}

@vertex
fn vs_main(
    @builtin(vertex_index) in_vertex_index: u32,
    instance: InstanceInput,
) -> VertexOutput {
    let x = f32(i32(in_vertex_index) % 2 * 2 - 1);
    let y = f32(i32(in_vertex_index) / 2 * 2 - 1);
    
    var out: VertexOutput;
    // Assuming identity/ortho for picking example
    out.clip_position = vec4<f32>(instance.position + vec3<f32>(x, y, 0.0) * instance.scale, 1.0);
    out.pick_color = encode_id(instance.instance_id);
    return out;
}
"#
}

/// WGSL shader for the picking pass fragment step.
pub fn wgsl_picking_fragment_shader() -> &'static str {
    r#"
struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) pick_color: vec4<f32>,
};

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    return in.pick_color;
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_pick_request_creation() {
        let req = PickRequest { x: 100, y: 200 };
        assert_eq!(req.x, 100);
        assert_eq!(req.y, 200);
    }

    #[test]
    fn test_pick_result_creation() {
        let res = PickResult {
            x: 100,
            y: 200,
            instance_id: Some(42),
            depth: 0.5,
        };
        assert_eq!(res.instance_id.unwrap(), 42);
    }

    #[test]
    fn test_encode_decode_pick_color_wrapper() {
        // Just verifying the wrapper logic holds true
        let original_id = 1337;
        let color = encode_pick_color(original_id);
        let decoded = decode_pick_color(color);
        assert_eq!(original_id, decoded);
    }

    #[test]
    fn test_wgsl_picking_vertex_shader_has_content() {
        let shader = wgsl_picking_vertex_shader();
        assert!(shader.contains("encode_id"));
        assert!(shader.contains("@vertex"));
    }

    #[test]
    fn test_wgsl_picking_fragment_shader_has_content() {
        let shader = wgsl_picking_fragment_shader();
        assert!(shader.contains("pick_color"));
        assert!(shader.contains("@fragment"));
    }
}
