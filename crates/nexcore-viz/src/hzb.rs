//! # Hierarchical Z-Buffer (HZB) Occlusion Culling
//!
//! Two-phase occlusion culling on the GPU using a Hierarchical Z-Buffer.
//! 
//! ## Architecture
//!
//! 1. **Phase 1: Previous-Frame Culling**
//!    Test all instances against the HZB from the previous frame. 
//!    Visible instances are written to an indirect draw buffer.
//! 2. **Render Phase 1**
//!    Issue an indirect draw for these highly-likely-visible instances, generating
//!    a conservative depth buffer for the current frame.
//! 3. **HZB Generation**
//!    Downsample the new depth buffer to create the current frame's HZB (mip chain).
//! 4. **Phase 2: Current-Frame Culling**
//!    Test the instances that *failed* Phase 1 against the new HZB.
//!    Newly visible instances (disoccluded) are written to a second indirect draw buffer.
//! 5. **Render Phase 2**
//!    Issue a second indirect draw for the newly visible instances.

use serde::{Deserialize, Serialize};

/// Configuration for HZB generation and culling.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct HzbConfig {
    /// Depth texture dimensions
    pub width: u32,
    pub height: u32,
    /// Number of mip levels to generate
    pub mip_levels: u32,
}

impl HzbConfig {
    pub fn new(width: u32, height: u32) -> Self {
        let max_dim = width.max(height) as f32;
        let mip_levels = max_dim.log2().ceil() as u32 + 1;
        Self {
            width,
            height,
            mip_levels,
        }
    }
}

/// WGSL shader for downsampling the depth buffer (generating HZB mips).
/// A compute shader that reads from mip N and writes the min depth to mip N+1.
pub fn wgsl_hzb_downsample_shader() -> &'static str {
    r#"
@group(0) @binding(0) var source_depth: texture_2d<f32>;
@group(0) @binding(1) var dest_depth: texture_storage_2d<r32float, write>;

@compute @workgroup_size(16, 16)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let dim = textureDimensions(dest_depth);
    if gid.x >= dim.x || gid.y >= dim.y {
        return;
    }

    let src_pos = gid.xy * 2u;
    
    // Sample 2x2 footprint and take the MAX depth (assuming reverse-Z, 0 = far, 1 = near)
    // If using standard Z (0 = near, 1 = far), we would take the MIN.
    // Assuming standard Z for this example: MIN depth is closest to camera.
    // Wait, for occlusion, we want the furthest point in the footprint to conservatively test against.
    // So if using standard Z (0 near, 1 far), we want MAX depth in the footprint.
    // If using reverse Z (1 near, 0 far), we want MIN depth.
    // Let's assume standard Z (1 = far), so we use max().
    
    let d0 = textureLoad(source_depth, src_pos, 0).r;
    let d1 = textureLoad(source_depth, src_pos + vec2<u32>(1u, 0u), 0).r;
    let d2 = textureLoad(source_depth, src_pos + vec2<u32>(0u, 1u), 0).r;
    let d3 = textureLoad(source_depth, src_pos + vec2<u32>(1u, 1u), 0).r;

    let max_depth = max(max(d0, d1), max(d2, d3));
    
    textureStore(dest_depth, gid.xy, vec4<f32>(max_depth, 0.0, 0.0, 0.0));
}
"#
}

/// WGSL shader for HZB culling.
/// Tests an AABB against the HZB to determine visibility.
pub fn wgsl_hzb_cull_shader() -> &'static str {
    r#"
struct InstanceAABB {
    min: vec3<f32>,
    max: vec3<f32>,
}

@group(0) @binding(0) var<storage, read> instances: array<InstanceAABB>;
@group(0) @binding(1) var hzb_texture: texture_2d<f32>;
@group(0) @binding(2) var hzb_sampler: sampler;

// Indirect draw command buffer
struct DrawIndexedIndirect {
    index_count: atomic<u32>,
    instance_count: atomic<u32>,
    first_index: u32,
    base_vertex: i32,
    first_instance: u32,
}
@group(0) @binding(3) var<storage, read_write> draw_cmd: DrawIndexedIndirect;
@group(0) @binding(4) var<storage, read_write> instance_indices: array<u32>;

struct CullUniforms {
    view_proj: mat4x4<f32>,
    hzb_width: f32,
    hzb_height: f32,
    instance_count: u32,
}
@group(0) @binding(5) var<uniform> uniforms: CullUniforms;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= uniforms.instance_count {
        return;
    }

    let aabb = instances[idx];
    
    // 1. Transform AABB to clip space to get 2D bounding box
    // (Simplified for illustration: real implementation transforms 8 corners)
    let center = (aabb.min + aabb.max) * 0.5;
    let extents = (aabb.max - aabb.min) * 0.5;
    
    let clip_pos = uniforms.view_proj * vec4<f32>(center, 1.0);
    let ndc_pos = clip_pos.xyz / clip_pos.w;
    
    // Very simplified bounds projection
    let clip_extents = extents * abs(uniforms.view_proj[0].x); 
    
    let min_ndc = ndc_pos.xy - clip_extents.xy;
    let max_ndc = ndc_pos.xy + clip_extents.xy;
    let nearest_z = ndc_pos.z - clip_extents.z; // Assuming standard Z (0 to 1)

    // 2. Map NDC to UV [0, 1]
    let min_uv = min_ndc * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);
    let max_uv = max_ndc * vec2<f32>(0.5, -0.5) + vec2<f32>(0.5, 0.5);

    // 3. Determine HZB mip level
    let pixel_width = (max_uv.x - min_uv.x) * uniforms.hzb_width;
    let pixel_height = (max_uv.y - min_uv.y) * uniforms.hzb_height;
    let max_dim = max(pixel_width, pixel_height);
    let lod = ceil(log2(max_dim));

    // 4. Sample HZB (using sampler with point filtering)
    // Real impl samples 4 pixels at this LOD level to cover the box
    let center_uv = (min_uv + max_uv) * 0.5;
    let hzb_depth = textureSampleLevel(hzb_texture, hzb_sampler, center_uv, lod).r;

    // 5. Visibility test (assuming standard Z where 1.0 is far)
    let is_visible = nearest_z <= hzb_depth;

    if is_visible {
        // Append to indirect draw buffer
        let out_idx = atomicAdd(&draw_cmd.instance_count, 1u);
        instance_indices[out_idx] = idx;
    }
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_hzb_config_mip_levels() {
        // 1024x1024 -> log2(1024) = 10 -> +1 = 11 levels
        let config = HzbConfig::new(1024, 1024);
        assert_eq!(config.mip_levels, 11);

        // 1920x1080 -> max is 1920 -> log2(1920) ~ 10.9 -> ceil is 11 -> +1 = 12 levels
        let config2 = HzbConfig::new(1920, 1080);
        assert_eq!(config2.mip_levels, 12);
    }
    
    #[test]
    fn test_wgsl_downsample_shader() {
        let shader = wgsl_hzb_downsample_shader();
        assert!(shader.contains("max(max(")); // verifies it's doing 2x2 reduction
        assert!(shader.contains("textureStore"));
    }

    #[test]
    fn test_wgsl_cull_shader() {
        let shader = wgsl_hzb_cull_shader();
        assert!(shader.contains("DrawIndexedIndirect"));
        assert!(shader.contains("textureSampleLevel"));
        assert!(shader.contains("atomicAdd"));
    }
}
