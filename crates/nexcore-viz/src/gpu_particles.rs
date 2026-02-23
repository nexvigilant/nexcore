//! # Compute-based GPU Particles
//!
//! A 100% GPU-driven particle system. Particle state is kept entirely in VRAM
//! and updated via WGSL compute shaders, removing the CPU integration bottleneck
//! and allowing visualization of millions of concurrent dynamic agents.
//!
//! ## Architecture
//!
//! 1. **Particle Buffers**: Double-buffered or ping-pong storage for Position, Velocity, Color, Life.
//! 2. **Emission Compute**: Spawns new particles from data sources or visual emitters.
//! 3. **Update Compute**: Integrates forces (gravity, curl noise, attraction) and updates positions/lifetimes.

use serde::{Deserialize, Serialize};

/// Configuration for the GPU Particle System.
#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
pub struct GpuParticleConfig {
    pub max_particles: u32,
    pub emission_rate: f32, // particles per second
    pub dt: f32,            // fixed timestep
}

impl Default for GpuParticleConfig {
    fn default() -> Self {
        Self {
            max_particles: 1_000_000,
            emission_rate: 10_000.0,
            dt: 0.016,
        }
    }
}

/// WGSL shader for updating particle states.
pub fn wgsl_particle_update_shader() -> &'static str {
    r#"
struct Particle {
    position: vec3<f32>,
    life: f32, // < 0 means dead
    velocity: vec3<f32>,
    color_packed: u32, // RGBA 8-bit
}

struct ParticleUniforms {
    max_particles: u32,
    dt: f32,
    time: f32,
    gravity: vec3<f32>,
}

@group(0) @binding(0) var<uniform> uniforms: ParticleUniforms;
@group(0) @binding(1) var<storage, read_write> particles: array<Particle>;

// Simple pseudo-random hash
fn hash(seed: u32) -> f32 {
    var state = seed * 747796405u + 2891336453u;
    state = ((state >> ((state >> 28u) + 4u)) ^ state) * 277803737u;
    state = (state >> 22u) ^ state;
    return f32(state) / 4294967295.0;
}

@compute @workgroup_size(256)
fn main(@builtin(global_invocation_id) gid: vec3<u32>) {
    let idx = gid.x;
    if idx >= uniforms.max_particles {
        return;
    }

    var p = particles[idx];
    
    if p.life > 0.0 {
        // Alive: Update
        p.life -= uniforms.dt;
        p.velocity += uniforms.gravity * uniforms.dt;
        
        // Add some noise/turbulence based on position and time
        let noise_force = vec3<f32>(
            hash(idx + u32(uniforms.time * 1000.0)) * 2.0 - 1.0,
            hash(idx + 1u + u32(uniforms.time * 1000.0)) * 2.0 - 1.0,
            hash(idx + 2u + u32(uniforms.time * 1000.0)) * 2.0 - 1.0
        ) * 0.5;
        
        p.velocity += noise_force * uniforms.dt;
        p.position += p.velocity * uniforms.dt;
        
    } else {
        // Dead: Respawn logic (simplified - typically handled in a separate pass or atomic counter)
        // For demonstration, we softly recycle them at origin
        if hash(idx + u32(uniforms.time * 10000.0)) < 0.01 {
            p.life = 5.0 + hash(idx) * 2.0;
            p.position = vec3<f32>(0.0);
            p.velocity = vec3<f32>(
                hash(idx + 3u) * 2.0 - 1.0,
                hash(idx + 4u) * 5.0,
                hash(idx + 5u) * 2.0 - 1.0
            );
        }
    }
    
    particles[idx] = p;
}
"#
}

/// WGSL shader for rendering GPU particles (Vertex + Fragment).
pub fn wgsl_particle_render_shader() -> &'static str {
    r#"
struct Particle {
    position: vec3<f32>,
    life: f32,
    velocity: vec3<f32>,
    color_packed: u32,
}

struct ViewUniforms {
    view_proj: mat4x4<f32>,
    camera_right: vec3<f32>,
    camera_up: vec3<f32>,
}

@group(0) @binding(0) var<uniform> view: ViewUniforms;
@group(0) @binding(1) var<storage, read> particles: array<Particle>;

struct VertexOutput {
    @builtin(position) clip_position: vec4<f32>,
    @location(0) color: vec4<f32>,
    @location(1) uv: vec2<f32>,
};

fn unpack_color(c: u32) -> vec4<f32> {
    return vec4<f32>(
        f32((c >> 16u) & 0xFFu) / 255.0,
        f32((c >> 8u) & 0xFFu) / 255.0,
        f32(c & 0xFFu) / 255.0,
        f32((c >> 24u) & 0xFFu) / 255.0,
    );
}

@vertex
fn vs_main(
    @builtin(vertex_index) v_idx: u32,
    @builtin(instance_index) i_idx: u32,
) -> VertexOutput {
    let p = particles[i_idx];
    
    // Output degenerate quad if dead
    if p.life <= 0.0 {
        var out: VertexOutput;
        out.clip_position = vec4<f32>(0.0);
        return out;
    }
    
    // Quad vertices
    let x = f32(i32(v_idx) % 2 * 2 - 1);
    let y = f32(i32(v_idx) / 2 * 2 - 1);
    
    // Billboard logic
    let particle_size = 0.05 * clamp(p.life, 0.0, 1.0);
    let local_pos = (view.camera_right * x + view.camera_up * y) * particle_size;
    let world_pos = p.position + local_pos;
    
    var out: VertexOutput;
    out.clip_position = view.view_proj * vec4<f32>(world_pos, 1.0);
    out.color = unpack_color(p.color_packed);
    out.uv = vec2<f32>(x, y);
    return out;
}

@fragment
fn fs_main(in: VertexOutput) -> @location(0) vec4<f32> {
    let dist = length(in.uv);
    if dist > 1.0 { discard; }
    
    // Soft particle edge
    let alpha = (1.0 - dist) * in.color.a;
    return vec4<f32>(in.color.rgb, alpha);
}
"#
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_gpu_particle_config_defaults() {
        let config = GpuParticleConfig::default();
        assert_eq!(config.max_particles, 1_000_000);
        assert!((config.dt - 0.016).abs() < 1e-4);
    }

    #[test]
    fn test_wgsl_update_shader_has_content() {
        let shader = wgsl_particle_update_shader();
        assert!(shader.contains("particles[idx]"));
        assert!(shader.contains("hash("));
    }

    #[test]
    fn test_wgsl_render_shader_has_content() {
        let shader = wgsl_particle_render_shader();
        assert!(shader.contains("unpack_color"));
        assert!(shader.contains("camera_right"));
    }
}