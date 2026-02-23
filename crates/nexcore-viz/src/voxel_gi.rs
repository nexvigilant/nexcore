//! # Voxel Global Illumination (VoxelGI) for Data Topologies
//!
//! Provides data structures and WGSL compute shaders for voxel-based real-time
//! global illumination (RTGI). Rather than world geometry, this is adapted for
//! dense data topologies (e.g., millions of points acting as emissive and occluding volumes).
//!
//! ## Architecture
//!
//! 1. **Voxelization**: Compute shaders to inject data points/instances into a 3D voxel grid.
//! 2. **Mipmap Generation**: Downsample the voxel grid for anisotropic cone tracing.
//! 3. **Cone Tracing**: Fragment shaders sample the voxel grid along cones to compute indirect diffuse and specular lighting.

use serde::{Deserialize, Serialize};

/// Configuration for the Voxel GI grid.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct VoxelGiConfig {
    pub grid_resolution: u32,
    pub volume_size: f32,
    pub max_bounces: u32,
    pub diffuse_cones: u32,
}

impl Default for VoxelGiConfig {
    fn default() -> Self {
        Self {
            grid_resolution: 128,
            volume_size: 100.0,
            max_bounces: 1,
            diffuse_cones: 6,
        }
    }
}

/// WGSL shader for injecting instances into the voxel grid.
pub fn wgsl_voxelize_shader() -> &'static str {
    r#"
struct InstanceData {
    position: vec4<f32>,
    color: vec4<f32>, // Emissive/Albedo
}

struct VoxelGridUniforms {
    grid_resolution: u32,
    volume_size: f32,
    volume_center: vec3<f32>,
    instance_count: u32,
}

@group(0) @binding(0) var<uniform> uniforms: VoxelGridUniforms;
@group(0) @binding(1) var<storage, read> instances: array<InstanceData>;
@group(0) @binding(2) var voxel_albedo: texture_storage_3d<rgba8unorm, write>;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= uniforms.instance_count {
        return;
    }

    let instance = instances[idx];
    
    // Map position to voxel grid coordinates
    let half_size = uniforms.volume_size * 0.5;
    let local_pos = instance.position.xyz - uniforms.volume_center + vec3<f32>(half_size);
    let normalized_pos = local_pos / uniforms.volume_size;
    let grid_pos = vec3<i32>(normalized_pos * f32(uniforms.grid_resolution));
    
    if all(grid_pos >= vec3<i32>(0)) && all(grid_pos < vec3<i32>(i32(uniforms.grid_resolution))) {
        // Simple injection: in a real implementation this might use atomic operations
        // on a flattened buffer, or write to a 3D texture using imageStore.
        textureStore(voxel_albedo, grid_pos, instance.color);
    }
}
"#
}

/// WGSL shader for Voxel Cone Tracing (Indirect Lighting).
pub fn wgsl_voxel_cone_tracing_shader() -> &'static str {
    r#"
struct VoxelUniforms {
    grid_resolution: u32,
    volume_size: f32,
    volume_center: vec3<f32>,
    max_mip_level: f32,
}

@group(0) @binding(0) var<uniform> uniforms: VoxelUniforms;
@group(0) @binding(1) var voxel_texture: texture_3d<f32>;
@group(0) @binding(2) var voxel_sampler: sampler;

// Trace a cone through the voxel grid
fn traceCone(start_pos: vec3<f32>, direction: vec3<f32>, aperture: f32, max_distance: f32) -> vec4<f32> {
    var accumulated_color = vec4<f32>(0.0);
    var distance = uniforms.volume_size / f32(uniforms.grid_resolution); // Start step
    
    while distance < max_distance && accumulated_color.a < 1.0 {
        let current_pos = start_pos + direction * distance;
        let local_pos = current_pos - uniforms.volume_center + vec3<f32>(uniforms.volume_size * 0.5);
        let uvw = local_pos / uniforms.volume_size;
        
        if any(uvw < vec3<f32>(0.0)) || any(uvw > vec3<f32>(1.0)) {
            break; // Out of bounds
        }
        
        let diameter = 2.0 * aperture * distance;
        let mip_level = clamp(log2(diameter * f32(uniforms.grid_resolution) / uniforms.volume_size), 0.0, uniforms.max_mip_level);
        
        let sample = textureSampleLevel(voxel_texture, voxel_sampler, uvw, mip_level);
        
        // Front-to-back blending
        let alpha = sample.a * (1.0 - accumulated_color.a);
        accumulated_color += vec4<f32>(sample.rgb * alpha, alpha);
        
        distance += max(diameter, uniforms.volume_size / f32(uniforms.grid_resolution));
    }
    
    return accumulated_color;
}

@fragment
fn fs_main(@location(0) world_pos: vec3<f32>, @location(1) normal: vec3<f32>) -> @location(0) vec4<f32> {
    // Diffuse indirect lighting using orthogonal cones (simplified)
    let up = vec3<f32>(0.0, 1.0, 0.0);
    let right = normalize(cross(up, normal));
    let forward = cross(normal, right);
    
    let aperture = 0.577; // Roughly 60 degrees
    let max_dist = uniforms.volume_size;
    
    var indirect_diffuse = vec4<f32>(0.0);
    indirect_diffuse += traceCone(world_pos, normal, aperture, max_dist);
    // Add more cones for full hemisphere coverage in production...
    
    return vec4<f32>(indirect_diffuse.rgb, 1.0);
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_voxel_gi_config_defaults() {
        let config = VoxelGiConfig::default();
        assert_eq!(config.grid_resolution, 128);
        assert_eq!(config.diffuse_cones, 6);
    }

    #[test]
    fn test_wgsl_voxelize_shader_has_content() {
        let shader = wgsl_voxelize_shader();
        assert!(shader.contains("textureStore"));
        assert!(shader.contains("grid_pos"));
    }

    #[test]
    fn test_wgsl_cone_tracing_shader_has_content() {
        let shader = wgsl_voxel_cone_tracing_shader();
        assert!(shader.contains("traceCone"));
        assert!(shader.contains("textureSampleLevel"));
    }
}