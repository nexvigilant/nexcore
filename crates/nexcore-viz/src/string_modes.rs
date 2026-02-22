//! # String Vibration Modes
//!
//! A string theory vibration mode engine for visualizing standing waves,
//! open/closed string harmonics, and mode superposition.
//!
//! ## Physics Background
//!
//! The module models a classical string under tension, which exhibits
//! quantized vibration modes. Two boundary condition types are supported:
//!
//! - **Open strings** (Dirichlet BCs): endpoints are fixed at `y=0`.
//!   Mode shapes are `sin(nπx/L)` with frequencies `ω_n = nπv/L`.
//! - **Closed strings** (periodic BCs): the string loops back on itself.
//!   Mode shapes are traveling waves with frequencies `ω_n = 2nπv/L`.
//!
//! ## Example
//!
//! ```
//! use nexcore_viz::string_modes::{StringConfig, StringType, VibrationMode, compute_string_state};
//!
//! let config = StringConfig {
//!     length: 1.0,
//!     tension: 1.0,
//!     linear_density: 1.0,
//!     num_points: 50,
//!     string_type: StringType::Open,
//!     max_modes: 10,
//! };
//! let modes = vec![VibrationMode { mode_number: 1, amplitude: 1.0, phase: 0.0 }];
//! if let Ok(state) = compute_string_state(&modes, 0.0, &config) {
//!     assert_eq!(state.points.len(), 50);
//! }
//! ```

use std::f64::consts::{PI, TAU};

use serde::{Deserialize, Serialize};

// ---------------------------------------------------------------------------
// Error type
// ---------------------------------------------------------------------------

/// Errors that can occur during string mode computation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub enum StringModeError {
    /// A mode number of zero or an otherwise invalid mode was supplied.
    InvalidMode,
    /// The string length is non-positive or otherwise invalid.
    InvalidLength,
    /// The modes slice was empty when at least one mode is required.
    EmptyModes,
    /// More modes were supplied than `max_modes` allows.
    TooManyModes(usize),
    /// The animation time step is non-positive or the duration is invalid.
    InvalidTimeStep,
}

impl std::fmt::Display for StringModeError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::InvalidMode => write!(f, "mode number must be >= 1"),
            Self::InvalidLength => write!(f, "string length must be positive"),
            Self::EmptyModes => write!(f, "at least one vibration mode is required"),
            Self::TooManyModes(n) => write!(f, "too many modes: {n} exceeds max_modes limit"),
            Self::InvalidTimeStep => {
                write!(f, "animation duration must be positive and fps must be > 0")
            }
        }
    }
}

impl std::error::Error for StringModeError {}

// ---------------------------------------------------------------------------
// Core types
// ---------------------------------------------------------------------------

/// The boundary condition type for the string.
///
/// - `Open` — Dirichlet boundary conditions: both endpoints are fixed (`y = 0`).
/// - `Closed` — Periodic boundary conditions: the string loops back on itself.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum StringType {
    /// Dirichlet boundary conditions (fixed endpoints).
    #[default]
    Open,
    /// Periodic boundary conditions (string forms a closed loop).
    Closed,
}

/// A single vibration mode of the string.
///
/// Modes are labeled by their harmonic number `n = 1, 2, 3, …`.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct VibrationMode {
    /// Harmonic number (must be >= 1).
    pub mode_number: u32,
    /// Amplitude of this mode.
    pub amplitude: f64,
    /// Initial phase offset in radians.
    pub phase: f64,
}

/// A single point on the string at a given instant.
///
/// For open strings, `z` is always `0.0`. For closed strings, `y` and `z`
/// represent the two independent transverse displacement directions.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct WavePoint {
    /// Position along the string `[0, L]`.
    pub x: f64,
    /// Transverse displacement in the primary direction.
    pub y: f64,
    /// Transverse displacement in the secondary direction (non-zero for closed strings).
    pub z: f64,
}

/// The instantaneous state of the entire string.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct StringState {
    /// Sampled points along the string.
    pub points: Vec<WavePoint>,
    /// Time at which this state was evaluated.
    pub time: f64,
    /// Total mechanical energy stored in all active modes.
    pub total_energy: f64,
    /// Boundary condition type used for this state.
    pub string_type: StringType,
}

/// Configuration parameters for a vibrating string.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct StringConfig {
    /// Physical length of the string (metres or arbitrary units).
    pub length: f64,
    /// String tension `T` (force units).
    pub tension: f64,
    /// Linear mass density `μ` (mass per unit length).
    pub linear_density: f64,
    /// Number of sample points along the string when building a [`StringState`].
    pub num_points: usize,
    /// Open or closed boundary conditions.
    pub string_type: StringType,
    /// Maximum number of modes that `compute_string_state` will accept.
    pub max_modes: usize,
}

impl Default for StringConfig {
    fn default() -> Self {
        Self {
            length: 1.0,
            tension: 1.0,
            linear_density: 1.0,
            num_points: 200,
            string_type: StringType::Open,
            max_modes: 50,
        }
    }
}

/// A sequence of [`StringState`] frames representing a time-evolving animation.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModeAnimation {
    /// Ordered list of animation frames.
    pub frames: Vec<StringState>,
    /// Total animation duration in seconds.
    pub duration: f64,
    /// Frames per second.
    pub fps: f64,
}

/// Spectral properties of a single mode.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct ModeSpectrum {
    /// Harmonic number.
    pub mode_number: u32,
    /// Angular frequency `ω_n` (radians per second).
    pub frequency: f64,
    /// Mechanical energy stored in this mode (assumes unit amplitude `A = 1`).
    pub energy: f64,
    /// Wavelength `λ_n`.
    pub wavelength: f64,
}

// ---------------------------------------------------------------------------
// Core physics functions
// ---------------------------------------------------------------------------

/// Compute the wave speed `v = sqrt(T / μ)`.
///
/// # Arguments
///
/// * `config` — String configuration providing tension `T` and linear density `μ`.
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, wave_speed};
///
/// let config = StringConfig { tension: 4.0, linear_density: 1.0, ..Default::default() };
/// assert!((wave_speed(&config) - 2.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn wave_speed(config: &StringConfig) -> f64 {
    (config.tension / config.linear_density).sqrt()
}

/// Compute the angular frequency `ω_n` for harmonic `mode`.
///
/// - Open string:   `ω_n = n π v / L`
/// - Closed string: `ω_n = 2 n π v / L`
///
/// # Arguments
///
/// * `mode`   — Harmonic number (≥ 1; passing 0 returns 0.0).
/// * `config` — String configuration.
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, StringType, mode_frequency};
///
/// let cfg = StringConfig::default(); // v=1, L=1, Open
/// // ω_1 = π
/// assert!((mode_frequency(1, &cfg) - std::f64::consts::PI).abs() < 1e-12);
/// ```
#[must_use]
pub fn mode_frequency(mode: u32, config: &StringConfig) -> f64 {
    let v = wave_speed(config);
    let n = f64::from(mode);
    match config.string_type {
        StringType::Open => n * PI * v / config.length,
        StringType::Closed => 2.0 * n * PI * v / config.length,
    }
}

/// Compute the wavelength `λ_n` for harmonic `mode`.
///
/// - Open string:   `λ_n = 2L / n`
/// - Closed string: `λ_n = L / n`
///
/// # Arguments
///
/// * `mode`   — Harmonic number (≥ 1; passing 0 yields infinity).
/// * `config` — String configuration.
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, mode_wavelength};
///
/// let cfg = StringConfig { length: 2.0, ..Default::default() };
/// // Open fundamental: λ_1 = 2L = 4.0
/// assert!((mode_wavelength(1, &cfg) - 4.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn mode_wavelength(mode: u32, config: &StringConfig) -> f64 {
    let n = f64::from(mode);
    if n == 0.0 {
        return f64::INFINITY;
    }
    match config.string_type {
        StringType::Open => 2.0 * config.length / n,
        StringType::Closed => config.length / n,
    }
}

/// Compute the mechanical energy stored in a single mode.
///
/// `E_n = (1/2) μ ω_n² A² (L/2)`
///
/// # Arguments
///
/// * `mode`   — Vibration mode carrying amplitude information.
/// * `config` — String configuration.
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, VibrationMode, mode_energy};
///
/// let cfg = StringConfig::default();
/// let m = VibrationMode { mode_number: 1, amplitude: 0.0, phase: 0.0 };
/// assert_eq!(mode_energy(&m, &cfg), 0.0);
/// ```
#[must_use]
pub fn mode_energy(mode: &VibrationMode, config: &StringConfig) -> f64 {
    let omega = mode_frequency(mode.mode_number, config);
    0.5 * config.linear_density * omega * omega * mode.amplitude * mode.amplitude
        * (config.length / 2.0)
}

/// Evaluate the displacement of an **open** string mode at position `x` and time `t`.
///
/// `y_n(x, t) = A sin(nπx/L) cos(ω_n t + φ)`
///
/// # Arguments
///
/// * `mode`   — Mode parameters (number, amplitude, phase).
/// * `x`      — Position along the string `[0, L]`.
/// * `t`      — Time.
/// * `config` — String configuration.
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, VibrationMode, evaluate_open_mode};
///
/// let cfg = StringConfig::default();
/// let m = VibrationMode { mode_number: 1, amplitude: 1.0, phase: 0.0 };
/// // Endpoints must be zero (Dirichlet BC)
/// assert!(evaluate_open_mode(&m, 0.0, 0.0, &cfg).abs() < 1e-12);
/// assert!(evaluate_open_mode(&m, cfg.length, 0.0, &cfg).abs() < 1e-12);
/// ```
#[must_use]
pub fn evaluate_open_mode(mode: &VibrationMode, x: f64, t: f64, config: &StringConfig) -> f64 {
    let n = f64::from(mode.mode_number);
    let omega = mode_frequency(mode.mode_number, config);
    let spatial = (n * PI * x / config.length).sin();
    let temporal = (omega * t + mode.phase).cos();
    mode.amplitude * spatial * temporal
}

/// Evaluate the displacement of a **closed** string mode at position `x` and time `t`.
///
/// Returns `(y, z)` — the two transverse displacement components formed by the
/// superposition of left- and right-traveling waves:
///
/// - `y_n(x, t) = A cos(2nπx/L − ω_n t + φ)`
/// - `z_n(x, t) = A sin(2nπx/L − ω_n t + φ)`
///
/// # Arguments
///
/// * `mode`   — Mode parameters.
/// * `x`      — Position along the string `[0, L]`.
/// * `t`      — Time.
/// * `config` — String configuration.
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, StringType, VibrationMode, evaluate_closed_mode};
///
/// let cfg = StringConfig { string_type: StringType::Closed, ..Default::default() };
/// let m = VibrationMode { mode_number: 1, amplitude: 1.0, phase: 0.0 };
/// let (y, z) = evaluate_closed_mode(&m, 0.0, 0.0, &cfg);
/// // At x=0, t=0, phase=0: y = A*cos(0) = 1, z = A*sin(0) = 0
/// assert!((y - 1.0).abs() < 1e-12);
/// assert!(z.abs() < 1e-12);
/// ```
#[must_use]
pub fn evaluate_closed_mode(
    mode: &VibrationMode,
    x: f64,
    t: f64,
    config: &StringConfig,
) -> (f64, f64) {
    let n = f64::from(mode.mode_number);
    let omega = mode_frequency(mode.mode_number, config);
    let arg = TAU * n * x / config.length - omega * t + mode.phase;
    (mode.amplitude * arg.cos(), mode.amplitude * arg.sin())
}

/// Compute the instantaneous state of the string by superposing all modes.
///
/// Samples the string at `config.num_points` evenly-spaced positions and returns
/// a [`StringState`] containing the displacement at each position.
///
/// # Errors
///
/// Returns [`StringModeError::EmptyModes`] when `modes` is empty.
/// Returns [`StringModeError::TooManyModes`] when `modes.len() > config.max_modes`.
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, VibrationMode, compute_string_state};
///
/// let cfg = StringConfig { num_points: 10, ..Default::default() };
/// let modes = vec![VibrationMode { mode_number: 1, amplitude: 1.0, phase: 0.0 }];
/// if let Ok(state) = compute_string_state(&modes, 0.0, &cfg) {
///     assert_eq!(state.points.len(), 10);
/// }
/// ```
pub fn compute_string_state(
    modes: &[VibrationMode],
    t: f64,
    config: &StringConfig,
) -> Result<StringState, StringModeError> {
    if modes.is_empty() {
        return Err(StringModeError::EmptyModes);
    }
    if modes.len() > config.max_modes {
        return Err(StringModeError::TooManyModes(modes.len()));
    }

    let n_pts = config.num_points;
    let mut points = Vec::with_capacity(n_pts);

    for i in 0..n_pts {
        // Distribute points uniformly over [0, L]
        let x = if n_pts <= 1 {
            0.0
        } else {
            config.length * (i as f64) / ((n_pts - 1) as f64)
        };

        let (mut y, mut z) = (0.0_f64, 0.0_f64);
        for mode in modes {
            match config.string_type {
                StringType::Open => {
                    y += evaluate_open_mode(mode, x, t, config);
                }
                StringType::Closed => {
                    let (dy, dz) = evaluate_closed_mode(mode, x, t, config);
                    y += dy;
                    z += dz;
                }
            }
        }
        points.push(WavePoint { x, y, z });
    }

    let energy = total_energy(modes, config);
    Ok(StringState {
        points,
        time: t,
        total_energy: energy,
        string_type: config.string_type,
    })
}

/// Generate an animation by computing the string state at evenly-spaced time steps.
///
/// The number of frames is `ceil(duration * fps)`.
///
/// # Errors
///
/// Returns [`StringModeError::InvalidTimeStep`] if `duration <= 0` or `fps <= 0`.
/// Returns [`StringModeError::EmptyModes`] or [`StringModeError::TooManyModes`]
/// propagated from [`compute_string_state`].
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, VibrationMode, animate_modes};
///
/// let cfg = StringConfig { num_points: 10, ..Default::default() };
/// let modes = vec![VibrationMode { mode_number: 1, amplitude: 0.5, phase: 0.0 }];
/// if let Ok(anim) = animate_modes(&modes, 1.0, 10.0, &cfg) {
///     assert_eq!(anim.frames.len(), 10);
/// }
/// ```
pub fn animate_modes(
    modes: &[VibrationMode],
    duration: f64,
    fps: f64,
    config: &StringConfig,
) -> Result<ModeAnimation, StringModeError> {
    if duration <= 0.0 || fps <= 0.0 {
        return Err(StringModeError::InvalidTimeStep);
    }

    let n_frames = (duration * fps).ceil() as usize;
    let dt = duration / n_frames as f64;
    let mut frames = Vec::with_capacity(n_frames);

    for i in 0..n_frames {
        let t = i as f64 * dt;
        let state = compute_string_state(modes, t, config)?;
        frames.push(state);
    }

    Ok(ModeAnimation {
        frames,
        duration,
        fps,
    })
}

/// Compute spectral properties (frequency, energy, wavelength) for the first `n_modes` harmonics.
///
/// The energy is computed assuming unit amplitude (`A = 1`) for each mode so that
/// relative spectral weights can be compared.
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, mode_spectrum};
///
/// let cfg = StringConfig::default();
/// let spectrum = mode_spectrum(&cfg, 3);
/// assert_eq!(spectrum.len(), 3);
/// assert!(spectrum[1].frequency > spectrum[0].frequency);
/// ```
#[must_use]
pub fn mode_spectrum(config: &StringConfig, n_modes: u32) -> Vec<ModeSpectrum> {
    (1..=n_modes)
        .map(|n| {
            let unit_mode = VibrationMode {
                mode_number: n,
                amplitude: 1.0,
                phase: 0.0,
            };
            ModeSpectrum {
                mode_number: n,
                frequency: mode_frequency(n, config),
                energy: mode_energy(&unit_mode, config),
                wavelength: mode_wavelength(n, config),
            }
        })
        .collect()
}

/// Compute the total mechanical energy as the sum of individual mode energies.
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, VibrationMode, total_energy, mode_energy};
///
/// let cfg = StringConfig::default();
/// let modes = vec![
///     VibrationMode { mode_number: 1, amplitude: 1.0, phase: 0.0 },
///     VibrationMode { mode_number: 2, amplitude: 0.5, phase: 0.0 },
/// ];
/// let e1 = mode_energy(&modes[0], &cfg);
/// let e2 = mode_energy(&modes[1], &cfg);
/// assert!((total_energy(&modes, &cfg) - (e1 + e2)).abs() < 1e-12);
/// ```
#[must_use]
pub fn total_energy(modes: &[VibrationMode], config: &StringConfig) -> f64 {
    modes.iter().map(|m| mode_energy(m, config)).sum()
}

/// Compute the fundamental angular frequency `ω_1`.
///
/// This is a convenience alias for `mode_frequency(1, config)`.
///
/// - Open string:   `ω_1 = πv / L`
/// - Closed string: `ω_1 = 2πv / L`
///
/// Divide by `2π` (TAU) to obtain ordinary frequency in Hz.
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, StringType, fundamental_frequency, mode_frequency};
///
/// let cfg = StringConfig::default();
/// assert!((fundamental_frequency(&cfg) - mode_frequency(1, &cfg)).abs() < 1e-12);
/// ```
#[must_use]
pub fn fundamental_frequency(config: &StringConfig) -> f64 {
    mode_frequency(1, config)
}

/// Compute the `x`-positions of nodes (zero-displacement points) for an **open** string mode.
///
/// For mode `n`, the nodes are at `x_k = k * L / n` for `k = 0, 1, …, n`.
/// This yields `n + 1` nodes including the fixed endpoints.
///
/// # Arguments
///
/// * `mode`   — Harmonic number.
/// * `config` — String configuration (only `length` is used).
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, standing_wave_nodes};
///
/// let cfg = StringConfig { length: 1.0, ..Default::default() };
/// // Fundamental: nodes at x=0 and x=1
/// let nodes = standing_wave_nodes(1, &cfg);
/// assert_eq!(nodes.len(), 2);
/// assert!((nodes[0] - 0.0).abs() < 1e-12);
/// assert!((nodes[1] - 1.0).abs() < 1e-12);
/// ```
#[must_use]
pub fn standing_wave_nodes(mode: u32, config: &StringConfig) -> Vec<f64> {
    let n = mode as usize;
    (0..=n)
        .map(|k| config.length * (k as f64) / (n as f64))
        .collect()
}

/// Compute the `x`-positions of antinodes (maximum-displacement points) for an **open** string mode.
///
/// For mode `n`, the antinodes are at `x_k = (2k − 1) * L / (2n)` for `k = 1, …, n`.
/// This yields exactly `n` antinodes.
///
/// # Arguments
///
/// * `mode`   — Harmonic number.
/// * `config` — String configuration (only `length` is used).
///
/// # Examples
///
/// ```
/// use nexcore_viz::string_modes::{StringConfig, standing_wave_antinodes};
///
/// let cfg = StringConfig { length: 1.0, ..Default::default() };
/// // Fundamental: single antinode at x=0.5
/// let anti = standing_wave_antinodes(1, &cfg);
/// assert_eq!(anti.len(), 1);
/// assert!((anti[0] - 0.5).abs() < 1e-12);
/// ```
#[must_use]
pub fn standing_wave_antinodes(mode: u32, config: &StringConfig) -> Vec<f64> {
    let n = mode as usize;
    (1..=n)
        .map(|k| config.length * (2 * k - 1) as f64 / (2 * n) as f64)
        .collect()
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    const EPS: f64 = 1e-10;

    fn open_cfg() -> StringConfig {
        StringConfig::default() // length=1, T=1, μ=1, Open
    }

    fn closed_cfg() -> StringConfig {
        StringConfig {
            string_type: StringType::Closed,
            ..Default::default()
        }
    }

    // --- wave_speed ---

    #[test]
    fn test_wave_speed_unit() {
        let cfg = open_cfg();
        assert!((wave_speed(&cfg) - 1.0).abs() < EPS);
    }

    #[test]
    fn test_wave_speed_four_tension() {
        let cfg = StringConfig {
            tension: 4.0,
            ..Default::default()
        };
        assert!((wave_speed(&cfg) - 2.0).abs() < EPS);
    }

    // --- mode_frequency ---

    #[test]
    fn test_mode_frequency_fundamental_open() {
        let cfg = open_cfg();
        let expected = PI * wave_speed(&cfg) / cfg.length;
        assert!((mode_frequency(1, &cfg) - expected).abs() < EPS);
    }

    #[test]
    fn test_mode_frequency_fundamental_closed() {
        let cfg = closed_cfg();
        let expected = TAU * wave_speed(&cfg) / cfg.length;
        assert!((mode_frequency(1, &cfg) - expected).abs() < EPS);
    }

    #[test]
    fn test_mode_frequency_closed_is_double_open() {
        let cfg_open = open_cfg();
        let cfg_closed = closed_cfg();
        let f_open = mode_frequency(1, &cfg_open);
        let f_closed = mode_frequency(1, &cfg_closed);
        assert!((f_closed - 2.0 * f_open).abs() < EPS);
    }

    #[test]
    fn test_mode_frequency_second_harmonic_double() {
        let cfg = open_cfg();
        let f1 = mode_frequency(1, &cfg);
        let f2 = mode_frequency(2, &cfg);
        assert!((f2 - 2.0 * f1).abs() < EPS);
    }

    // --- mode_wavelength ---

    #[test]
    fn test_mode_wavelength_fundamental_open() {
        let cfg = open_cfg();
        assert!((mode_wavelength(1, &cfg) - 2.0 * cfg.length).abs() < EPS);
    }

    #[test]
    fn test_mode_wavelength_fundamental_closed() {
        let cfg = closed_cfg();
        assert!((mode_wavelength(1, &cfg) - cfg.length).abs() < EPS);
    }

    // --- mode_energy ---

    #[test]
    fn test_mode_energy_positive_nonzero_amplitude() {
        let cfg = open_cfg();
        let m = VibrationMode {
            mode_number: 1,
            amplitude: 1.0,
            phase: 0.0,
        };
        assert!(mode_energy(&m, &cfg) > 0.0);
    }

    #[test]
    fn test_mode_energy_zero_amplitude() {
        let cfg = open_cfg();
        let m = VibrationMode {
            mode_number: 1,
            amplitude: 0.0,
            phase: 0.0,
        };
        assert_eq!(mode_energy(&m, &cfg), 0.0);
    }

    // --- evaluate_open_mode ---

    #[test]
    fn test_evaluate_open_mode_endpoints_zero() {
        let cfg = open_cfg();
        let m = VibrationMode {
            mode_number: 1,
            amplitude: 1.0,
            phase: 0.0,
        };
        assert!(evaluate_open_mode(&m, 0.0, 0.0, &cfg).abs() < EPS);
        assert!(evaluate_open_mode(&m, cfg.length, 0.0, &cfg).abs() < EPS);
    }

    #[test]
    fn test_evaluate_open_mode_higher_endpoints_zero() {
        let cfg = open_cfg();
        for n in [2_u32, 3, 5] {
            let m = VibrationMode {
                mode_number: n,
                amplitude: 1.0,
                phase: 0.0,
            };
            assert!(
                evaluate_open_mode(&m, 0.0, 0.0, &cfg).abs() < EPS,
                "mode {n} x=0 not zero"
            );
            assert!(
                evaluate_open_mode(&m, cfg.length, 0.0, &cfg).abs() < EPS,
                "mode {n} x=L not zero"
            );
        }
    }

    #[test]
    fn test_evaluate_open_mode_midpoint_t0() {
        let cfg = open_cfg(); // L=1
        let m = VibrationMode {
            mode_number: 1,
            amplitude: 1.0,
            phase: 0.0,
        };
        // y_1(L/2, 0) = sin(π/2) * cos(0) = 1
        let y = evaluate_open_mode(&m, cfg.length / 2.0, 0.0, &cfg);
        assert!((y - 1.0).abs() < EPS);
    }

    #[test]
    fn test_evaluate_open_mode_node_zero() {
        let cfg = open_cfg();
        // Mode 2 has a node at x = L/2
        let m = VibrationMode {
            mode_number: 2,
            amplitude: 1.0,
            phase: 0.0,
        };
        let y = evaluate_open_mode(&m, cfg.length / 2.0, 0.0, &cfg);
        assert!(y.abs() < EPS);
    }

    // --- evaluate_closed_mode ---

    #[test]
    fn test_evaluate_closed_mode_returns_two_components() {
        let cfg = closed_cfg();
        let m = VibrationMode {
            mode_number: 1,
            amplitude: 1.0,
            phase: 0.0,
        };
        let (y, z) = evaluate_closed_mode(&m, 0.0, 0.0, &cfg);
        // At x=0, t=0, φ=0: y = cos(0) = 1, z = sin(0) = 0
        assert!((y - 1.0).abs() < EPS);
        assert!(z.abs() < EPS);
    }

    #[test]
    fn test_evaluate_closed_mode_amplitude_conservation() {
        let cfg = closed_cfg();
        let amp = 2.5;
        let m = VibrationMode {
            mode_number: 1,
            amplitude: amp,
            phase: 0.3,
        };
        // y^2 + z^2 = A^2 (cos^2 + sin^2 = 1)
        let (y, z) = evaluate_closed_mode(&m, 0.4, 0.7, &cfg);
        assert!((y * y + z * z - amp * amp).abs() < EPS);
    }

    // --- compute_string_state ---

    #[test]
    fn test_compute_string_state_empty_modes_error() {
        let cfg = open_cfg();
        let result = compute_string_state(&[], 0.0, &cfg);
        assert_eq!(result, Err(StringModeError::EmptyModes));
    }

    #[test]
    fn test_compute_string_state_too_many_modes_error() {
        let cfg = StringConfig {
            max_modes: 2,
            ..Default::default()
        };
        let modes: Vec<VibrationMode> = (1..=3_u32)
            .map(|n| VibrationMode {
                mode_number: n,
                amplitude: 1.0,
                phase: 0.0,
            })
            .collect();
        assert_eq!(
            compute_string_state(&modes, 0.0, &cfg),
            Err(StringModeError::TooManyModes(3))
        );
    }

    #[test]
    fn test_compute_string_state_point_count() {
        let cfg = StringConfig {
            num_points: 42,
            ..Default::default()
        };
        let modes = vec![VibrationMode {
            mode_number: 1,
            amplitude: 1.0,
            phase: 0.0,
        }];
        if let Ok(state) = compute_string_state(&modes, 0.0, &cfg) {
            assert_eq!(state.points.len(), 42);
        }
    }

    #[test]
    fn test_compute_string_state_open_endpoints_fixed() {
        let cfg = StringConfig {
            num_points: 50,
            ..Default::default()
        };
        let modes = vec![VibrationMode {
            mode_number: 1,
            amplitude: 1.0,
            phase: 0.0,
        }];
        if let Ok(state) = compute_string_state(&modes, 0.0, &cfg) {
            let first_y = state.points.first().map_or(0.0, |p| p.y);
            let last_y = state.points.last().map_or(0.0, |p| p.y);
            assert!(
                first_y.abs() < EPS,
                "open string left endpoint non-zero: {first_y}"
            );
            assert!(
                last_y.abs() < EPS,
                "open string right endpoint non-zero: {last_y}"
            );
        }
    }

    #[test]
    fn test_compute_string_state_energy_stored() {
        let cfg = open_cfg();
        let m = VibrationMode {
            mode_number: 1,
            amplitude: 1.0,
            phase: 0.0,
        };
        if let Ok(state) = compute_string_state(&[m.clone()], 0.0, &cfg) {
            assert!((state.total_energy - mode_energy(&m, &cfg)).abs() < EPS);
        }
    }

    // --- animate_modes ---

    #[test]
    fn test_animate_modes_correct_frame_count() {
        let cfg = StringConfig {
            num_points: 5,
            ..Default::default()
        };
        let modes = vec![VibrationMode {
            mode_number: 1,
            amplitude: 1.0,
            phase: 0.0,
        }];
        if let Ok(anim) = animate_modes(&modes, 1.0, 10.0, &cfg) {
            assert_eq!(anim.frames.len(), 10);
        }
    }

    #[test]
    fn test_animate_modes_invalid_duration_error() {
        let cfg = open_cfg();
        let modes = vec![VibrationMode {
            mode_number: 1,
            amplitude: 1.0,
            phase: 0.0,
        }];
        assert_eq!(
            animate_modes(&modes, 0.0, 10.0, &cfg),
            Err(StringModeError::InvalidTimeStep)
        );
        assert_eq!(
            animate_modes(&modes, -1.0, 10.0, &cfg),
            Err(StringModeError::InvalidTimeStep)
        );
    }

    #[test]
    fn test_animate_modes_invalid_fps_error() {
        let cfg = open_cfg();
        let modes = vec![VibrationMode {
            mode_number: 1,
            amplitude: 1.0,
            phase: 0.0,
        }];
        assert_eq!(
            animate_modes(&modes, 1.0, 0.0, &cfg),
            Err(StringModeError::InvalidTimeStep)
        );
    }

    // --- mode_spectrum ---

    #[test]
    fn test_mode_spectrum_correct_count() {
        let cfg = open_cfg();
        let spectrum = mode_spectrum(&cfg, 5);
        assert_eq!(spectrum.len(), 5);
    }

    #[test]
    fn test_mode_spectrum_frequencies_monotone_increasing() {
        let cfg = open_cfg();
        let spectrum = mode_spectrum(&cfg, 8);
        for w in spectrum.windows(2) {
            assert!(
                w[1].frequency > w[0].frequency,
                "frequency not monotone: {} vs {}",
                w[0].frequency,
                w[1].frequency
            );
        }
    }

    #[test]
    fn test_mode_spectrum_energies_increase() {
        // With unit amplitude, higher modes carry more energy because ω_n grows.
        let cfg = open_cfg();
        let spectrum = mode_spectrum(&cfg, 5);
        for w in spectrum.windows(2) {
            assert!(
                w[1].energy > w[0].energy,
                "energy did not increase: mode {} energy {} vs mode {} energy {}",
                w[0].mode_number,
                w[0].energy,
                w[1].mode_number,
                w[1].energy
            );
        }
    }

    // --- total_energy ---

    #[test]
    fn test_total_energy_equals_sum_of_mode_energies() {
        let cfg = open_cfg();
        let modes = vec![
            VibrationMode {
                mode_number: 1,
                amplitude: 1.0,
                phase: 0.0,
            },
            VibrationMode {
                mode_number: 2,
                amplitude: 0.5,
                phase: 0.0,
            },
            VibrationMode {
                mode_number: 3,
                amplitude: 0.25,
                phase: 1.0,
            },
        ];
        let expected: f64 = modes.iter().map(|m| mode_energy(m, &cfg)).sum();
        assert!((total_energy(&modes, &cfg) - expected).abs() < EPS);
    }

    // --- fundamental_frequency ---

    #[test]
    fn test_fundamental_frequency_open_vs_closed() {
        let f_open = fundamental_frequency(&open_cfg());
        let f_closed = fundamental_frequency(&closed_cfg());
        assert!((f_closed - 2.0 * f_open).abs() < EPS);
    }

    #[test]
    fn test_fundamental_frequency_equals_mode_one() {
        let cfg = open_cfg();
        assert!((fundamental_frequency(&cfg) - mode_frequency(1, &cfg)).abs() < EPS);
    }

    // --- standing_wave_nodes ---

    #[test]
    fn test_standing_wave_nodes_count() {
        let cfg = open_cfg();
        for n in 1_u32..=5 {
            let nodes = standing_wave_nodes(n, &cfg);
            assert_eq!(
                nodes.len(),
                n as usize + 1,
                "mode {n}: expected {} nodes, got {}",
                n + 1,
                nodes.len()
            );
        }
    }

    #[test]
    fn test_standing_wave_nodes_endpoints() {
        let cfg = open_cfg();
        for n in 1_u32..=4 {
            let nodes = standing_wave_nodes(n, &cfg);
            let first = nodes.first().copied().unwrap_or(f64::NAN);
            let last = nodes.last().copied().unwrap_or(f64::NAN);
            assert!(first.abs() < EPS, "mode {n} first node not at 0");
            assert!(
                (last - cfg.length).abs() < EPS,
                "mode {n} last node not at L"
            );
        }
    }

    #[test]
    fn test_standing_wave_nodes_displacement_zero() {
        let cfg = open_cfg();
        for n in 1_u32..=3 {
            let m = VibrationMode {
                mode_number: n,
                amplitude: 1.0,
                phase: 0.0,
            };
            for x in standing_wave_nodes(n, &cfg) {
                let y = evaluate_open_mode(&m, x, 0.0, &cfg);
                assert!(
                    y.abs() < EPS,
                    "mode {n} node at x={x} gives non-zero displacement {y}"
                );
            }
        }
    }

    // --- standing_wave_antinodes ---

    #[test]
    fn test_standing_wave_antinodes_count() {
        let cfg = open_cfg();
        for n in 1_u32..=5 {
            let anti = standing_wave_antinodes(n, &cfg);
            assert_eq!(
                anti.len(),
                n as usize,
                "mode {n}: expected {n} antinodes, got {}",
                anti.len()
            );
        }
    }

    #[test]
    fn test_standing_wave_antinodes_max_displacement() {
        let cfg = open_cfg();
        let amp = 1.0;
        for n in 1_u32..=3 {
            let m = VibrationMode {
                mode_number: n,
                amplitude: amp,
                phase: 0.0,
            };
            for x in standing_wave_antinodes(n, &cfg) {
                let y = evaluate_open_mode(&m, x, 0.0, &cfg);
                // |y| should equal amplitude at antinodes (t=0 means temporal factor = 1)
                assert!(
                    (y.abs() - amp).abs() < EPS,
                    "mode {n} antinode at x={x}: |y|={}, expected {amp}",
                    y.abs()
                );
            }
        }
    }

    // --- StringConfig::default ---

    #[test]
    fn test_string_config_default_values() {
        let cfg = StringConfig::default();
        assert_eq!(cfg.length, 1.0);
        assert_eq!(cfg.tension, 1.0);
        assert_eq!(cfg.linear_density, 1.0);
        assert_eq!(cfg.num_points, 200);
        assert_eq!(cfg.string_type, StringType::Open);
        assert_eq!(cfg.max_modes, 50);
    }

    // --- StringModeError Display ---

    #[test]
    fn test_error_display_messages() {
        assert_eq!(
            StringModeError::InvalidMode.to_string(),
            "mode number must be >= 1"
        );
        assert_eq!(
            StringModeError::InvalidLength.to_string(),
            "string length must be positive"
        );
        assert_eq!(
            StringModeError::EmptyModes.to_string(),
            "at least one vibration mode is required"
        );
        assert_eq!(
            StringModeError::TooManyModes(7).to_string(),
            "too many modes: 7 exceeds max_modes limit"
        );
        assert_eq!(
            StringModeError::InvalidTimeStep.to_string(),
            "animation duration must be positive and fps must be > 0"
        );
    }

    // --- Serde roundtrip ---

    #[test]
    fn test_serde_roundtrip_mode_animation() {
        let cfg = StringConfig {
            num_points: 5,
            ..Default::default()
        };
        let modes = vec![VibrationMode {
            mode_number: 1,
            amplitude: 1.0,
            phase: 0.5,
        }];
        if let Ok(anim) = animate_modes(&modes, 0.5, 4.0, &cfg) {
            if let Ok(json) = serde_json::to_string(&anim) {
                if let Ok(restored) = serde_json::from_str::<ModeAnimation>(&json) {
                    assert_eq!(restored.frames.len(), anim.frames.len());
                    assert!((restored.duration - anim.duration).abs() < EPS);
                    assert!((restored.fps - anim.fps).abs() < EPS);
                }
            }
        }
    }

    // --- Superposition shape ---

    #[test]
    fn test_superposition_modes_1_and_3_shape() {
        // Modes 1 and 3 at t=0 with equal amplitudes.
        // y(x, 0) = sin(πx/L) + sin(3πx/L)
        // At x = L/4: sin(π/4) + sin(3π/4) = √2/2 + √2/2 = √2
        let cfg = StringConfig {
            num_points: 100,
            ..Default::default()
        };
        let modes = vec![
            VibrationMode {
                mode_number: 1,
                amplitude: 1.0,
                phase: 0.0,
            },
            VibrationMode {
                mode_number: 3,
                amplitude: 1.0,
                phase: 0.0,
            },
        ];
        if let Ok(state) = compute_string_state(&modes, 0.0, &cfg) {
            // Find the sample point nearest to x = L/4 = 0.25
            let target_x = 0.25_f64;
            let nearest = state.points.iter().min_by(|a, b| {
                (a.x - target_x)
                    .abs()
                    .partial_cmp(&(b.x - target_x).abs())
                    .unwrap_or(std::cmp::Ordering::Equal)
            });
            if let Some(pt) = nearest {
                let expected = (PI / 4.0).sin() + (3.0 * PI / 4.0).sin();
                assert!(
                    (pt.y - expected).abs() < 0.02,
                    "superposition at x≈0.25: got {}, expected {expected}",
                    pt.y
                );
            }
        }
    }

    #[test]
    fn test_superposition_modes_1_and_3_symmetry() {
        // y(x) = sin(πx/L) + sin(3πx/L) is symmetric about x=L/2
        // due to both modes being odd harmonics.
        let cfg = StringConfig {
            num_points: 201, // odd so midpoint is exact
            ..Default::default()
        };
        let modes = vec![
            VibrationMode {
                mode_number: 1,
                amplitude: 1.0,
                phase: 0.0,
            },
            VibrationMode {
                mode_number: 3,
                amplitude: 1.0,
                phase: 0.0,
            },
        ];
        if let Ok(state) = compute_string_state(&modes, 0.0, &cfg) {
            let n = state.points.len();
            for i in 0..n / 2 {
                let y_left = state.points[i].y;
                let y_right = state.points[n - 1 - i].y;
                assert!(
                    (y_left - y_right).abs() < EPS,
                    "symmetry broken at i={i}: {y_left} vs {y_right}"
                );
            }
        }
    }
}
