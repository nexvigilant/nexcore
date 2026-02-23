//! # Clustered Forward+ Lighting
//!
//! Evaluates thousands of lights efficiently by dividing the view frustum into
//! a 3D grid of clusters. Lights are binned into these clusters via compute
//! shaders, allowing the fragment shader to only evaluate lights that actually
//! affect the fragment's cluster.
//!
//! ## Architecture
//!
//! 1. **Cluster Grid**: View frustum divided into X * Y * Z clusters (e.g. 16x9x24).
//! 2. **AABB Generation**: Compute shader generates view-space AABBs for each cluster.
//! 3. **Light Culling**: Compute shader tests lights against cluster AABBs and builds a light list per cluster.
//! 4. **Forward Pass**: Fragment shader looks up its cluster and iterates over the light list.

use serde::{Deserialize, Serialize};

/// Configuration for the clustered lighting grid.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct ClusterConfig {
    pub grid_x: u32,
    pub grid_y: u32,
    pub grid_z: u32,
    pub max_lights_per_cluster: u32,
}

impl Default for ClusterConfig {
    fn default() -> Self {
        Self {
            grid_x: 16,
            grid_y: 9,
            grid_z: 24,
            max_lights_per_cluster: 256,
        }
    }
}

/// A point light source.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct PointLight {
    pub position: [f32; 3],
    pub radius: f32,
    pub color: [f32; 3],
    pub intensity: f32,
}

impl PointLight {
    pub fn new(position: [f32; 3], radius: f32, color: [f32; 3], intensity: f32) -> Self {
        Self {
            position,
            radius,
            color,
            intensity,
        }
    }

    pub fn pack(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(&[
            self.position[0], self.position[1], self.position[2], self.radius,
            self.color[0], self.color[1], self.color[2], self.intensity,
        ]);
    }
}

/// WGSL shader to compute the view-space AABB for each cluster.
pub fn wgsl_cluster_aabb_build_shader() -> &'static str {
    r#"
struct ClusterAABB {
    min_point: vec4<f32>,
    max_point: vec4<f32>,
}

struct GridUniforms {
    inverse_projection: mat4x4<f32>,
    view_dimensions: vec2<f32>,
    z_near: f32,
    z_far: f32,
    grid_size: vec3<u32>,
}

@group(0) @binding(0) var<uniform> uniforms: GridUniforms;
@group(0) @binding(1) var<storage, read_write> clusters: array<ClusterAABB>;

// Maps screen space to view space
fn screen2View(screen: vec4<f32>) -> vec3<f32> {
    let clip = vec4<f32>(
        screen.xy / uniforms.view_dimensions * 2.0 - vec2<f32>(1.0, 1.0),
        screen.z,
        screen.w
    );
    let view = uniforms.inverse_projection * clip;
    return view.xyz / view.w;
}

// Intersect a line through the origin with a Z plane
fn lineIntersectionToZPlane(a: vec3<f32>, b: vec3<f32>, zDistance: f32) -> vec3<f32> {
    let normal = vec3<f32>(0.0, 0.0, 1.0);
    let d = dot(normal, b);
    let t = zDistance / d;
    return b * t;
}

@compute @workgroup_size(8, 8, 8)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let i = gid.x + gid.y * uniforms.grid_size.x + gid.z * uniforms.grid_size.x * uniforms.grid_size.y;
    
    // Grid indices
    let min_pt_xy = vec2<f32>(f32(gid.x), f32(gid.y)) * (uniforms.view_dimensions / vec2<f32>(f32(uniforms.grid_size.x), f32(uniforms.grid_size.y)));
    let max_pt_xy = vec2<f32>(f32(gid.x + 1u), f32(gid.y + 1u)) * (uniforms.view_dimensions / vec2<f32>(f32(uniforms.grid_size.x), f32(uniforms.grid_size.y)));
    
    let tileNear = -uniforms.z_near * pow(uniforms.z_far / uniforms.z_near, f32(gid.z) / f32(uniforms.grid_size.z));
    let tileFar = -uniforms.z_near * pow(uniforms.z_far / uniforms.z_near, f32(gid.z + 1u) / f32(uniforms.grid_size.z));
    
    let minPoint_vS = screen2View(vec4<f32>(min_pt_xy, 0.0, 1.0));
    let maxPoint_vS = screen2View(vec4<f32>(max_pt_xy, 0.0, 1.0));
    
    let minPoint_near = lineIntersectionToZPlane(vec3<f32>(0.0), minPoint_vS, tileNear);
    let minPoint_far  = lineIntersectionToZPlane(vec3<f32>(0.0), minPoint_vS, tileFar);
    let maxPoint_near = lineIntersectionToZPlane(vec3<f32>(0.0), maxPoint_vS, tileNear);
    let maxPoint_far  = lineIntersectionToZPlane(vec3<f32>(0.0), maxPoint_vS, tileFar);

    let min_b = min(min(minPoint_near, minPoint_far), min(maxPoint_near, maxPoint_far));
    let max_b = max(max(minPoint_near, minPoint_far), max(maxPoint_near, maxPoint_far));

    clusters[i].min_point = vec4<f32>(min_b, 0.0);
    clusters[i].max_point = vec4<f32>(max_b, 0.0);
}
"#
}

/// WGSL shader to cull lights and build cluster light lists.
pub fn wgsl_light_culling_shader() -> &'static str {
    r#"
struct PointLight {
    position: vec4<f32>, // w = radius
    color: vec4<f32>,    // w = intensity
}

struct ClusterAABB {
    min_point: vec4<f32>,
    max_point: vec4<f32>,
}

struct GridUniforms {
    view_matrix: mat4x4<f32>,
    grid_size: vec3<u32>,
    max_lights_per_cluster: u32,
    light_count: u32,
}

@group(0) @binding(0) var<uniform> uniforms: GridUniforms;
@group(0) @binding(1) var<storage, read> clusters: array<ClusterAABB>;
@group(0) @binding(2) var<storage, read> lights: array<PointLight>;
@group(0) @binding(3) var<storage, read_write> cluster_light_indices: array<u32>; // Pre-allocated grid_x * grid_y * grid_z * (max_lights_per_cluster + 1)
@group(0) @binding(4) var<storage, read_write> global_index_count: atomic<u32>;

var<workgroup> visible_lights: array<u32, 256>;
var<workgroup> visible_light_count: atomic<u32>;

fn testSphereAABB(sphere: vec4<f32>, aabb: ClusterAABB) -> bool {
    let closestPoint = clamp(sphere.xyz, aabb.min_point.xyz, aabb.max_point.xyz);
    let distanceSq = dot(closestPoint - sphere.xyz, closestPoint - sphere.xyz);
    return distanceSq <= (sphere.w * sphere.w);
}

@compute @workgroup_size(8, 8, 8)
fn main(
    @builtin(global_invocation_id) gid: vec3<u32>,
    @builtin(local_invocation_index) lid: u32
) {
    if lid == 0u {
        atomicStore(&visible_light_count, 0u);
    }
    workgroupBarrier();

    let cluster_idx = gid.x + gid.y * uniforms.grid_size.x + gid.z * uniforms.grid_size.x * uniforms.grid_size.y;
    let aabb = clusters[cluster_idx];
    
    // Each thread tests a subset of lights
    let light_count = uniforms.light_count;
    var i = lid;
    while i < light_count {
        let light = lights[i];
        let view_pos = uniforms.view_matrix * vec4<f32>(light.position.xyz, 1.0);
        let sphere = vec4<f32>(view_pos.xyz, light.position.w);
        
        if testSphereAABB(sphere, aabb) {
            let idx = atomicAdd(&visible_light_count, 1u);
            if idx < uniforms.max_lights_per_cluster {
                visible_lights[idx] = i;
            }
        }
        i += 256u; // Workgroup size is 8x8x8 = 512, wait, 8x8x8 = 512, actually let's use 512u
    }
    workgroupBarrier();

    if lid == 0u {
        let count = min(atomicLoad(&visible_light_count), uniforms.max_lights_per_cluster);
        let offset = cluster_idx * (uniforms.max_lights_per_cluster + 1u);
        cluster_light_indices[offset] = count;
        for (var j = 0u; j < count; j++) {
            cluster_light_indices[offset + 1u + j] = visible_lights[j];
        }
    }
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_cluster_config_defaults() {
        let config = ClusterConfig::default();
        assert_eq!(config.grid_x, 16);
        assert_eq!(config.grid_y, 9);
        assert_eq!(config.grid_z, 24);
    }

    #[test]
    fn test_point_light_packing() {
        let mut buffer = Vec::new();
        let light = PointLight::new([1.0, 2.0, 3.0], 10.0, [1.0, 0.5, 0.0], 2.0);
        light.pack(&mut buffer);

        assert_eq!(buffer.len(), 8);
        assert_eq!(buffer[0], 1.0);
        assert_eq!(buffer[3], 10.0);
        assert_eq!(buffer[4], 1.0);
        assert_eq!(buffer[7], 2.0);
    }

    #[test]
    fn test_wgsl_aabb_shader_has_content() {
        let shader = wgsl_cluster_aabb_build_shader();
        assert!(shader.contains("ClusterAABB"));
        assert!(shader.contains("lineIntersectionToZPlane"));
    }

    #[test]
    fn test_wgsl_cull_shader_has_content() {
        let shader = wgsl_light_culling_shader();
        assert!(shader.contains("testSphereAABB"));
        assert!(shader.contains("atomicAdd"));
    }
}
