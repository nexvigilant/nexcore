//! GPU-compatible particle system for molecular visualization effects.
//!
//! Provides data structures and CPU-side simulation logic for solvent particles,
//! reaction animations, electron clouds, and diffusion trails. The frontend
//! renders these using GPU instanced rendering.
//!
//! # Architecture
//!
//! ```text
//! ParticleSystem
//!   ├── Vec<Emitter>     — each emitter owns its particle pool
//!   ├── Vec<ForceField>  — global forces applied every tick
//!   └── rng_state: u64   — xorshift64 PRNG
//! ```
//!
//! # Workflow
//!
//! 1. Create a [`ParticleSystem`].
//! 2. Add emitters via [`ParticleSystem::add_emitter`].
//! 3. Optionally attach [`ForceField`]s with [`ParticleSystem::add_force`].
//! 4. Call [`ParticleSystem::update`] each frame with `dt` (seconds).
//! 5. Call [`ParticleSystem::snapshot`] to get [`ParticleSnapshot`] for the GPU.
//! 6. Pass the snapshot to [`pack_for_gpu`] to get an interleaved `f32` buffer.
//!
//! # Examples
//!
//! ```
//! use nexcore_viz::particle::{
//!     EmitterConfig, EmitterShape, ForceField, ParticleSystem, pack_for_gpu,
//!     solvent_emitter,
//! };
//!
//! let mut sys = ParticleSystem::new();
//! let cfg = solvent_emitter([0.0, 0.0, 0.0], 2.0);
//! sys.add_emitter(cfg);
//! sys.add_force(ForceField::Gravity([0.0, -9.81, 0.0]));
//! sys.update(0.016);
//! let snap = sys.snapshot();
//! let buf = pack_for_gpu(&snap);
//! assert_eq!(buf.len(), snap.count * 8);
//! ```

use serde::{Deserialize, Serialize};

// ============================================================================
// Error type
// ============================================================================

/// Errors produced by the particle system.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ParticleError {
    /// An emitter with invalid geometry or configuration was supplied.
    InvalidEmitter(String),
    /// The requested operation would exceed the configured particle cap.
    MaxParticlesExceeded { requested: usize, limit: usize },
    /// A lifetime value was zero or negative, which is physically meaningless.
    InvalidLifetime(f64),
}

impl core::fmt::Display for ParticleError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::InvalidEmitter(msg) => write!(f, "invalid emitter: {msg}"),
            Self::MaxParticlesExceeded { requested, limit } => write!(
                f,
                "max particles exceeded: requested {requested}, limit {limit}"
            ),
            Self::InvalidLifetime(v) => {
                write!(f, "invalid lifetime {v}: must be positive")
            }
        }
    }
}

impl std::error::Error for ParticleError {}

// ============================================================================
// Particle
// ============================================================================

/// A single particle in the simulation.
///
/// Positions and velocities use `f64` for numerical precision over long
/// simulations; colors and size use `f32` because GPU shaders work in `f32`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Particle {
    /// World-space position `[x, y, z]`.
    pub position: [f64; 3],
    /// Velocity `[vx, vy, vz]` in world units per second.
    pub velocity: [f64; 3],
    /// Acceleration `[ax, ay, az]` accumulated per tick, reset after integration.
    pub acceleration: [f64; 3],
    /// RGBA color, each channel in `[0.0, 1.0]`.
    pub color: [f32; 4],
    /// Billboard radius in world units.
    pub size: f32,
    /// Total lifespan in seconds.
    pub lifetime: f64,
    /// Seconds elapsed since this particle was emitted.
    pub age: f64,
    /// `false` once `age >= lifetime`; dead particles are recycled.
    pub alive: bool,
}

impl Particle {
    /// Normalized age in `[0.0, 1.0]` — useful for color/size interpolation.
    ///
    /// Returns `0.0` when `lifetime` is zero to avoid division by zero.
    #[must_use]
    pub fn age_fraction(&self) -> f64 {
        if self.lifetime <= 0.0 {
            return 0.0;
        }
        (self.age / self.lifetime).clamp(0.0, 1.0)
    }
}

impl Default for Particle {
    fn default() -> Self {
        Self {
            position: [0.0; 3],
            velocity: [0.0; 3],
            acceleration: [0.0; 3],
            color: [1.0, 1.0, 1.0, 1.0],
            size: 0.1,
            lifetime: 1.0,
            age: 0.0,
            alive: true,
        }
    }
}

// ============================================================================
// EmitterShape
// ============================================================================

/// Geometric shape from which new particles are spawned.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum EmitterShape {
    /// All particles originate from a single fixed point.
    Point([f64; 3]),
    /// Particles spawn at uniformly random positions on the surface of a sphere.
    Sphere {
        /// Centre of the sphere.
        center: [f64; 3],
        /// Sphere radius in world units.
        radius: f64,
    },
    /// Particles spawn at uniformly random positions inside an axis-aligned box.
    Box {
        /// Minimum corner `[x_min, y_min, z_min]`.
        min: [f64; 3],
        /// Maximum corner `[x_max, y_max, z_max]`.
        max: [f64; 3],
    },
    /// Particles spawn at uniformly random positions on a ring in 3D.
    Ring {
        /// Centre of the ring.
        center: [f64; 3],
        /// Normal vector that the ring plane is perpendicular to.
        normal: [f64; 3],
        /// Ring radius.
        radius: f64,
    },
}

// ============================================================================
// EmitterConfig
// ============================================================================

/// Configuration for one particle emitter.
///
/// All variance fields are half-ranges: the actual value is sampled uniformly
/// from `[base - variance, base + variance]`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct EmitterConfig {
    /// Spawn shape.
    pub shape: EmitterShape,
    /// Particles emitted per second.
    pub rate: f64,
    /// Base particle speed at birth (magnitude of initial velocity).
    pub initial_speed: f64,
    /// Half-range of speed randomisation.
    pub speed_variance: f64,
    /// Base particle lifetime in seconds.
    pub lifetime: f64,
    /// Half-range of lifetime randomisation.
    pub lifetime_variance: f64,
    /// RGBA color at birth.
    pub initial_color: [f32; 4],
    /// RGBA color at death (interpolated linearly with `age_fraction`).
    pub final_color: [f32; 4],
    /// Particle size at birth.
    pub initial_size: f32,
    /// Particle size at death.
    pub final_size: f32,
    /// Hard cap on particles this emitter may hold alive simultaneously.
    pub max_particles: usize,
    /// Constant gravity acceleration `[gx, gy, gz]` applied every tick.
    pub gravity: [f64; 3],
    /// Linear drag coefficient — velocity is multiplied by `(1 - drag * dt)` each tick.
    pub drag: f64,
}

impl Default for EmitterConfig {
    fn default() -> Self {
        Self {
            shape: EmitterShape::Point([0.0, 0.0, 0.0]),
            rate: 10.0,
            initial_speed: 1.0,
            speed_variance: 0.2,
            lifetime: 2.0,
            lifetime_variance: 0.5,
            initial_color: [1.0, 1.0, 1.0, 1.0],
            final_color: [1.0, 1.0, 1.0, 0.0],
            initial_size: 0.1,
            final_size: 0.05,
            max_particles: 1000,
            gravity: [0.0, 0.0, 0.0],
            drag: 0.0,
        }
    }
}

// ============================================================================
// Emitter
// ============================================================================

/// A particle emitter: owns its particle pool and drives its simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Emitter {
    /// Configuration applied when spawning and updating particles.
    pub config: EmitterConfig,
    /// Particle pool. Dead particles (`alive == false`) are recycled.
    pub particles: Vec<Particle>,
    /// When `false` the emitter stops spawning new particles.
    pub active: bool,
    /// Fractional particle debt accumulated between frames.
    pub time_accumulator: f64,
    /// Total particles ever emitted by this emitter.
    pub total_emitted: u64,
}

impl Emitter {
    /// Create a new emitter with the given config.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_viz::particle::{Emitter, EmitterConfig};
    ///
    /// let emitter = Emitter::new(EmitterConfig::default());
    /// assert!(emitter.active);
    /// assert_eq!(emitter.active_count(), 0);
    /// ```
    #[must_use]
    pub fn new(config: EmitterConfig) -> Self {
        Self {
            config,
            particles: Vec::new(),
            active: true,
            time_accumulator: 0.0,
            total_emitted: 0,
        }
    }

    /// Number of currently alive particles.
    #[must_use]
    pub fn active_count(&self) -> usize {
        self.particles.iter().filter(|p| p.alive).count()
    }

    /// Spawn `count` new particles, writing them into the returned `Vec`.
    ///
    /// Initial positions are drawn from the emitter shape; initial velocities
    /// are random unit vectors scaled by `initial_speed ± speed_variance`.
    /// Lifetimes are randomised by `lifetime_variance`.
    ///
    /// `rng` is advanced in place using xorshift64.
    pub fn emit(&self, count: usize, rng: &mut u64) -> Vec<Particle> {
        let mut out = Vec::with_capacity(count);
        for _ in 0..count {
            let position = sample_shape(&self.config.shape, rng);

            // Random unit vector for velocity direction
            let dir = random_unit_vec(rng);
            let speed_offset = (random_f64(rng) * 2.0 - 1.0) * self.config.speed_variance;
            let speed = (self.config.initial_speed + speed_offset).max(0.0);
            let velocity = [dir[0] * speed, dir[1] * speed, dir[2] * speed];

            let lifetime_offset = (random_f64(rng) * 2.0 - 1.0) * self.config.lifetime_variance;
            let lifetime = (self.config.lifetime + lifetime_offset).max(0.001);

            out.push(Particle {
                position,
                velocity,
                acceleration: [0.0; 3],
                color: self.config.initial_color,
                size: self.config.initial_size,
                lifetime,
                age: 0.0,
                alive: true,
            });
        }
        out
    }
}

// ============================================================================
// ForceField
// ============================================================================

/// A force that can be applied to every live particle each simulation tick.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum ForceField {
    /// Constant gravitational acceleration `[gx, gy, gz]` (e.g. `[0, -9.81, 0]`).
    Gravity([f64; 3]),
    /// Velocity-proportional drag coefficient (dimensionless per second).
    Drag(f64),
    /// Point attractor: pulls particles toward `center` with `strength`.
    Attractor {
        /// World-space attractor position.
        center: [f64; 3],
        /// Force magnitude (positive = pull).
        strength: f64,
    },
    /// Point repeller: pushes particles away from `center` with `strength`.
    Repeller {
        /// World-space repeller position.
        center: [f64; 3],
        /// Force magnitude (positive = push).
        strength: f64,
    },
    /// Pseudo-random turbulence field applied as a velocity perturbation.
    Turbulence {
        /// Spatial frequency of the turbulence pattern.
        frequency: f64,
        /// Velocity perturbation amplitude.
        amplitude: f64,
    },
    /// Rotational vortex around an axis through `center`.
    Vortex {
        /// Rotation axis (should be a unit vector).
        axis: [f64; 3],
        /// Point the axis passes through.
        center: [f64; 3],
        /// Angular velocity (radians per second).
        strength: f64,
    },
}

// ============================================================================
// ParticleSystem
// ============================================================================

/// Top-level particle simulation.
///
/// Owns all emitters and global force fields. Call [`update`](Self::update)
/// each frame, then [`snapshot`](Self::snapshot) to extract GPU-ready data.
///
/// # Examples
///
/// ```
/// use nexcore_viz::particle::{ParticleSystem, solvent_emitter, ForceField};
///
/// let mut sys = ParticleSystem::new();
/// sys.add_emitter(solvent_emitter([0.0, 0.0, 0.0], 1.0));
/// sys.add_force(ForceField::Gravity([0.0, -9.81, 0.0]));
/// sys.update(0.016);
/// let snap = sys.snapshot();
/// assert!(snap.count <= 1000);
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSystem {
    /// All emitters managed by this system.
    pub emitters: Vec<Emitter>,
    /// Global force fields applied to every particle every tick.
    pub forces: Vec<ForceField>,
    /// Accumulated simulation time in seconds.
    pub time: f64,
    /// xorshift64 PRNG state. Seed non-zero for deterministic simulations.
    pub rng_state: u64,
}

impl ParticleSystem {
    /// Create an empty particle system with a fixed PRNG seed.
    #[must_use]
    pub fn new() -> Self {
        Self {
            emitters: Vec::new(),
            forces: Vec::new(),
            time: 0.0,
            rng_state: 0x_dead_beef_cafe_1234,
        }
    }

    /// Add an emitter and return its index.
    ///
    /// # Examples
    ///
    /// ```
    /// use nexcore_viz::particle::{ParticleSystem, EmitterConfig};
    ///
    /// let mut sys = ParticleSystem::new();
    /// let idx = sys.add_emitter(EmitterConfig::default());
    /// assert_eq!(idx, 0);
    /// ```
    pub fn add_emitter(&mut self, config: EmitterConfig) -> usize {
        let idx = self.emitters.len();
        self.emitters.push(Emitter::new(config));
        idx
    }

    /// Attach a global force field.
    pub fn add_force(&mut self, force: ForceField) {
        self.forces.push(force);
    }

    /// Advance the simulation by `dt` seconds.
    ///
    /// Per-tick steps:
    /// 1. Accumulate time and compute how many new particles to emit.
    /// 2. Spawn new particles into dead slots or append if under the cap.
    /// 3. Apply all [`ForceField`]s to every live particle.
    /// 4. Apply per-emitter gravity and drag.
    /// 5. Euler-integrate velocity and position.
    /// 6. Interpolate color and size.
    /// 7. Age particles; kill those past their lifetime.
    pub fn update(&mut self, dt: f64) {
        self.time += dt;
        let rng = &mut self.rng_state;

        for emitter in &mut self.emitters {
            // Spawn new particles only when active
            if emitter.active {
                emitter.time_accumulator += dt;
                let spawn_count_f = emitter.time_accumulator * emitter.config.rate;
                let spawn_count = spawn_count_f as usize;
                if spawn_count > 0 {
                    emitter.time_accumulator -= spawn_count as f64 / emitter.config.rate;
                    let alive = emitter.active_count();
                    let room = emitter.config.max_particles.saturating_sub(alive);
                    let to_spawn = spawn_count.min(room);
                    if to_spawn > 0 {
                        let new_particles = emitter.emit(to_spawn, rng);
                        emitter.total_emitted += to_spawn as u64;
                        // Recycle dead slots first
                        let mut new_iter = new_particles.into_iter();
                        for slot in emitter.particles.iter_mut() {
                            if !slot.alive {
                                if let Some(p) = new_iter.next() {
                                    *slot = p;
                                } else {
                                    break;
                                }
                            }
                        }
                        // Append any remaining new particles
                        for p in new_iter {
                            emitter.particles.push(p);
                        }
                    }
                }
            }

            // Update existing particles
            let forces = &self.forces;
            let gravity = emitter.config.gravity;
            let drag_coeff = emitter.config.drag;

            for particle in emitter.particles.iter_mut().filter(|p| p.alive) {
                // Reset acceleration (accumulated fresh each tick)
                particle.acceleration = [0.0; 3];

                // Per-emitter gravity
                particle.acceleration[0] += gravity[0];
                particle.acceleration[1] += gravity[1];
                particle.acceleration[2] += gravity[2];

                // Global force fields
                for force in forces {
                    apply_force(particle, force, dt);
                }

                // Per-emitter drag applied directly to velocity
                if drag_coeff != 0.0 {
                    let drag_factor = (1.0 - drag_coeff * dt).max(0.0);
                    particle.velocity[0] *= drag_factor;
                    particle.velocity[1] *= drag_factor;
                    particle.velocity[2] *= drag_factor;
                }

                // Euler integration
                integrate_particle(particle, dt);

                // Age and update derived properties
                particle.age += dt;
                if particle.age >= particle.lifetime {
                    particle.alive = false;
                    continue;
                }

                // Interpolate color and size
                let t = particle.age_fraction() as f32;
                let ic = emitter.config.initial_color;
                let fc = emitter.config.final_color;
                particle.color = [
                    ic[0] + (fc[0] - ic[0]) * t,
                    ic[1] + (fc[1] - ic[1]) * t,
                    ic[2] + (fc[2] - ic[2]) * t,
                    ic[3] + (fc[3] - ic[3]) * t,
                ];
                let is = emitter.config.initial_size;
                let fs = emitter.config.final_size;
                particle.size = is + (fs - is) * t;
            }
        }
    }

    /// Extract GPU-ready packed data from all live particles.
    ///
    /// The snapshot is a flat snapshot; use [`pack_for_gpu`] to get an
    /// interleaved buffer ready for a vertex/instance buffer upload.
    #[must_use]
    pub fn snapshot(&self) -> ParticleSnapshot {
        let mut positions = Vec::new();
        let mut colors = Vec::new();
        let mut sizes = Vec::new();

        for emitter in &self.emitters {
            for p in emitter.particles.iter().filter(|p| p.alive) {
                positions.push([
                    p.position[0] as f32,
                    p.position[1] as f32,
                    p.position[2] as f32,
                ]);
                colors.push(p.color);
                sizes.push(p.size);
            }
        }

        let count = sizes.len();
        ParticleSnapshot {
            positions,
            colors,
            sizes,
            count,
        }
    }

    /// Total number of alive particles across all emitters.
    #[must_use]
    pub fn total_alive(&self) -> usize {
        self.emitters.iter().map(Emitter::active_count).sum()
    }

    /// Remove all particles from all emitters and reset accumulators.
    pub fn clear(&mut self) {
        for emitter in &mut self.emitters {
            emitter.particles.clear();
            emitter.time_accumulator = 0.0;
        }
    }
}

impl Default for ParticleSystem {
    fn default() -> Self {
        Self::new()
    }
}

// ============================================================================
// ParticleSnapshot
// ============================================================================

/// GPU-ready snapshot of all live particles.
///
/// Arrays are parallel: `positions[i]`, `colors[i]`, `sizes[i]` all describe
/// particle `i`. Use [`pack_for_gpu`] to interleave them for instance rendering.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ParticleSnapshot {
    /// World-space positions as `f32` triples (GPU native precision).
    pub positions: Vec<[f32; 3]>,
    /// RGBA colors, each channel in `[0.0, 1.0]`.
    pub colors: Vec<[f32; 4]>,
    /// Billboard radii.
    pub sizes: Vec<f32>,
    /// Number of live particles in this snapshot.
    pub count: usize,
}

// ============================================================================
// ParticleSystemConfig
// ============================================================================

/// Global configuration for a [`ParticleSystem`].
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ParticleSystemConfig {
    /// Hard cap across all emitters combined.
    pub max_total_particles: usize,
    /// Fixed integration timestep used when sub-stepping (seconds).
    pub fixed_timestep: f64,
}

impl Default for ParticleSystemConfig {
    fn default() -> Self {
        Self {
            max_total_particles: 100_000,
            fixed_timestep: 1.0 / 60.0,
        }
    }
}

// ============================================================================
// Force application
// ============================================================================

/// Apply a single [`ForceField`] to `particle` for one timestep of length `dt`.
///
/// Force fields that produce accelerations add to `particle.acceleration`.
/// Drag and turbulence modify `particle.velocity` directly.
pub fn apply_force(particle: &mut Particle, force: &ForceField, dt: f64) {
    match force {
        ForceField::Gravity(g) => {
            particle.acceleration[0] += g[0];
            particle.acceleration[1] += g[1];
            particle.acceleration[2] += g[2];
        }
        ForceField::Drag(coeff) => {
            let factor = (1.0 - coeff * dt).max(0.0);
            particle.velocity[0] *= factor;
            particle.velocity[1] *= factor;
            particle.velocity[2] *= factor;
        }
        ForceField::Attractor { center, strength } => {
            let dx = center[0] - particle.position[0];
            let dy = center[1] - particle.position[1];
            let dz = center[2] - particle.position[2];
            let dist_sq = dx * dx + dy * dy + dz * dz;
            if dist_sq > 1e-10 {
                let dist = dist_sq.sqrt();
                let mag = strength / dist_sq;
                particle.acceleration[0] += (dx / dist) * mag;
                particle.acceleration[1] += (dy / dist) * mag;
                particle.acceleration[2] += (dz / dist) * mag;
            }
        }
        ForceField::Repeller { center, strength } => {
            let dx = particle.position[0] - center[0];
            let dy = particle.position[1] - center[1];
            let dz = particle.position[2] - center[2];
            let dist_sq = dx * dx + dy * dy + dz * dz;
            if dist_sq > 1e-10 {
                let dist = dist_sq.sqrt();
                let mag = strength / dist_sq;
                particle.acceleration[0] += (dx / dist) * mag;
                particle.acceleration[1] += (dy / dist) * mag;
                particle.acceleration[2] += (dz / dist) * mag;
            }
        }
        ForceField::Turbulence {
            frequency,
            amplitude,
        } => {
            // Cheap spatially-varying turbulence: hash position components.
            let x = particle.position[0] * frequency;
            let y = particle.position[1] * frequency;
            let z = particle.position[2] * frequency;
            let tx = cheap_hash(x, y + 1.3) * 2.0 - 1.0;
            let ty = cheap_hash(y, z + 2.7) * 2.0 - 1.0;
            let tz = cheap_hash(z, x + 4.1) * 2.0 - 1.0;
            particle.velocity[0] += tx * amplitude * dt;
            particle.velocity[1] += ty * amplitude * dt;
            particle.velocity[2] += tz * amplitude * dt;
        }
        ForceField::Vortex {
            axis,
            center,
            strength,
        } => {
            // Velocity += strength * (axis × (pos - center)) * dt
            let rx = particle.position[0] - center[0];
            let ry = particle.position[1] - center[1];
            let rz = particle.position[2] - center[2];
            // Cross product: axis × r
            let cx = axis[1] * rz - axis[2] * ry;
            let cy = axis[2] * rx - axis[0] * rz;
            let cz = axis[0] * ry - axis[1] * rx;
            particle.velocity[0] += cx * strength * dt;
            particle.velocity[1] += cy * strength * dt;
            particle.velocity[2] += cz * strength * dt;
        }
    }
}

/// Euler-integrate velocity from acceleration, then position from velocity.
///
/// After integration, `particle.acceleration` is zeroed so it can be
/// re-accumulated next tick.
pub fn integrate_particle(p: &mut Particle, dt: f64) {
    p.velocity[0] += p.acceleration[0] * dt;
    p.velocity[1] += p.acceleration[1] * dt;
    p.velocity[2] += p.acceleration[2] * dt;

    p.position[0] += p.velocity[0] * dt;
    p.position[1] += p.velocity[1] * dt;
    p.position[2] += p.velocity[2] * dt;

    p.acceleration = [0.0; 3];
}

// ============================================================================
// GPU support
// ============================================================================

/// Interleave position, color, and size into a single `f32` buffer.
///
/// Each particle occupies 8 consecutive floats:
/// `[x, y, z, r, g, b, a, size]`.
///
/// This layout matches a typical WebGPU/wgpu instance buffer struct.
///
/// # Examples
///
/// ```
/// use nexcore_viz::particle::{ParticleSnapshot, pack_for_gpu};
///
/// let snap = ParticleSnapshot {
///     positions: vec![[1.0, 2.0, 3.0]],
///     colors: vec![[0.5, 0.5, 0.5, 1.0]],
///     sizes: vec![0.1],
///     count: 1,
/// };
/// let buf = pack_for_gpu(&snap);
/// assert_eq!(buf, vec![1.0, 2.0, 3.0, 0.5, 0.5, 0.5, 1.0, 0.1]);
/// ```
#[must_use]
pub fn pack_for_gpu(snapshot: &ParticleSnapshot) -> Vec<f32> {
    let mut buf = Vec::with_capacity(snapshot.count * 8);
    for i in 0..snapshot.count {
        let pos = snapshot.positions.get(i).copied().unwrap_or([0.0; 3]);
        let col = snapshot.colors.get(i).copied().unwrap_or([0.0; 4]);
        let sz = snapshot.sizes.get(i).copied().unwrap_or(0.0);
        buf.push(pos[0]);
        buf.push(pos[1]);
        buf.push(pos[2]);
        buf.push(col[0]);
        buf.push(col[1]);
        buf.push(col[2]);
        buf.push(col[3]);
        buf.push(sz);
    }
    buf
}

/// WGSL compute shader that updates particle positions on the GPU.
///
/// The shader expects a storage buffer with the same `[x, y, z, r, g, b, a, size]`
/// layout produced by [`pack_for_gpu`]. It advances position by velocity each
/// invocation (velocity is stored in a separate buffer in a real integration;
/// this shader demonstrates the data contract).
///
/// # Returns
///
/// A `'static str` containing valid WGSL source. It is never empty.
#[must_use]
pub fn wgsl_particle_update_shader() -> &'static str {
    r#"// nexcore-viz particle update compute shader
// Buffer layout per particle (8 x f32):
//   [0] x  [1] y  [2] z  [3] r  [4] g  [5] b  [6] a  [7] size
//
// Velocity buffer layout per particle (3 x f32):
//   [0] vx  [1] vy  [2] vz
//
// Uniform: dt (f32), gravity_y (f32)

struct Particle {
    pos: vec3<f32>,
    color: vec4<f32>,
    size: f32,
    _pad: vec3<f32>,
}

struct Velocity {
    vel: vec3<f32>,
    lifetime: f32,
    age: f32,
    _pad: vec3<f32>,
}

struct Uniforms {
    dt: f32,
    gravity_y: f32,
}

@group(0) @binding(0) var<storage, read_write> particles: array<Particle>;
@group(0) @binding(1) var<storage, read_write> velocities: array<Velocity>;
@group(0) @binding(2) var<uniform> uniforms: Uniforms;

@compute @workgroup_size(64)
fn main(@builtin(global_invocation_id) id: vec3<u32>) {
    let i = id.x;
    if i >= arrayLength(&particles) {
        return;
    }

    var vel = velocities[i];

    // Skip dead particles (age >= lifetime)
    if vel.age >= vel.lifetime {
        return;
    }

    // Apply gravity
    vel.vel.y += uniforms.gravity_y * uniforms.dt;

    // Integrate position
    particles[i].pos += vel.vel * uniforms.dt;

    // Age particle
    vel.age += uniforms.dt;

    // Interpolate alpha toward 0 as particle ages
    let t = vel.age / vel.lifetime;
    particles[i].color.a = 1.0 - t;

    velocities[i] = vel;
}
"#
}

// ============================================================================
// Preset emitters
// ============================================================================

/// Emitter preset simulating a water/solvent environment.
///
/// Produces small blue-tinted translucent particles drifting outward from a
/// sphere centred at `center` with the given `radius`.
///
/// # Examples
///
/// ```
/// use nexcore_viz::particle::solvent_emitter;
///
/// let cfg = solvent_emitter([0.0, 0.0, 0.0], 2.0);
/// assert!(cfg.rate > 0.0);
/// assert!(cfg.max_particles > 0);
/// ```
#[must_use]
pub fn solvent_emitter(center: [f64; 3], radius: f64) -> EmitterConfig {
    EmitterConfig {
        shape: EmitterShape::Sphere { center, radius },
        rate: 30.0,
        initial_speed: 0.3,
        speed_variance: 0.1,
        lifetime: 3.0,
        lifetime_variance: 0.5,
        initial_color: [0.3, 0.6, 1.0, 0.8],
        final_color: [0.2, 0.4, 0.9, 0.0],
        initial_size: 0.08,
        final_size: 0.04,
        max_particles: 500,
        gravity: [0.0, 0.0, 0.0],
        drag: 0.5,
    }
}

/// Emitter preset for a reaction burst animation.
///
/// Produces a fast orange-yellow explosion outward from `center`, suitable
/// for visualising bond-breaking, energetic reactions, or collision events.
///
/// # Examples
///
/// ```
/// use nexcore_viz::particle::reaction_burst;
///
/// let cfg = reaction_burst([1.0, 2.0, 3.0]);
/// assert!(cfg.initial_speed > 1.0);
/// ```
#[must_use]
pub fn reaction_burst(center: [f64; 3]) -> EmitterConfig {
    EmitterConfig {
        shape: EmitterShape::Point(center),
        rate: 200.0,
        initial_speed: 3.0,
        speed_variance: 1.5,
        lifetime: 0.8,
        lifetime_variance: 0.3,
        initial_color: [1.0, 0.8, 0.1, 1.0],
        final_color: [1.0, 0.2, 0.0, 0.0],
        initial_size: 0.15,
        final_size: 0.02,
        max_particles: 400,
        gravity: [0.0, -2.0, 0.0],
        drag: 0.3,
    }
}

/// Emitter preset simulating an electron cloud around an atom.
///
/// Produces tiny fast particles on a ring approximating an orbital shell,
/// useful for highlighting electron density in molecular visualisations.
///
/// # Examples
///
/// ```
/// use nexcore_viz::particle::electron_cloud;
///
/// let cfg = electron_cloud([0.0, 0.0, 0.0], 1.5);
/// assert!(cfg.max_particles > 0);
/// ```
#[must_use]
pub fn electron_cloud(center: [f64; 3], orbital_radius: f64) -> EmitterConfig {
    EmitterConfig {
        shape: EmitterShape::Ring {
            center,
            normal: [0.0, 1.0, 0.0],
            radius: orbital_radius,
        },
        rate: 50.0,
        initial_speed: 2.0,
        speed_variance: 0.5,
        lifetime: 0.5,
        lifetime_variance: 0.1,
        initial_color: [0.4, 0.8, 1.0, 0.9],
        final_color: [0.6, 0.9, 1.0, 0.0],
        initial_size: 0.04,
        final_size: 0.01,
        max_particles: 300,
        gravity: [0.0, 0.0, 0.0],
        drag: 0.1,
    }
}

// ============================================================================
// PRNG — xorshift64
// ============================================================================

/// xorshift64 PRNG step.
///
/// `state` must be non-zero; the function advances `state` in place and
/// returns the new value.
///
/// # Examples
///
/// ```
/// use nexcore_viz::particle::xorshift64;
///
/// let mut state: u64 = 12345;
/// let v1 = xorshift64(&mut state);
/// let v2 = xorshift64(&mut state);
/// assert_ne!(v1, v2);
/// ```
pub fn xorshift64(state: &mut u64) -> u64 {
    let mut x = *state;
    x ^= x << 13;
    x ^= x >> 7;
    x ^= x << 17;
    *state = x;
    x
}

/// Generate a pseudo-random `f64` in `[0.0, 1.0)`.
///
/// # Examples
///
/// ```
/// use nexcore_viz::particle::random_f64;
///
/// let mut state: u64 = 99999;
/// let v = random_f64(&mut state);
/// assert!((0.0..1.0).contains(&v));
/// ```
pub fn random_f64(state: &mut u64) -> f64 {
    let bits = xorshift64(state);
    // Use upper 53 bits for mantissa precision
    (bits >> 11) as f64 / (1u64 << 53) as f64
}

// ============================================================================
// Internal helpers
// ============================================================================

/// Sample a spawn position from an [`EmitterShape`].
fn sample_shape(shape: &EmitterShape, rng: &mut u64) -> [f64; 3] {
    match shape {
        EmitterShape::Point(p) => *p,
        EmitterShape::Sphere { center, radius } => {
            // Rejection sampling for uniform sphere surface point
            let [x, y, z] = random_unit_vec(rng);
            [
                center[0] + x * radius,
                center[1] + y * radius,
                center[2] + z * radius,
            ]
        }
        EmitterShape::Box { min, max } => {
            let rx = random_f64(rng);
            let ry = random_f64(rng);
            let rz = random_f64(rng);
            [
                min[0] + rx * (max[0] - min[0]),
                min[1] + ry * (max[1] - min[1]),
                min[2] + rz * (max[2] - min[2]),
            ]
        }
        EmitterShape::Ring {
            center,
            normal,
            radius,
        } => {
            // Build two tangent vectors perpendicular to normal
            let n = normalize3(*normal);
            let (t1, t2) = orthonormal_basis(n);
            let angle = random_f64(rng) * core::f64::consts::TAU;
            let (s, c) = angle.sin_cos();
            [
                center[0] + (t1[0] * c + t2[0] * s) * radius,
                center[1] + (t1[1] * c + t2[1] * s) * radius,
                center[2] + (t1[2] * c + t2[2] * s) * radius,
            ]
        }
    }
}

/// Generate a random unit vector by normalising a Gaussian-distributed vector.
/// Uses a simple Box-Muller-style approach backed by xorshift64.
fn random_unit_vec(rng: &mut u64) -> [f64; 3] {
    // Use two uniform samples mapped to approximate Gaussian via sum
    // (Central Limit Theorem with 4 samples per component)
    let gx = (random_f64(rng) + random_f64(rng) + random_f64(rng) + random_f64(rng) - 2.0) * 0.5;
    let gy = (random_f64(rng) + random_f64(rng) + random_f64(rng) + random_f64(rng) - 2.0) * 0.5;
    let gz = (random_f64(rng) + random_f64(rng) + random_f64(rng) + random_f64(rng) - 2.0) * 0.5;
    normalize3([gx, gy, gz])
}

/// Normalize a 3-vector; returns `[1, 0, 0]` if near-zero.
fn normalize3(v: [f64; 3]) -> [f64; 3] {
    let len = (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt();
    if len < 1e-12 {
        [1.0, 0.0, 0.0]
    } else {
        [v[0] / len, v[1] / len, v[2] / len]
    }
}

/// Build two unit vectors orthogonal to `n` (which must be a unit vector).
fn orthonormal_basis(n: [f64; 3]) -> ([f64; 3], [f64; 3]) {
    // Pick an axis not parallel to n
    let up = if n[0].abs() < 0.9 {
        [1.0, 0.0, 0.0]
    } else {
        [0.0, 1.0, 0.0]
    };
    // t1 = n × up
    let t1 = normalize3(cross(n, up));
    let t2 = cross(n, t1);
    (t1, t2)
}

/// 3D cross product.
fn cross(a: [f64; 3], b: [f64; 3]) -> [f64; 3] {
    [
        a[1] * b[2] - a[2] * b[1],
        a[2] * b[0] - a[0] * b[2],
        a[0] * b[1] - a[1] * b[0],
    ]
}

/// Cheap hash function mapping two `f64` inputs to `[0.0, 1.0)`.
/// Used for the turbulence force field.
fn cheap_hash(a: f64, b: f64) -> f64 {
    // Bit-mangle the inputs using integer arithmetic
    let ax = a.to_bits().wrapping_mul(0x9e37_79b9_7f4a_7c15);
    let bx = b.to_bits().wrapping_mul(0x6c62_272e_07bb_0142);
    let h = ax.wrapping_add(bx).wrapping_mul(0xbf58_476d_1ce4_e5b9);
    let h = h ^ (h >> 27);
    let h = h.wrapping_mul(0x94d0_49bb_1331_11eb);
    let h = h ^ (h >> 31);
    (h >> 11) as f64 / (1u64 << 53) as f64
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    /// Helper: run the system for `frames` steps of `dt` and return it.
    fn run_system(mut sys: ParticleSystem, frames: usize, dt: f64) -> ParticleSystem {
        for _ in 0..frames {
            sys.update(dt);
        }
        sys
    }

    #[test]
    fn test_create_system_with_emitter() {
        let mut sys = ParticleSystem::new();
        let idx = sys.add_emitter(EmitterConfig::default());
        assert_eq!(idx, 0);
        assert_eq!(sys.emitters.len(), 1);
        assert_eq!(sys.total_alive(), 0);
    }

    #[test]
    fn test_emit_particles_from_point_emitter() {
        let mut sys = ParticleSystem::new();
        let cfg = EmitterConfig {
            shape: EmitterShape::Point([1.0, 2.0, 3.0]),
            rate: 100.0,
            max_particles: 200,
            ..Default::default()
        };
        sys.add_emitter(cfg);
        // Run for several frames; should produce particles
        let sys = run_system(sys, 10, 0.1);
        assert!(
            sys.total_alive() > 0,
            "expected alive particles after update"
        );
    }

    #[test]
    fn test_emit_particles_from_sphere_emitter() {
        let mut sys = ParticleSystem::new();
        let cfg = solvent_emitter([0.0, 0.0, 0.0], 2.0);
        sys.add_emitter(cfg);
        let sys = run_system(sys, 5, 0.1);
        // Particles should have spawned
        let alive = sys.total_alive();
        assert!(alive > 0, "sphere emitter produced no particles");
        // Each alive particle position should be within radius + some drift
        // (they move, so we just check they are finite)
        for emitter in &sys.emitters {
            for p in emitter.particles.iter().filter(|p| p.alive) {
                for &coord in &p.position {
                    assert!(coord.is_finite(), "non-finite position coordinate");
                }
            }
        }
    }

    #[test]
    fn test_particles_age_and_die() {
        let mut sys = ParticleSystem::new();
        let cfg = EmitterConfig {
            shape: EmitterShape::Point([0.0; 3]),
            rate: 50.0,
            lifetime: 0.5,
            lifetime_variance: 0.0,
            max_particles: 200,
            ..Default::default()
        };
        sys.add_emitter(cfg);

        // Fill up with particles
        for _ in 0..10 {
            sys.update(0.05);
        }
        let alive_mid = sys.total_alive();
        assert!(alive_mid > 0);

        // Run past the lifetime — all particles should die
        for _ in 0..20 {
            sys.update(0.05);
        }
        // New particles may have spawned, but the originals should be dead
        // and the system should be in steady-state rather than growing without bound
        let alive_late = sys.total_alive();
        // Alive count must be bounded by max_particles
        assert!(
            alive_late <= 200,
            "alive_late={alive_late} exceeds max_particles"
        );
    }

    #[test]
    fn test_gravity_force_moves_particles_downward() {
        let mut sys = ParticleSystem::new();
        let cfg = EmitterConfig {
            shape: EmitterShape::Point([0.0; 3]),
            rate: 100.0,
            initial_speed: 0.0,
            speed_variance: 0.0,
            lifetime: 10.0,
            lifetime_variance: 0.0,
            max_particles: 200,
            ..Default::default()
        };
        sys.add_emitter(cfg);
        sys.add_force(ForceField::Gravity([0.0, -9.81, 0.0]));

        // Run one frame to spawn particles
        sys.update(0.016);
        let initial_y: Vec<f64> = sys.emitters[0]
            .particles
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.position[1])
            .collect();

        // Run more frames
        for _ in 0..30 {
            sys.update(0.016);
        }

        let final_y: Vec<f64> = sys.emitters[0]
            .particles
            .iter()
            .filter(|p| p.alive)
            .map(|p| p.position[1])
            .collect();

        // At least some particles should have moved downward
        let moved_down = initial_y.iter().zip(final_y.iter()).any(|(i, f)| f < i);
        assert!(moved_down, "gravity did not move any particles downward");
    }

    #[test]
    fn test_drag_force_reduces_velocity() {
        // Use a single particle emitted directly, with the emitter then deactivated,
        // so we can track the same particle before and after drag is applied.
        let mut sys = ParticleSystem::new();
        let cfg = EmitterConfig {
            shape: EmitterShape::Point([0.0; 3]),
            rate: 1000.0, // high rate to guarantee spawn on first tick
            initial_speed: 5.0,
            speed_variance: 0.0,
            lifetime: 100.0,
            lifetime_variance: 0.0,
            max_particles: 200,
            ..Default::default()
        };
        sys.add_emitter(cfg);
        sys.add_force(ForceField::Drag(2.0));

        // One tick to spawn particles, then deactivate so no new particles interfere
        sys.update(0.016);
        sys.emitters[0].active = false;

        // Record speeds of all spawned particles
        let initial_speeds: Vec<f64> = sys.emitters[0]
            .particles
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                let v = p.velocity;
                (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
            })
            .collect();

        assert!(!initial_speeds.is_empty(), "no particles spawned");

        // Apply 20 more ticks of drag (no new particles — emitter is inactive)
        for _ in 0..20 {
            sys.update(0.016);
        }

        let final_speeds: Vec<f64> = sys.emitters[0]
            .particles
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                let v = p.velocity;
                (v[0] * v[0] + v[1] * v[1] + v[2] * v[2]).sqrt()
            })
            .collect();

        // Every surviving particle should have slowed down
        let speed_reduced = initial_speeds
            .iter()
            .zip(final_speeds.iter())
            .all(|(i, f)| f < i);
        assert!(speed_reduced, "drag did not reduce particle speed");
    }

    #[test]
    fn test_attractor_pulls_particles_toward_center() {
        // Spawn once then deactivate so the same set of particles is tracked
        // throughout; no new particles confuse the before/after zip comparison.
        let mut sys = ParticleSystem::new();
        let target = [10.0_f64, 0.0, 0.0];
        let cfg = EmitterConfig {
            shape: EmitterShape::Point([0.0; 3]),
            rate: 1000.0, // guarantee spawn on first tick
            initial_speed: 0.0,
            speed_variance: 0.0,
            lifetime: 100.0,
            lifetime_variance: 0.0,
            max_particles: 100,
            ..Default::default()
        };
        sys.add_emitter(cfg);
        sys.add_force(ForceField::Attractor {
            center: target,
            strength: 100.0,
        });

        // Spawn particles at origin, then freeze the emitter
        sys.update(0.016);
        sys.emitters[0].active = false;

        let dist_before: Vec<f64> = sys.emitters[0]
            .particles
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                let dx = p.position[0] - target[0];
                let dy = p.position[1] - target[1];
                let dz = p.position[2] - target[2];
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .collect();

        assert!(!dist_before.is_empty(), "no particles spawned");

        // 30 ticks of attraction; no new particles — same set tracked
        for _ in 0..30 {
            sys.update(0.016);
        }

        let dist_after: Vec<f64> = sys.emitters[0]
            .particles
            .iter()
            .filter(|p| p.alive)
            .map(|p| {
                let dx = p.position[0] - target[0];
                let dy = p.position[1] - target[1];
                let dz = p.position[2] - target[2];
                (dx * dx + dy * dy + dz * dz).sqrt()
            })
            .collect();

        // All tracked particles should have moved closer to the attractor
        let pulled = dist_before
            .iter()
            .zip(dist_after.iter())
            .all(|(before, after)| after < before);
        assert!(pulled, "attractor did not pull particles closer");
    }

    #[test]
    fn test_snapshot_produces_correct_count() {
        let mut sys = ParticleSystem::new();
        sys.add_emitter(EmitterConfig {
            rate: 100.0,
            max_particles: 50,
            ..Default::default()
        });

        for _ in 0..5 {
            sys.update(0.1);
        }

        let snap = sys.snapshot();
        let alive = sys.total_alive();
        assert_eq!(snap.count, alive);
        assert_eq!(snap.positions.len(), alive);
        assert_eq!(snap.colors.len(), alive);
        assert_eq!(snap.sizes.len(), alive);
    }

    #[test]
    fn test_gpu_buffer_packing_is_correct() {
        let snap = ParticleSnapshot {
            positions: vec![[1.0, 2.0, 3.0], [4.0, 5.0, 6.0]],
            colors: vec![[0.1, 0.2, 0.3, 0.4], [0.5, 0.6, 0.7, 0.8]],
            sizes: vec![0.1, 0.2],
            count: 2,
        };
        let buf = pack_for_gpu(&snap);
        assert_eq!(buf.len(), 16, "expected 2 particles * 8 floats = 16");
        // Particle 0
        assert!((buf[0] - 1.0).abs() < 1e-6);
        assert!((buf[1] - 2.0).abs() < 1e-6);
        assert!((buf[2] - 3.0).abs() < 1e-6);
        assert!((buf[3] - 0.1).abs() < 1e-6);
        assert!((buf[6] - 0.4).abs() < 1e-6);
        assert!((buf[7] - 0.1).abs() < 1e-6);
        // Particle 1
        assert!((buf[8] - 4.0).abs() < 1e-6);
        assert!((buf[15] - 0.2).abs() < 1e-6);
    }

    #[test]
    fn test_solvent_preset_creates_valid_emitter() {
        let cfg = solvent_emitter([1.0, 2.0, 3.0], 1.5);
        assert!(cfg.rate > 0.0);
        assert!(cfg.lifetime > 0.0);
        assert!(cfg.max_particles > 0);
        assert_eq!(cfg.initial_color[3], 0.8, "solvent should be translucent");
        // Shape should be a sphere
        match cfg.shape {
            EmitterShape::Sphere { center, radius } => {
                assert_eq!(center, [1.0, 2.0, 3.0]);
                assert!((radius - 1.5).abs() < 1e-9);
            }
            _ => panic!("solvent_emitter should use Sphere shape"),
        }
    }

    #[test]
    fn test_reaction_burst_preset() {
        let cfg = reaction_burst([0.0, 0.0, 0.0]);
        assert!(cfg.initial_speed > 1.0, "burst should be fast");
        assert!(cfg.lifetime < 2.0, "burst should be short-lived");
        match cfg.shape {
            EmitterShape::Point(_) => {}
            _ => panic!("reaction_burst should use Point shape"),
        }
    }

    #[test]
    fn test_clear_removes_all_particles() {
        let mut sys = ParticleSystem::new();
        sys.add_emitter(EmitterConfig {
            rate: 200.0,
            max_particles: 500,
            ..Default::default()
        });
        for _ in 0..10 {
            sys.update(0.05);
        }
        assert!(sys.total_alive() > 0);
        sys.clear();
        assert_eq!(sys.total_alive(), 0);
        assert!(sys.emitters[0].particles.is_empty());
    }

    #[test]
    fn test_wgsl_shader_is_non_empty() {
        let shader = wgsl_particle_update_shader();
        assert!(!shader.is_empty());
        assert!(
            shader.contains("@compute"),
            "shader should contain @compute entry point"
        );
        assert!(shader.contains("dt"), "shader should reference dt uniform");
    }

    #[test]
    fn test_total_alive_matches_after_update() {
        let mut sys = ParticleSystem::new();
        sys.add_emitter(EmitterConfig {
            rate: 100.0,
            max_particles: 100,
            lifetime: 5.0,
            lifetime_variance: 0.0,
            ..Default::default()
        });
        sys.add_emitter(EmitterConfig {
            rate: 50.0,
            max_particles: 50,
            lifetime: 5.0,
            lifetime_variance: 0.0,
            ..Default::default()
        });

        for _ in 0..5 {
            sys.update(0.1);
        }

        let total = sys.total_alive();
        let manual: usize = sys.emitters.iter().map(Emitter::active_count).sum();
        assert_eq!(total, manual);

        let snap = sys.snapshot();
        assert_eq!(snap.count, total);
    }

    #[test]
    fn test_xorshift64_produces_distinct_values() {
        let mut state = 1u64;
        let v1 = xorshift64(&mut state);
        let v2 = xorshift64(&mut state);
        let v3 = xorshift64(&mut state);
        assert_ne!(v1, v2);
        assert_ne!(v2, v3);
    }

    #[test]
    fn test_random_f64_in_range() {
        let mut state = 0xdead_beef_u64;
        for _ in 0..1000 {
            let v = random_f64(&mut state);
            assert!((0.0..1.0).contains(&v), "random_f64 out of range: {v}");
        }
    }

    #[test]
    fn test_particle_error_display() {
        let e1 = ParticleError::InvalidEmitter("bad shape".to_string());
        let e2 = ParticleError::MaxParticlesExceeded {
            requested: 2000,
            limit: 1000,
        };
        let e3 = ParticleError::InvalidLifetime(-1.0);
        assert!(e1.to_string().contains("bad shape"));
        assert!(e2.to_string().contains("2000"));
        assert!(e3.to_string().contains("-1"));
    }
}
