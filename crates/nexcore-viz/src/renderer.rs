//! # WebGPU Rendering Pipeline Infrastructure
//!
//! Pure-Rust rendering pipeline types, WGSL shader code generation, and
//! post-processing algorithm implementations for Phase 5 "Eyes" of the
//! nexcore visualization engine.
//!
//! ## Overview
//!
//! This module provides:
//! - Pipeline state configuration types (`PipelineState`, `BlendMode`, etc.)
//! - Quality-level management with adaptive FPS-driven adjustment
//! - WGSL shader string generation for molecular, TAA, GTAO, and SSS passes
//! - Temporal Anti-Aliasing (TAA) Halton jitter sequences and blend factors
//! - Ground Truth Ambient Occlusion (GTAO) sample generation and falloff
//! - Subsurface Scattering (SSS) Christensen-Burley diffusion profiles
//!
//! All computation is pure Rust with `f64` arithmetic; no GPU runtime is
//! required. Shaders are returned as `String` values for offline inspection,
//! caching, or transmission to a WebGPU host.
//!
//! ## Example
//!
//! ```rust
//! use nexcore_viz::renderer::{
//!     default_pipeline, quality_settings, QualityLevel,
//!     generate_molecular_vertex_shader,
//! };
//!
//! let pipeline = default_pipeline();
//! let settings = quality_settings(QualityLevel::High);
//! let vert = generate_molecular_vertex_shader();
//! assert!(vert.contains("fn "));
//! assert!(vert.contains("position"));
//! assert_eq!(settings.ao_samples, 16);
//! ```

use serde::{Deserialize, Serialize};
use std::fmt;

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors produced by the rendering pipeline infrastructure.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum RendererError {
    /// A pipeline or shader configuration value is invalid.
    InvalidConfig(String),
    /// WGSL shader source generation failed for the given reason.
    ShaderGenerationFailed(String),
    /// The requested MSAA sample count is not supported.
    InvalidSampleCount,
}

impl fmt::Display for RendererError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidConfig(msg) => write!(f, "Invalid renderer config: {msg}"),
            Self::ShaderGenerationFailed(msg) => {
                write!(f, "Shader generation failed: {msg}")
            }
            Self::InvalidSampleCount => {
                write!(f, "Invalid sample count: must be 1, 2, 4, 8, or 16")
            }
        }
    }
}

impl std::error::Error for RendererError {}

// ---------------------------------------------------------------------------
// Pipeline state enums
// ---------------------------------------------------------------------------

/// Alpha-blending equation applied during rasterisation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum BlendMode {
    /// No blending — fragment colour replaces the framebuffer directly.
    Opaque,
    /// Standard over-compositing: `src_alpha * src + (1 - src_alpha) * dst`.
    AlphaBlend,
    /// Additive blending: `src + dst`.
    Additive,
    /// Premultiplied alpha: `src + (1 - src_alpha) * dst`.
    Premultiplied,
}

/// Depth comparison function used during rasterisation.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum DepthFunc {
    /// Pass if incoming depth is strictly less than stored depth.
    Less,
    /// Pass if incoming depth is less than or equal to stored depth.
    LessEqual,
    /// Pass if incoming depth is strictly greater than stored depth.
    Greater,
    /// Always pass — depth test disabled.
    Always,
    /// Never pass — all fragments discarded.
    Never,
}

/// Face-culling mode.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum CullMode {
    /// No face culling.
    None,
    /// Cull front-facing (CCW) polygons.
    Front,
    /// Cull back-facing (CW) polygons.
    Back,
}

/// Anti-aliasing method applied to the final frame.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum AaMethod {
    /// No anti-aliasing.
    None,
    /// Multisample AA with 4 samples per pixel.
    Msaa4x,
    /// Temporal Anti-Aliasing with history reprojection.
    Taa,
    /// TAA+ with enhanced velocity rejection and sharpening pass.
    TaaPlus,
}

// ---------------------------------------------------------------------------
// Pipeline state
// ---------------------------------------------------------------------------

/// Complete rasterisation state for a single render pass.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::{default_pipeline, BlendMode, CullMode};
///
/// let ps = default_pipeline();
/// assert_eq!(ps.blend_mode, BlendMode::Opaque);
/// assert_eq!(ps.cull_mode, CullMode::Back);
/// assert_eq!(ps.sample_count, 1);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct PipelineState {
    /// Source-to-destination blending equation.
    pub blend_mode: BlendMode,
    /// Depth-buffer comparison function.
    pub depth_func: DepthFunc,
    /// Whether to write the fragment depth into the depth buffer.
    pub depth_write: bool,
    /// Triangle face-culling mode.
    pub cull_mode: CullMode,
    /// Number of MSAA samples (1 = no MSAA).
    pub sample_count: u32,
}

// ---------------------------------------------------------------------------
// Quality settings
// ---------------------------------------------------------------------------

/// Rendering quality tier.
#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub enum QualityLevel {
    /// Minimum settings for low-end hardware.
    Low,
    /// Balanced settings suitable for most desktops.
    Medium,
    /// High-fidelity settings for modern discrete GPUs.
    High,
    /// Maximum quality; exhausts available GPU headroom.
    Ultra,
}

/// Per-quality rendering configuration returned by [`quality_settings`].
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::{quality_settings, QualityLevel};
///
/// let lo = quality_settings(QualityLevel::Low);
/// let hi = quality_settings(QualityLevel::Ultra);
/// assert!(lo.ao_samples < hi.ao_samples);
/// assert!(lo.shadow_map_size < hi.shadow_map_size);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct QualitySettings {
    /// Shadow map resolution in texels (one side of the square map).
    pub shadow_map_size: u32,
    /// Number of ambient-occlusion samples per pixel.
    pub ao_samples: u32,
    /// Anti-aliasing method selected at this quality tier.
    pub aa_method: AaMethod,
    /// Whether subsurface scattering is evaluated.
    pub sss_enabled: bool,
    /// Maximum number of dynamic lights evaluated per fragment.
    pub max_lights: u32,
}

// ---------------------------------------------------------------------------
// TAA types
// ---------------------------------------------------------------------------

/// Configuration for the Temporal Anti-Aliasing resolve pass.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::TaaConfig;
///
/// let cfg = TaaConfig::default();
/// assert_eq!(cfg.jitter_sequence_length, 16);
/// assert!((cfg.feedback_factor - 0.9).abs() < 1e-10);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaaConfig {
    /// Number of frames in the Halton jitter cycle.
    pub jitter_sequence_length: usize,
    /// History-blend weight `alpha` in `output = alpha * history + (1-alpha) * current`.
    pub feedback_factor: f64,
    /// Velocity length at which feedback is fully suppressed (ghost prevention).
    pub velocity_rejection: f64,
}

impl Default for TaaConfig {
    fn default() -> Self {
        Self {
            jitter_sequence_length: 16,
            feedback_factor: 0.9,
            velocity_rejection: 0.5,
        }
    }
}

/// Subpixel jitter offset in normalised screen coordinates `[-0.5, 0.5]`.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JitterSample {
    /// Horizontal offset.
    pub x: f64,
    /// Vertical offset.
    pub y: f64,
}

// ---------------------------------------------------------------------------
// GTAO types
// ---------------------------------------------------------------------------

/// Configuration for the Ground Truth Ambient Occlusion compute pass.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::GtaoConfig;
///
/// let cfg = GtaoConfig::default();
/// assert_eq!(cfg.num_directions, 4);
/// assert_eq!(cfg.num_steps, 4);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct GtaoConfig {
    /// Number of sampled azimuthal directions per pixel.
    pub num_directions: usize,
    /// Number of marching steps along each direction.
    pub num_steps: usize,
    /// World-space hemisphere radius for AO ray marching.
    pub radius: f64,
    /// Normalised distance at which falloff begins (fraction of `radius`).
    pub falloff_start: f64,
    /// Exponent controlling the steepness of the falloff curve.
    pub power: f64,
}

impl Default for GtaoConfig {
    fn default() -> Self {
        Self {
            num_directions: 4,
            num_steps: 4,
            radius: 0.5,
            falloff_start: 0.2,
            power: 2.0,
        }
    }
}

/// A single GTAO hemisphere sample: a 2D direction and a step-offset fraction.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AoSample {
    /// Unit direction vector in the XZ plane `[cos theta, sin theta]`.
    pub direction: [f64; 2],
    /// Fractional step offset along the ray `[0, 1]`.
    pub step_offset: f64,
}

// ---------------------------------------------------------------------------
// SSS types
// ---------------------------------------------------------------------------

/// Physical diffusion profile for a subsurface-scattering material.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SssProfile {
    /// Mean free path distance in world-space units.
    pub scatter_distance: f64,
    /// Spectral tint `[R, G, B]` of scattered light.
    pub scatter_color: [f64; 3],
    /// Pre-computed transmittance samples at discrete thicknesses.
    pub transmittance_samples: Vec<f64>,
}

/// Built-in SSS material presets plus an escape hatch for custom profiles.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum SssMaterial {
    /// Human skin with shallow red-channel scatter.
    Skin,
    /// Wax with wide uniform scatter and warm tint.
    Wax,
    /// Marble with fine white scatter.
    Marble,
    /// Jade with deep green-dominant scatter.
    Jade,
    /// Milk with very wide, nearly achromatic scatter.
    Milk,
    /// User-supplied diffusion profile.
    Custom(SssProfile),
}

/// One radial sample in the Christensen-Burley diffusion kernel.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DiffusionSample {
    /// Distance from the entry point in world-space units.
    pub radius: f64,
    /// Importance weight for this sample.
    pub weight: f64,
}

// ---------------------------------------------------------------------------
// Pipeline functions
// ---------------------------------------------------------------------------

/// Return a default opaque pipeline state.
///
/// The default state uses:
/// - [`BlendMode::Opaque`]
/// - [`DepthFunc::Less`] (depth test enabled, depth writes on)
/// - [`CullMode::Back`]
/// - `sample_count = 1` (no MSAA)
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::{default_pipeline, BlendMode, DepthFunc, CullMode};
///
/// let ps = default_pipeline();
/// assert_eq!(ps.blend_mode, BlendMode::Opaque);
/// assert_eq!(ps.depth_func, DepthFunc::Less);
/// assert!(ps.depth_write);
/// assert_eq!(ps.cull_mode, CullMode::Back);
/// assert_eq!(ps.sample_count, 1);
/// ```
#[must_use]
pub fn default_pipeline() -> PipelineState {
    PipelineState {
        blend_mode: BlendMode::Opaque,
        depth_func: DepthFunc::Less,
        depth_write: true,
        cull_mode: CullMode::Back,
        sample_count: 1,
    }
}

/// Return [`QualitySettings`] for a given [`QualityLevel`].
///
/// | Level  | Shadow | AO  | AA       | SSS | Lights |
/// |--------|--------|-----|----------|-----|--------|
/// | Low    | 512    | 4   | None     | off | 4      |
/// | Medium | 1024   | 8   | Msaa4x   | off | 8      |
/// | High   | 2048   | 16  | Taa      | on  | 16     |
/// | Ultra  | 4096   | 32  | TaaPlus  | on  | 32     |
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::{quality_settings, QualityLevel, AaMethod};
///
/// let s = quality_settings(QualityLevel::Ultra);
/// assert_eq!(s.shadow_map_size, 4096);
/// assert_eq!(s.ao_samples, 32);
/// assert_eq!(s.aa_method, AaMethod::TaaPlus);
/// assert!(s.sss_enabled);
/// assert_eq!(s.max_lights, 32);
/// ```
#[must_use]
pub fn quality_settings(level: QualityLevel) -> QualitySettings {
    match level {
        QualityLevel::Low => QualitySettings {
            shadow_map_size: 512,
            ao_samples: 4,
            aa_method: AaMethod::None,
            sss_enabled: false,
            max_lights: 4,
        },
        QualityLevel::Medium => QualitySettings {
            shadow_map_size: 1024,
            ao_samples: 8,
            aa_method: AaMethod::Msaa4x,
            sss_enabled: false,
            max_lights: 8,
        },
        QualityLevel::High => QualitySettings {
            shadow_map_size: 2048,
            ao_samples: 16,
            aa_method: AaMethod::Taa,
            sss_enabled: true,
            max_lights: 16,
        },
        QualityLevel::Ultra => QualitySettings {
            shadow_map_size: 4096,
            ao_samples: 32,
            aa_method: AaMethod::TaaPlus,
            sss_enabled: true,
            max_lights: 32,
        },
    }
}

/// Adaptively adjust quality based on measured vs. target FPS.
///
/// - If `current_fps < target_fps * 0.9` the level is lowered by one step.
/// - If `current_fps > target_fps * 1.1` the level is raised by one step.
/// - Otherwise the level is returned unchanged.
///
/// Levels are clamped: [`QualityLevel::Low`] cannot go lower and
/// [`QualityLevel::Ultra`] cannot go higher.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::{adaptive_quality, QualityLevel};
///
/// // FPS well below target — drop a level
/// assert_eq!(adaptive_quality(45.0, 60.0, QualityLevel::High), QualityLevel::Medium);
///
/// // FPS well above target — raise a level
/// assert_eq!(adaptive_quality(75.0, 60.0, QualityLevel::Medium), QualityLevel::High);
///
/// // Already at Ultra with headroom — stays at Ultra
/// assert_eq!(adaptive_quality(75.0, 60.0, QualityLevel::Ultra), QualityLevel::Ultra);
/// ```
#[must_use]
pub fn adaptive_quality(
    current_fps: f64,
    target_fps: f64,
    current_level: QualityLevel,
) -> QualityLevel {
    if current_fps < target_fps * 0.9 {
        match current_level {
            QualityLevel::Low => QualityLevel::Low,
            QualityLevel::Medium => QualityLevel::Low,
            QualityLevel::High => QualityLevel::Medium,
            QualityLevel::Ultra => QualityLevel::High,
        }
    } else if current_fps > target_fps * 1.1 {
        match current_level {
            QualityLevel::Low => QualityLevel::Medium,
            QualityLevel::Medium => QualityLevel::High,
            QualityLevel::High => QualityLevel::Ultra,
            QualityLevel::Ultra => QualityLevel::Ultra,
        }
    } else {
        current_level
    }
}

// ---------------------------------------------------------------------------
// WGSL shader generation
// ---------------------------------------------------------------------------

/// Generate a WGSL vertex shader for instanced sphere (atom) rendering.
///
/// The shader reads per-instance atom positions and radii from a storage
/// buffer and outputs clip-space positions for billboard quad rasterisation.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::generate_molecular_vertex_shader;
///
/// let src = generate_molecular_vertex_shader();
/// assert!(src.contains("fn "));
/// assert!(src.contains("@"));
/// assert!(src.contains("position"));
/// ```
#[must_use]
pub fn generate_molecular_vertex_shader() -> String {
    // nexcore-viz: Molecular Instanced Sphere Vertex Shader (WGSL)
    // Renders atom positions as instanced screen-space billboards.
    [
        "// nexcore-viz: Molecular Instanced Sphere Vertex Shader (WGSL)",
        "// Renders atom positions as instanced screen-space billboards.",
        "",
        "struct CameraUniforms {",
        "    view_proj: mat4x4<f32>,",
        "    view: mat4x4<f32>,",
        "    camera_pos: vec3<f32>,",
        "    _pad: f32,",
        "}",
        "",
        "struct AtomInstance {",
        "    position: vec3<f32>,",
        "    radius: f32,",
        "    color: vec4<f32>,",
        "}",
        "",
        "struct VertexOutput {",
        "    @builtin(position) clip_position: vec4<f32>,",
        "    @location(0) world_position: vec3<f32>,",
        "    @location(1) sphere_center: vec3<f32>,",
        "    @location(2) sphere_radius: f32,",
        "    @location(3) frag_color: vec4<f32>,",
        "    @location(4) uv: vec2<f32>,",
        "}",
        "",
        "@group(0) @binding(0) var<uniform> camera: CameraUniforms;",
        "@group(1) @binding(0) var<storage, read> atoms: array<AtomInstance>;",
        "",
        "const QUAD_CORNERS: array<vec2<f32>, 4> = array<vec2<f32>, 4>(",
        "    vec2<f32>(-1.0, -1.0),",
        "    vec2<f32>( 1.0, -1.0),",
        "    vec2<f32>(-1.0,  1.0),",
        "    vec2<f32>( 1.0,  1.0),",
        ");",
        "",
        "@vertex",
        "fn vs_main(",
        "    @builtin(vertex_index) vertex_index: u32,",
        "    @builtin(instance_index) instance_index: u32,",
        ") -> VertexOutput {",
        "    let atom   = atoms[instance_index];",
        "    let corner = QUAD_CORNERS[vertex_index % 4u];",
        "    let view_right = vec3<f32>(camera.view[0][0], camera.view[1][0], camera.view[2][0]);",
        "    let view_up    = vec3<f32>(camera.view[0][1], camera.view[1][1], camera.view[2][1]);",
        "    let world_pos  = atom.position",
        "        + (view_right * corner.x + view_up * corner.y) * atom.radius;",
        "    var out: VertexOutput;",
        "    out.clip_position  = camera.view_proj * vec4<f32>(world_pos, 1.0);",
        "    out.world_position = world_pos;",
        "    out.sphere_center  = atom.position;",
        "    out.sphere_radius  = atom.radius;",
        "    out.frag_color     = atom.color;",
        "    out.uv             = corner;",
        "    return out;",
        "}",
    ]
    .join("\n")
}

/// Generate a WGSL fragment shader for molecular sphere rendering.
///
/// Implements ray-sphere intersection and Blinn-Phong lighting to produce
/// per-fragment depth-correct shading of atom spheres.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::generate_molecular_fragment_shader;
///
/// let src = generate_molecular_fragment_shader();
/// assert!(src.contains("fn "));
/// assert!(src.contains("@"));
/// assert!(src.contains("color"));
/// ```
#[must_use]
pub fn generate_molecular_fragment_shader() -> String {
    [
        "// nexcore-viz: Molecular Sphere Fragment Shader (WGSL)",
        "// Ray-sphere intersection with Blinn-Phong lighting.",
        "",
        "struct LightUniforms {",
        "    direction: vec3<f32>,",
        "    intensity: f32,",
        "    ambient: vec3<f32>,",
        "    _pad: f32,",
        "}",
        "",
        "struct FragmentOutput {",
        "    @location(0) color: vec4<f32>,",
        "    @builtin(frag_depth) depth: f32,",
        "}",
        "",
        "@group(0) @binding(1) var<uniform> light: LightUniforms;",
        "",
        "@fragment",
        "fn fs_main(",
        "    @builtin(position) frag_coord: vec4<f32>,",
        "    @location(0) world_position: vec3<f32>,",
        "    @location(1) sphere_center: vec3<f32>,",
        "    @location(2) sphere_radius: f32,",
        "    @location(3) frag_color: vec4<f32>,",
        "    @location(4) uv: vec2<f32>,",
        ") -> FragmentOutput {",
        "    let r2 = dot(uv, uv);",
        "    if r2 > 1.0 { discard; }",
        "    let z      = sqrt(max(0.0, 1.0 - r2));",
        "    let normal = normalize(vec3<f32>(uv.x, uv.y, z));",
        "    let light_dir = normalize(-light.direction);",
        "    let diffuse   = max(dot(normal, light_dir), 0.0);",
        "    let half_vec  = normalize(light_dir + vec3<f32>(0.0, 0.0, 1.0));",
        "    let specular  = pow(max(dot(normal, half_vec), 0.0), 64.0);",
        "    let base_color = frag_color.rgb;",
        "    let lit_color  = light.ambient * base_color",
        "        + base_color * diffuse * light.intensity",
        "        + vec3<f32>(1.0) * specular * 0.3 * light.intensity;",
        "    let depth_out = frag_coord.z - (1.0 - z) * 0.001;",
        "    var out: FragmentOutput;",
        "    out.color = vec4<f32>(lit_color, frag_color.a);",
        "    out.depth = clamp(depth_out, 0.0, 1.0);",
        "    return out;",
        "}",
    ]
    .join("\n")
}

/// Generate a WGSL fragment shader for TAA history resolve.
///
/// Reprojects the previous frame accumulation buffer, applies neighbourhood
/// colour-clamp ghost rejection, and blends with the current frame using the
/// velocity-weighted feedback factor.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::generate_taa_resolve_shader;
///
/// let src = generate_taa_resolve_shader();
/// assert!(src.contains("fn "));
/// assert!(src.contains("@"));
/// ```
#[must_use]
pub fn generate_taa_resolve_shader() -> String {
    [
        "// nexcore-viz: TAA History Resolve Shader (WGSL)",
        "",
        "struct TaaUniforms {",
        "    feedback_factor: f32,",
        "    velocity_rejection_scale: f32,",
        "    inv_screen_size: vec2<f32>,",
        "    jitter_offset: vec2<f32>,",
        "    _pad: vec2<f32>,",
        "}",
        "",
        "@group(0) @binding(0) var<uniform> taa_params: TaaUniforms;",
        "@group(0) @binding(1) var current_frame:  texture_2d<f32>;",
        "@group(0) @binding(2) var history_frame:  texture_2d<f32>;",
        "@group(0) @binding(3) var velocity_buffer: texture_2d<f32>;",
        "@group(0) @binding(4) var linear_sampler: sampler;",
        "",
        "@fragment",
        "fn fs_taa_resolve(",
        "    @builtin(position) frag_coord: vec4<f32>,",
        "    @location(0) uv: vec2<f32>,",
        ") -> @location(0) vec4<f32> {",
        "    let pixel    = vec2<i32>(frag_coord.xy);",
        "    let current  = textureLoad(current_frame, pixel, 0);",
        "    let velocity = textureLoad(velocity_buffer, pixel, 0).xy;",
        "    let prev_uv  = uv - velocity;",
        "    var color_min = vec3<f32>(1.0);",
        "    var color_max = vec3<f32>(0.0);",
        "    for (var dy = -1; dy <= 1; dy++) {",
        "        for (var dx = -1; dx <= 1; dx++) {",
        "            let nb = textureLoad(current_frame, pixel + vec2<i32>(dx, dy), 0).rgb;",
        "            color_min = min(color_min, nb);",
        "            color_max = max(color_max, nb);",
        "        }",
        "    }",
        "    let history_raw     = textureSample(history_frame, linear_sampler, prev_uv).rgb;",
        "    let history_clamped = clamp(history_raw, color_min, color_max);",
        "    let velocity_len    = length(velocity);",
        "    let rejection       = 1.0 - smoothstep(0.0, taa_params.velocity_rejection_scale, velocity_len);",
        "    let alpha           = taa_params.feedback_factor * rejection;",
        "    let resolved        = mix(current.rgb, history_clamped, alpha);",
        "    return vec4<f32>(resolved, current.a);",
        "}",
    ]
    .join("\n")
}

/// Generate a WGSL compute shader for Ground Truth Ambient Occlusion.
///
/// Implements horizon-based GTAO: marches along pre-generated directions in
/// screen space and integrates visibility against the bent normal.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::generate_gtao_shader;
///
/// let src = generate_gtao_shader();
/// assert!(src.contains("fn "));
/// assert!(src.contains("@"));
/// ```
#[must_use]
pub fn generate_gtao_shader() -> String {
    [
        "// nexcore-viz: GTAO Compute Shader (WGSL)",
        "",
        "struct GtaoUniforms {",
        "    num_directions: u32,",
        "    num_steps: u32,",
        "    radius: f32,",
        "    falloff_start: f32,",
        "    power: f32,",
        "    inv_screen_size: vec2<f32>,",
        "    _pad: f32,",
        "    proj: mat4x4<f32>,",
        "    inv_proj: mat4x4<f32>,",
        "}",
        "",
        "@group(0) @binding(0) var<uniform> gtao_params: GtaoUniforms;",
        "@group(0) @binding(1) var depth_texture:  texture_2d<f32>;",
        "@group(0) @binding(2) var normal_texture: texture_2d<f32>;",
        "@group(0) @binding(3) var noise_texture:  texture_2d<f32>;",
        "@group(0) @binding(4) var output_ao: texture_storage_2d<r32float, write>;",
        "",
        "fn reconstruct_view_pos(uv: vec2<f32>, depth: f32) -> vec3<f32> {",
        "    let ndc    = vec4<f32>(uv * 2.0 - 1.0, depth, 1.0);",
        "    let view_h = gtao_params.inv_proj * ndc;",
        "    return view_h.xyz / view_h.w;",
        "}",
        "",
        "fn ao_falloff(dist: f32) -> f32 {",
        "    let t = smoothstep(gtao_params.falloff_start, 1.0, dist / gtao_params.radius);",
        "    return 1.0 - t;",
        "}",
        "",
        "@compute @workgroup_size(8, 8, 1)",
        "fn cs_gtao(@builtin(global_invocation_id) gid: vec3<u32>) {",
        "    let dims = textureDimensions(depth_texture);",
        "    if gid.x >= dims.x || gid.y >= dims.y { return; }",
        "    let pixel  = vec2<i32>(gid.xy);",
        "    let uv     = (vec2<f32>(gid.xy) + 0.5) * gtao_params.inv_screen_size;",
        "    let depth  = textureLoad(depth_texture, pixel, 0).r;",
        "    let normal = normalize(textureLoad(normal_texture, pixel, 0).xyz * 2.0 - 1.0);",
        "    let pos_v  = reconstruct_view_pos(uv, depth);",
        "    let noise_angle = textureLoad(noise_texture, vec2<i32>(gid.xy % 4u), 0).r * 6.28318;",
        "    var ao_accum = 0.0;",
        "    let step_size = gtao_params.radius / f32(gtao_params.num_steps);",
        "    for (var d = 0u; d < gtao_params.num_directions; d++) {",
        "        let angle = noise_angle + f32(d) * 6.28318 / f32(gtao_params.num_directions);",
        "        let dir2  = vec2<f32>(cos(angle), sin(angle));",
        "        var max_h = -1.0;",
        "        for (var s = 1u; s <= gtao_params.num_steps; s++) {",
        "            let step_uv  = uv + dir2 * (f32(s) * step_size) * gtao_params.inv_screen_size;",
        "            let step_d   = textureLoad(depth_texture, vec2<i32>(step_uv * vec2<f32>(dims)), 0).r;",
        "            let step_pos = reconstruct_view_pos(step_uv, step_d);",
        "            let horizon  = step_pos - pos_v;",
        "            let dist     = length(horizon);",
        "            if dist > 0.001 && dist < gtao_params.radius {",
        "                let cos_h = dot(normalize(horizon), normal);",
        "                max_h = max(max_h, cos_h * ao_falloff(dist));",
        "            }",
        "        }",
        "        ao_accum += 1.0 - max(0.0, max_h);",
        "    }",
        "    let ao = pow(ao_accum / f32(gtao_params.num_directions), gtao_params.power);",
        "    textureStore(output_ao, pixel, vec4<f32>(clamp(ao, 0.0, 1.0), 0.0, 0.0, 1.0));",
        "}",
    ]
    .join("\n")
}

/// Generate a WGSL fragment shader for subsurface scattering transmittance.
///
/// Evaluates the Christensen-Burley diffusion kernel for each colour channel
/// to produce a translucency term modulated by local material thickness.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::generate_sss_shader;
///
/// let src = generate_sss_shader();
/// assert!(src.contains("fn "));
/// assert!(src.contains("@"));
/// ```
#[must_use]
pub fn generate_sss_shader() -> String {
    [
        "// nexcore-viz: SSS Transmittance Fragment Shader (WGSL)",
        "",
        "struct SssUniforms {",
        "    scatter_distance: f32,",
        "    scatter_color: vec3<f32>,",
        "    thickness_scale: f32,",
        "    _pad: vec3<f32>,",
        "}",
        "",
        "@group(0) @binding(0) var<uniform> sss_params: SssUniforms;",
        "@group(0) @binding(1) var thickness_map:     texture_2d<f32>;",
        "@group(0) @binding(2) var thickness_sampler: sampler;",
        "",
        "fn burley_transmittance(thickness: f32, d: f32) -> f32 {",
        "    if d < 0.0001 { return 0.0; }",
        "    let t = thickness / d;",
        "    return 0.25 * (3.0 * exp(-t) + exp(-t / 3.0));",
        "}",
        "",
        "@fragment",
        "fn fs_sss(",
        "    @builtin(position) frag_coord: vec4<f32>,",
        "    @location(0) uv: vec2<f32>,",
        "    @location(1) base_color: vec4<f32>,",
        "    @location(2) light_color: vec3<f32>,",
        ") -> @location(0) vec4<f32> {",
        "    let thickness = textureSample(thickness_map, thickness_sampler, uv).r",
        "        * sss_params.thickness_scale;",
        "    let d_r = sss_params.scatter_distance * max(sss_params.scatter_color.r, 0.001);",
        "    let d_g = sss_params.scatter_distance * max(sss_params.scatter_color.g, 0.001);",
        "    let d_b = sss_params.scatter_distance * max(sss_params.scatter_color.b, 0.001);",
        "    let transmittance = vec3<f32>(",
        "        burley_transmittance(thickness, d_r),",
        "        burley_transmittance(thickness, d_g),",
        "        burley_transmittance(thickness, d_b),",
        "    );",
        "    let sss_color = base_color.rgb * transmittance * light_color;",
        "    return vec4<f32>(sss_color, base_color.a);",
        "}",
    ]
    .join("\n")
}

// ---------------------------------------------------------------------------
// TAA functions
// ---------------------------------------------------------------------------

/// Compute the `index`-th term of the Halton low-discrepancy sequence in
/// the given `base`.
///
/// The sequence is 1-indexed: `halton_sequence(2, 1) == 0.5`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::halton_sequence;
///
/// // Base-2 sequence: 1/2, 1/4, 3/4, …
/// assert!((halton_sequence(2, 1) - 0.5).abs() < 1e-10);
/// assert!((halton_sequence(2, 2) - 0.25).abs() < 1e-10);
/// assert!((halton_sequence(2, 3) - 0.75).abs() < 1e-10);
///
/// // Base-3 sequence: 1/3, 2/3, …
/// assert!((halton_sequence(3, 1) - 1.0 / 3.0).abs() < 1e-10);
/// assert!((halton_sequence(3, 2) - 2.0 / 3.0).abs() < 1e-10);
/// ```
#[must_use]
pub fn halton_sequence(base: u32, index: u32) -> f64 {
    let base_f = f64::from(base);
    let mut result = 0.0_f64;
    let mut denominator = 1.0_f64;
    let mut n = index;
    while n > 0 {
        denominator *= base_f;
        result += f64::from(n % base) / denominator;
        n /= base;
    }
    result
}

/// Generate the full subpixel jitter pattern for a TAA frame cycle.
///
/// Produces `config.jitter_sequence_length` samples using the Halton(2, 3)
/// sequence mapped to `[-0.5, 0.5]` in each axis.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::{TaaConfig, generate_jitter_pattern};
///
/// let cfg = TaaConfig { jitter_sequence_length: 8, ..TaaConfig::default() };
/// let samples = generate_jitter_pattern(&cfg);
/// assert_eq!(samples.len(), 8);
/// assert!(samples.iter().all(|s| s.x >= -0.5 && s.x <= 0.5));
/// assert!(samples.iter().all(|s| s.y >= -0.5 && s.y <= 0.5));
/// ```
#[must_use]
pub fn generate_jitter_pattern(config: &TaaConfig) -> Vec<JitterSample> {
    (1..=config.jitter_sequence_length)
        .map(|i| JitterSample {
            x: halton_sequence(2, i as u32) - 0.5,
            y: halton_sequence(3, i as u32) - 0.5,
        })
        .collect()
}

/// Compute the TAA history blend factor for a given motion-vector length.
///
/// - At zero velocity the full `feedback_factor` is returned.
/// - The factor decreases linearly to zero as `velocity_length` reaches
///   `config.velocity_rejection`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::{TaaConfig, taa_blend_factor};
///
/// let cfg = TaaConfig::default();
/// let full = taa_blend_factor(0.0, &cfg);
/// assert!((full - 0.9).abs() < 1e-10);
///
/// let reduced = taa_blend_factor(0.4, &cfg);
/// assert!(reduced < full);
/// ```
#[must_use]
pub fn taa_blend_factor(velocity_length: f64, config: &TaaConfig) -> f64 {
    if config.velocity_rejection <= 0.0 {
        return 0.0;
    }
    let t = (velocity_length / config.velocity_rejection).clamp(0.0, 1.0);
    config.feedback_factor * (1.0 - t)
}

// ---------------------------------------------------------------------------
// GTAO functions
// ---------------------------------------------------------------------------

/// Generate the hemisphere sample set for a GTAO pass.
///
/// Returns `num_directions * num_steps` samples with evenly-spaced azimuthal
/// directions and uniformly-distributed step offsets.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::{GtaoConfig, generate_ao_samples};
///
/// let cfg = GtaoConfig { num_directions: 4, num_steps: 4, ..GtaoConfig::default() };
/// let samples = generate_ao_samples(&cfg);
/// assert_eq!(samples.len(), 16);
/// ```
#[must_use]
pub fn generate_ao_samples(config: &GtaoConfig) -> Vec<AoSample> {
    use std::f64::consts::TAU;
    let mut samples = Vec::with_capacity(config.num_directions * config.num_steps);
    for d in 0..config.num_directions {
        let angle = (d as f64 / config.num_directions as f64) * TAU;
        let dir = [angle.cos(), angle.sin()];
        for s in 0..config.num_steps {
            let step_offset = if config.num_steps > 1 {
                s as f64 / (config.num_steps - 1) as f64
            } else {
                0.0
            };
            samples.push(AoSample {
                direction: dir,
                step_offset,
            });
        }
    }
    samples
}

/// Smooth falloff weight for a sample at `distance` from the shading point.
///
/// Returns `1.0` at zero distance and approaches `0.0` as distance nears
/// `config.radius`. The rolloff begins at `config.falloff_start * config.radius`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::{GtaoConfig, gtao_falloff};
///
/// let cfg = GtaoConfig::default();
/// assert!((gtao_falloff(0.0, &cfg) - 1.0).abs() < 1e-6);
/// assert!(gtao_falloff(cfg.radius * 10.0, &cfg) < 0.001);
/// ```
#[must_use]
pub fn gtao_falloff(distance: f64, config: &GtaoConfig) -> f64 {
    if config.radius <= 0.0 {
        return 0.0;
    }
    let t = distance / config.radius;
    if t <= config.falloff_start {
        return 1.0;
    }
    if t >= 1.0 {
        return 0.0;
    }
    let u = (t - config.falloff_start) / (1.0 - config.falloff_start);
    let smooth = u * u * (3.0 - 2.0 * u);
    (1.0 - smooth).powf(config.power)
}

/// Convert a cosine horizon angle to an AO occlusion value.
///
/// - `cos_horizon == 1.0` (horizon at zenith) produces no occlusion: `0.0`
/// - `cos_horizon == 0.0` (horizon at 90 degrees) produces full occlusion: `1.0`
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::horizon_angle_to_ao;
///
/// assert!((horizon_angle_to_ao(0.0) - 1.0).abs() < 1e-10);
/// assert!((horizon_angle_to_ao(1.0) - 0.0).abs() < 1e-10);
/// ```
#[must_use]
pub fn horizon_angle_to_ao(cos_horizon: f64) -> f64 {
    // (1 - cos) maps [0,1] -> [1,0]; multiply by 2 and keep in [0,1].
    // The factor of 2 comes from the half-angle area formula.
    let clamped = cos_horizon.clamp(0.0, 1.0);
    1.0 - clamped
}

// ---------------------------------------------------------------------------
// SSS functions
// ---------------------------------------------------------------------------

/// Generate the Christensen-Burley diffusion profile sample set for a material.
///
/// Samples the exponential-sum R(r) kernel at 16 radii from near-zero out to
/// `3 * scatter_distance`, weighted by the Burley PDF.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::{SssMaterial, sss_diffusion_profile};
///
/// let samples = sss_diffusion_profile(&SssMaterial::Skin);
/// assert!(!samples.is_empty());
/// assert!(samples.iter().all(|s| s.weight > 0.0));
/// ```
#[must_use]
pub fn sss_diffusion_profile(material: &SssMaterial) -> Vec<DiffusionSample> {
    let profile = preset_sss_profile(material);
    let d = profile.scatter_distance;
    let n = 16_usize;
    let max_r = d * 3.0;
    (1..=n)
        .map(|i| {
            let r = max_r * (i as f64 / n as f64);
            let weight = burley_diffusion_r(r, d);
            DiffusionSample { radius: r, weight }
        })
        .collect()
}

/// Evaluate the Christensen-Burley radial diffusion function R(r).
///
/// ```text
/// R(r) = (exp(-r/d) + exp(-r/(3d))) / (8 * pi * d * r)
/// ```
///
/// Returns `0.0` for non-positive inputs to avoid division by zero.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::burley_diffusion_r;
///
/// let v = burley_diffusion_r(0.1, 1.0);
/// assert!(v > 0.0);
///
/// // Function decreases with distance
/// assert!(burley_diffusion_r(0.1, 1.0) > burley_diffusion_r(1.0, 1.0));
/// ```
#[must_use]
pub fn burley_diffusion_r(r: f64, d: f64) -> f64 {
    use std::f64::consts::PI;
    if r <= 0.0 || d <= 0.0 {
        return 0.0;
    }
    let numerator = (-r / d).exp() + (-r / (3.0 * d)).exp();
    let denominator = 8.0 * PI * d * r;
    numerator / denominator
}

/// Compute RGB transmittance through a slab of the given `thickness`.
///
/// Uses a per-channel Burley integral: thicker slabs absorb more light and
/// the result approaches `[0, 0, 0]`. At zero thickness the result is `[1, 1, 1]`.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::{SssMaterial, preset_sss_profile, sss_transmittance};
///
/// let profile = preset_sss_profile(&SssMaterial::Skin);
/// let thin = sss_transmittance(0.0, &profile);
/// assert!((thin[0] - 1.0).abs() < 1e-6);
///
/// let thick = sss_transmittance(1000.0, &profile);
/// assert!(thick[0] < 0.001 && thick[1] < 0.001 && thick[2] < 0.001);
/// ```
#[must_use]
pub fn sss_transmittance(thickness: f64, profile: &SssProfile) -> [f64; 3] {
    if thickness <= 0.0 {
        return [1.0, 1.0, 1.0];
    }
    let d = profile.scatter_distance;
    let compute = |channel_scale: f64| -> f64 {
        let d_ch = d * channel_scale.max(1e-6);
        let t = thickness / d_ch;
        (0.25 * (3.0 * (-t).exp() + (-t / 3.0).exp())).clamp(0.0, 1.0)
    };
    [
        compute(profile.scatter_color[0]),
        compute(profile.scatter_color[1]),
        compute(profile.scatter_color[2]),
    ]
}

/// Return a preset [`SssProfile`] for the given material type.
///
/// For [`SssMaterial::Custom`] the embedded profile is cloned directly.
///
/// # Example
///
/// ```rust
/// use nexcore_viz::renderer::{SssMaterial, preset_sss_profile};
///
/// let skin = preset_sss_profile(&SssMaterial::Skin);
/// assert!(skin.scatter_distance > 0.0);
/// ```
#[must_use]
pub fn preset_sss_profile(material: &SssMaterial) -> SssProfile {
    match material {
        SssMaterial::Skin => SssProfile {
            scatter_distance: 1.5,
            scatter_color: [1.0, 0.4, 0.2],
            transmittance_samples: vec![1.0, 0.8, 0.6, 0.4, 0.2, 0.1],
        },
        SssMaterial::Wax => SssProfile {
            scatter_distance: 3.0,
            scatter_color: [0.9, 0.8, 0.6],
            transmittance_samples: vec![1.0, 0.7, 0.5, 0.3, 0.2, 0.1],
        },
        SssMaterial::Marble => SssProfile {
            scatter_distance: 0.8,
            scatter_color: [0.95, 0.93, 0.90],
            transmittance_samples: vec![1.0, 0.9, 0.7, 0.5, 0.3, 0.1],
        },
        SssMaterial::Jade => SssProfile {
            scatter_distance: 2.0,
            scatter_color: [0.2, 0.8, 0.3],
            transmittance_samples: vec![1.0, 0.75, 0.55, 0.35, 0.2, 0.05],
        },
        SssMaterial::Milk => SssProfile {
            scatter_distance: 5.0,
            scatter_color: [0.9, 0.88, 0.85],
            transmittance_samples: vec![1.0, 0.65, 0.45, 0.3, 0.15, 0.05],
        },
        SssMaterial::Custom(profile) => profile.clone(),
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    // --- Pipeline -----------------------------------------------------------

    #[test]
    fn test_default_pipeline_blend_mode() {
        assert_eq!(default_pipeline().blend_mode, BlendMode::Opaque);
    }

    #[test]
    fn test_default_pipeline_depth_func() {
        assert_eq!(default_pipeline().depth_func, DepthFunc::Less);
    }

    #[test]
    fn test_default_pipeline_depth_write() {
        assert!(default_pipeline().depth_write);
    }

    #[test]
    fn test_default_pipeline_cull_mode() {
        assert_eq!(default_pipeline().cull_mode, CullMode::Back);
    }

    #[test]
    fn test_default_pipeline_sample_count() {
        assert_eq!(default_pipeline().sample_count, 1);
    }

    #[test]
    fn test_quality_settings_low_vs_ultra_ao() {
        let lo = quality_settings(QualityLevel::Low);
        let ul = quality_settings(QualityLevel::Ultra);
        assert!(lo.ao_samples < ul.ao_samples);
    }

    #[test]
    fn test_quality_settings_low_values() {
        let s = quality_settings(QualityLevel::Low);
        assert_eq!(s.shadow_map_size, 512);
        assert_eq!(s.ao_samples, 4);
        assert!(!s.sss_enabled);
        assert_eq!(s.max_lights, 4);
    }

    #[test]
    fn test_quality_settings_ultra_values() {
        let s = quality_settings(QualityLevel::Ultra);
        assert_eq!(s.shadow_map_size, 4096);
        assert_eq!(s.ao_samples, 32);
        assert!(s.sss_enabled);
        assert_eq!(s.aa_method, AaMethod::TaaPlus);
    }

    #[test]
    fn test_adaptive_quality_drops_on_low_fps() {
        assert_eq!(
            adaptive_quality(45.0, 60.0, QualityLevel::High),
            QualityLevel::Medium
        );
    }

    #[test]
    fn test_adaptive_quality_raises_on_high_fps() {
        assert_eq!(
            adaptive_quality(75.0, 60.0, QualityLevel::Medium),
            QualityLevel::High
        );
    }

    #[test]
    fn test_adaptive_quality_stays_at_ultra() {
        assert_eq!(
            adaptive_quality(75.0, 60.0, QualityLevel::Ultra),
            QualityLevel::Ultra
        );
    }

    #[test]
    fn test_adaptive_quality_stays_at_low() {
        assert_eq!(
            adaptive_quality(30.0, 60.0, QualityLevel::Low),
            QualityLevel::Low
        );
    }

    #[test]
    fn test_adaptive_quality_no_change_within_tolerance() {
        assert_eq!(
            adaptive_quality(60.0, 60.0, QualityLevel::High),
            QualityLevel::High
        );
    }

    // --- Halton sequence ----------------------------------------------------

    #[test]
    fn test_halton_base2_index1() {
        assert!((halton_sequence(2, 1) - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_halton_base2_index2() {
        assert!((halton_sequence(2, 2) - 0.25).abs() < 1e-10);
    }

    #[test]
    fn test_halton_base2_index3() {
        assert!((halton_sequence(2, 3) - 0.75).abs() < 1e-10);
    }

    #[test]
    fn test_halton_base3_index1() {
        assert!((halton_sequence(3, 1) - 1.0 / 3.0).abs() < 1e-10);
    }

    #[test]
    fn test_halton_base3_index2() {
        assert!((halton_sequence(3, 2) - 2.0 / 3.0).abs() < 1e-10);
    }

    // --- TAA jitter ---------------------------------------------------------

    #[test]
    fn test_jitter_pattern_length_matches_config() {
        let cfg = TaaConfig {
            jitter_sequence_length: 8,
            ..TaaConfig::default()
        };
        assert_eq!(generate_jitter_pattern(&cfg).len(), 8);
    }

    #[test]
    fn test_jitter_pattern_default_length() {
        assert_eq!(generate_jitter_pattern(&TaaConfig::default()).len(), 16);
    }

    #[test]
    fn test_jitter_pattern_x_in_range() {
        let samples = generate_jitter_pattern(&TaaConfig::default());
        assert!(samples.iter().all(|s| s.x >= -0.5 && s.x <= 0.5));
    }

    #[test]
    fn test_jitter_pattern_y_in_range() {
        let samples = generate_jitter_pattern(&TaaConfig::default());
        assert!(samples.iter().all(|s| s.y >= -0.5 && s.y <= 0.5));
    }

    // --- TAA blend factor ---------------------------------------------------

    #[test]
    fn test_taa_blend_factor_zero_velocity() {
        let cfg = TaaConfig::default();
        assert!((taa_blend_factor(0.0, &cfg) - cfg.feedback_factor).abs() < 1e-10);
    }

    #[test]
    fn test_taa_blend_factor_high_velocity_lower() {
        let cfg = TaaConfig::default();
        assert!(taa_blend_factor(0.4, &cfg) < taa_blend_factor(0.0, &cfg));
    }

    #[test]
    fn test_taa_blend_factor_full_rejection() {
        let cfg = TaaConfig::default();
        assert!(taa_blend_factor(cfg.velocity_rejection, &cfg).abs() < 1e-10);
    }

    // --- GTAO samples -------------------------------------------------------

    #[test]
    fn test_ao_samples_count_4x4() {
        let cfg = GtaoConfig {
            num_directions: 4,
            num_steps: 4,
            ..GtaoConfig::default()
        };
        assert_eq!(generate_ao_samples(&cfg).len(), 16);
    }

    #[test]
    fn test_ao_samples_count_3x5() {
        let cfg = GtaoConfig {
            num_directions: 3,
            num_steps: 5,
            ..GtaoConfig::default()
        };
        assert_eq!(generate_ao_samples(&cfg).len(), 15);
    }

    // --- GTAO falloff -------------------------------------------------------

    #[test]
    fn test_gtao_falloff_at_zero() {
        let cfg = GtaoConfig::default();
        assert!((gtao_falloff(0.0, &cfg) - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_gtao_falloff_at_large_distance() {
        let cfg = GtaoConfig::default();
        assert!(gtao_falloff(cfg.radius * 10.0, &cfg) < 0.001);
    }

    #[test]
    fn test_gtao_falloff_decreasing() {
        let cfg = GtaoConfig::default();
        assert!(gtao_falloff(0.1, &cfg) >= gtao_falloff(0.4, &cfg));
    }

    // --- Horizon angle ------------------------------------------------------

    #[test]
    fn test_horizon_angle_cos_zero_max_occlusion() {
        assert!((horizon_angle_to_ao(0.0) - 1.0).abs() < 1e-10);
    }

    #[test]
    fn test_horizon_angle_cos_one_no_occlusion() {
        assert!((horizon_angle_to_ao(1.0) - 0.0).abs() < 1e-10);
    }

    #[test]
    fn test_horizon_angle_monotone() {
        assert!(horizon_angle_to_ao(0.3) > horizon_angle_to_ao(0.7));
    }

    // --- SSS diffusion profile ----------------------------------------------

    #[test]
    fn test_sss_diffusion_profile_skin_has_samples() {
        assert!(!sss_diffusion_profile(&SssMaterial::Skin).is_empty());
    }

    #[test]
    fn test_sss_diffusion_profile_positive_weights() {
        assert!(
            sss_diffusion_profile(&SssMaterial::Skin)
                .iter()
                .all(|s| s.weight > 0.0)
        );
    }

    // --- Burley R(r) --------------------------------------------------------

    #[test]
    fn test_burley_diffusion_r_positive() {
        assert!(burley_diffusion_r(0.1, 1.0) > 0.0);
    }

    #[test]
    fn test_burley_diffusion_r_decreases_with_distance() {
        assert!(burley_diffusion_r(0.1, 1.0) > burley_diffusion_r(1.0, 1.0));
    }

    #[test]
    fn test_burley_diffusion_r_zero_at_nonpositive() {
        assert_eq!(burley_diffusion_r(0.0, 1.0), 0.0);
        assert_eq!(burley_diffusion_r(-1.0, 1.0), 0.0);
    }

    // --- SSS transmittance --------------------------------------------------

    #[test]
    fn test_sss_transmittance_zero_thickness_is_one() {
        let p = preset_sss_profile(&SssMaterial::Skin);
        let t = sss_transmittance(0.0, &p);
        assert!((t[0] - 1.0).abs() < 1e-6);
        assert!((t[1] - 1.0).abs() < 1e-6);
        assert!((t[2] - 1.0).abs() < 1e-6);
    }

    #[test]
    fn test_sss_transmittance_large_thickness_near_zero() {
        let p = preset_sss_profile(&SssMaterial::Skin);
        let t = sss_transmittance(1000.0, &p);
        assert!(t[0] < 0.001);
        assert!(t[1] < 0.001);
        assert!(t[2] < 0.001);
    }

    #[test]
    fn test_sss_transmittance_decreasing_with_thickness() {
        let p = preset_sss_profile(&SssMaterial::Wax);
        let thin = sss_transmittance(0.5, &p);
        let thick = sss_transmittance(5.0, &p);
        assert!(thin[0] > thick[0]);
    }

    // --- Preset profiles ----------------------------------------------------

    #[test]
    fn test_preset_sss_profile_skin_nonzero_distance() {
        assert!(preset_sss_profile(&SssMaterial::Skin).scatter_distance > 0.0);
    }

    #[test]
    fn test_preset_sss_profile_custom_passthrough() {
        let custom = SssProfile {
            scatter_distance: 42.0,
            scatter_color: [1.0, 0.5, 0.0],
            transmittance_samples: vec![1.0, 0.5],
        };
        let p = preset_sss_profile(&SssMaterial::Custom(custom));
        assert!((p.scatter_distance - 42.0).abs() < 1e-10);
    }

    // --- WGSL shaders -------------------------------------------------------

    #[test]
    fn test_vert_shader_contains_fn() {
        assert!(generate_molecular_vertex_shader().contains("fn "));
    }

    #[test]
    fn test_vert_shader_contains_attribute() {
        assert!(generate_molecular_vertex_shader().contains('@'));
    }

    #[test]
    fn test_vert_shader_contains_position() {
        assert!(generate_molecular_vertex_shader().contains("position"));
    }

    #[test]
    fn test_frag_shader_contains_fn() {
        assert!(generate_molecular_fragment_shader().contains("fn "));
    }

    #[test]
    fn test_frag_shader_contains_attribute() {
        assert!(generate_molecular_fragment_shader().contains('@'));
    }

    #[test]
    fn test_frag_shader_contains_color() {
        assert!(generate_molecular_fragment_shader().contains("color"));
    }

    #[test]
    fn test_taa_shader_contains_fn() {
        assert!(generate_taa_resolve_shader().contains("fn "));
    }

    #[test]
    fn test_taa_shader_contains_attribute() {
        assert!(generate_taa_resolve_shader().contains('@'));
    }

    #[test]
    fn test_gtao_shader_contains_fn() {
        assert!(generate_gtao_shader().contains("fn "));
    }

    #[test]
    fn test_gtao_shader_contains_attribute() {
        assert!(generate_gtao_shader().contains('@'));
    }

    #[test]
    fn test_sss_shader_contains_fn() {
        assert!(generate_sss_shader().contains("fn "));
    }

    #[test]
    fn test_sss_shader_contains_attribute() {
        assert!(generate_sss_shader().contains('@'));
    }

    // --- Error display ------------------------------------------------------

    #[test]
    fn test_error_invalid_config_display() {
        let e = RendererError::InvalidConfig("bad blend".to_owned());
        let s = e.to_string();
        assert!(s.contains("Invalid renderer config"));
        assert!(s.contains("bad blend"));
    }

    #[test]
    fn test_error_shader_generation_failed_display() {
        let e = RendererError::ShaderGenerationFailed("missing binding".to_owned());
        let s = e.to_string();
        assert!(s.contains("Shader generation failed"));
        assert!(s.contains("missing binding"));
    }

    #[test]
    fn test_error_invalid_sample_count_display() {
        assert!(
            RendererError::InvalidSampleCount
                .to_string()
                .contains("Invalid sample count")
        );
    }

    // --- Serde roundtrip ----------------------------------------------------

    #[test]
    fn test_pipeline_state_serde_roundtrip() {
        let original = default_pipeline();
        let json = serde_json::to_string(&original).unwrap_or_default();
        assert!(!json.is_empty());
        let decoded: Result<PipelineState, _> = serde_json::from_str(&json);
        assert!(decoded.is_ok());
        let ps = decoded.unwrap_or_else(|_| default_pipeline());
        assert_eq!(ps.sample_count, 1);
        assert_eq!(ps.blend_mode, BlendMode::Opaque);
    }

    // --- Default values -----------------------------------------------------

    #[test]
    fn test_taa_config_default_jitter_length() {
        assert_eq!(TaaConfig::default().jitter_sequence_length, 16);
    }

    #[test]
    fn test_taa_config_default_feedback_factor() {
        assert!((TaaConfig::default().feedback_factor - 0.9).abs() < 1e-10);
    }

    #[test]
    fn test_taa_config_default_velocity_rejection() {
        assert!((TaaConfig::default().velocity_rejection - 0.5).abs() < 1e-10);
    }

    #[test]
    fn test_gtao_config_default_num_directions() {
        assert_eq!(GtaoConfig::default().num_directions, 4);
    }

    #[test]
    fn test_gtao_config_default_num_steps() {
        assert_eq!(GtaoConfig::default().num_steps, 4);
    }

    #[test]
    fn test_gtao_config_default_radius() {
        assert!((GtaoConfig::default().radius - 0.5).abs() < 1e-10);
    }
}
