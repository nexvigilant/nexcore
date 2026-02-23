//! # Volumetric Rendering
//!
//! Evaluates and renders 3D data density fields (e.g., pharmacovigilance signal
//! density) as volumetric fog or clouds using a raymarching approach.
//!
//! ## Architecture
//!
//! - **VolumeData**: Metadata about the 3D texture containing the density field.
//! - **Raymarch Shader**: WGSL fragment shader that steps through the volume,
//!   sampling density and integrating lighting from the clustered light grid.

use serde::{Deserialize, Serialize};

/// Configuration for volumetric rendering.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VolumeConfig {
    /// Number of steps along the ray through the volume.
    pub steps: u32,
    /// Base absorption coefficient.
    pub absorption: f32,
    /// Base scattering coefficient.
    pub scattering: f32,
    /// Henyey-Greenstein phase function asymmetry parameter (g) in [-1, 1].
    pub phase_g: f32,
    /// Global density multiplier.
    pub density_scale: f32,
}

impl Default for VolumeConfig {
    fn default() -> Self {
        Self {
            steps: 64,
            absorption: 0.1,
            scattering: 0.5,
            phase_g: 0.3,
            density_scale: 1.0,
        }
    }
}

/// Metadata describing the 3D texture bounds in world space.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VolumeBounds {
    pub min_extents: [f32; 3],
    pub max_extents: [f32; 3],
}

impl VolumeBounds {
    pub fn new(min_extents: [f32; 3], max_extents: [f32; 3]) -> Self {
        Self {
            min_extents,
            max_extents,
        }
    }

    pub fn pack(&self, buffer: &mut Vec<f32>) {
        buffer.extend_from_slice(&[
            self.min_extents[0],
            self.min_extents[1],
            self.min_extents[2],
            0.0,
            self.max_extents[0],
            self.max_extents[1],
            self.max_extents[2],
            0.0,
        ]);
    }
}

/// WGSL shader for raymarching the volume and integrating lighting.
pub fn wgsl_raymarch_volume_shader() -> &'static str {
    r#"
struct VolumeUniforms {
    min_extents: vec3<f32>,
    max_extents: vec3<f32>,
    steps: u32,
    absorption: f32,
    scattering: f32,
    phase_g: f32,
    density_scale: f32,
    camera_pos: vec3<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: VolumeUniforms;
@group(0) @binding(1) var volume_texture: texture_3d<f32>;
@group(0) @binding(2) var volume_sampler: sampler;
@group(0) @binding(3) var depth_texture: texture_depth_2d;

// Henyey-Greenstein Phase Function
fn phase_hg(cos_theta: f32, g: f32) -> f32 {
    let g2 = g * g;
    let denom = 1.0 + g2 - 2.0 * g * cos_theta;
    return (1.0 - g2) / (4.0 * 3.14159265 * pow(denom, 1.5));
}

// Ray-AABB intersection
fn intersectAABB(ro: vec3<f32>, rd: vec3<f32>, boxMin: vec3<f32>, boxMax: vec3<f32>) -> vec2<f32> {
    let t0 = (boxMin - ro) / rd;
    let t1 = (boxMax - ro) / rd;
    let tmin = min(t0, t1);
    let tmax = max(t0, t1);
    
    let tnear = max(max(tmin.x, tmin.y), tmin.z);
    let tfar = min(min(tmax.x, tmax.y), tmax.z);
    
    return vec2<f32>(tnear, tfar);
}

@fragment
fn fs_main(@builtin(position) frag_coord: vec4<f32>, @location(0) world_pos: vec3<f32>) -> @location(0) vec4<f32> {
    let ro = uniforms.camera_pos;
    let rd = normalize(world_pos - ro);
    
    // Get depth buffer value to avoid marching behind geometry
    let scene_depth = textureLoad(depth_texture, vec2<i32>(frag_coord.xy), 0);
    // Convert to linear depth here (implementation depends on projection matrix)
    let max_dist = 1000.0; // Placeholder for actual depth calculation
    
    let hit = intersectAABB(ro, rd, uniforms.min_extents, uniforms.max_extents);
    let tnear = max(0.0, hit.x);
    let tfar = min(hit.y, max_dist);
    
    if tnear >= tfar {
        discard;
    }
    
    let step_size = (tfar - tnear) / f32(uniforms.steps);
    var t = tnear;
    var transmittance = 1.0;
    var scattered_light = vec3<f32>(0.0);
    
    for (var i = 0u; i < uniforms.steps; i++) {
        let pos = ro + rd * t;
        
        // Map to 3D texture UVW
        let uvw = (pos - uniforms.min_extents) / (uniforms.max_extents - uniforms.min_extents);
        let density = textureSampleLevel(volume_texture, volume_sampler, uvw, 0.0).r * uniforms.density_scale;
        
        if density > 0.0 {
            // Simplified lighting: just constant ambient for now, real version uses clustered lighting
            let ambient = vec3<f32>(0.1, 0.15, 0.2);
            let in_scatter = uniforms.scattering * density * ambient;
            let extinction = (uniforms.absorption + uniforms.scattering) * density;
            
            let transmittance_eval = exp(-extinction * step_size);
            scattered_light += transmittance * in_scatter * (1.0 - transmittance_eval) / max(extinction, 0.001);
            transmittance *= transmittance_eval;
            
            if transmittance < 0.01 {
                break;
            }
        }
        
        t += step_size;
    }
    
    return vec4<f32>(scattered_light, 1.0 - transmittance);
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_volume_config_default() {
        let config = VolumeConfig::default();
        assert_eq!(config.steps, 64);
        assert_eq!(config.density_scale, 1.0);
    }

    #[test]
    fn test_volume_bounds_packing() {
        let bounds = VolumeBounds::new([-1.0, -1.0, -1.0], [1.0, 1.0, 1.0]);
        let mut buffer = Vec::new();
        bounds.pack(&mut buffer);

        assert_eq!(buffer.len(), 8);
        assert_eq!(buffer[0], -1.0);
        assert_eq!(buffer[4], 1.0);
    }

    #[test]
    fn test_wgsl_raymarch_shader_has_content() {
        let shader = wgsl_raymarch_volume_shader();
        assert!(shader.contains("intersectAABB"));
        assert!(shader.contains("phase_hg"));
        assert!(shader.contains("textureSampleLevel"));
    }
}
