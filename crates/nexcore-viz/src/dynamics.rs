//! Molecular dynamics simulation engine.
//!
//! Implements velocity Verlet integration, a Berendsen thermostat, and a
//! Maxwell-Boltzmann velocity initializer for NVT (constant temperature)
//! molecular dynamics. Uses the UFF force field from [`crate::force_field`].
//!
//! Random number generation uses an inline xorshift64 PRNG seeded
//! deterministically — no external `rand` crate is required. Box-Muller
//! transform converts uniform samples to Gaussian deviates for the
//! Maxwell-Boltzmann velocity distribution.
//!
//! ## Units
//!
//! | Quantity | Unit |
//! |----------|------|
//! | Distance | Å (angstrom) |
//! | Time | fs (femtosecond) |
//! | Energy | kcal/mol |
//! | Mass | g/mol (daltons) |
//! | Velocity | Å/fs |
//! | Force | kcal/mol/Å |
//! | Temperature | K |
//!
//! ## Example
//!
//! ```rust
//! use nexcore_viz::molecular::{Atom, Bond, BondOrder, Element, Molecule};
//! use nexcore_viz::force_field::ForceFieldConfig;
//! use nexcore_viz::dynamics::{DynamicsConfig, run_simulation};
//!
//! let mut mol = Molecule::new("Water");
//! mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
//! mol.atoms.push(Atom::new(2, Element::H, [0.96, 0.0, 0.0]));
//! mol.atoms.push(Atom::new(3, Element::H, [-0.24, 0.93, 0.0]));
//! mol.bonds.push(Bond { atom1: 0, atom2: 1, order: BondOrder::Single });
//! mol.bonds.push(Bond { atom1: 0, atom2: 2, order: BondOrder::Single });
//!
//! let config = DynamicsConfig {
//!     timestep: 0.5,
//!     temperature: 300.0,
//!     friction: 0.1,
//!     total_steps: 10,
//!     save_interval: 5,
//! };
//! let ff_config = ForceFieldConfig::default();
//! let result = run_simulation(&mut mol, &config, &ff_config).ok();
//! assert!(result.is_some());
//! ```

use crate::force_field::{
    ForceFieldConfig, ForceFieldError, compute_energy, compute_forces,
};
use crate::molecular::Molecule;
use serde::{Deserialize, Serialize};

// ============================================================================
// Error type
// ============================================================================

/// Errors that can occur during molecular dynamics simulation.
#[derive(Debug, Clone, PartialEq)]
pub enum DynamicsError {
    /// An error propagated from the force field layer.
    ForceField(ForceFieldError),
    /// The requested timestep is non-positive or non-finite.
    InvalidTimestep { timestep: f64 },
    /// The molecule has no atoms to simulate.
    EmptyMolecule,
}

impl std::fmt::Display for DynamicsError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::ForceField(e) => write!(f, "force field error: {e}"),
            Self::InvalidTimestep { timestep } => {
                write!(f, "invalid timestep {timestep}: must be positive and finite")
            }
            Self::EmptyMolecule => write!(f, "molecule has no atoms"),
        }
    }
}

impl std::error::Error for DynamicsError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::ForceField(e) => Some(e),
            _ => None,
        }
    }
}

impl From<ForceFieldError> for DynamicsError {
    fn from(e: ForceFieldError) -> Self {
        Self::ForceField(e)
    }
}

// ============================================================================
// Simulation configuration
// ============================================================================

/// Configuration for a molecular dynamics simulation run.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct DynamicsConfig {
    /// Integration timestep in femtoseconds (fs). Typical: 0.5–2.0 fs.
    pub timestep: f64,
    /// Target temperature in Kelvin.
    pub temperature: f64,
    /// Berendsen thermostat coupling constant (dimensionless, 0–1).
    /// Smaller values give tighter coupling; 0.0 = full rescaling each step.
    pub friction: f64,
    /// Total number of integration steps.
    pub total_steps: usize,
    /// Save a trajectory frame every this many steps.
    pub save_interval: usize,
}

impl Default for DynamicsConfig {
    fn default() -> Self {
        Self {
            timestep: 1.0,
            temperature: 300.0,
            friction: 0.1,
            total_steps: 1000,
            save_interval: 10,
        }
    }
}

// ============================================================================
// Simulation state
// ============================================================================

/// Complete instantaneous state of the MD simulation.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SimulationState {
    /// Atom positions in Å, length = n_atoms.
    pub positions: Vec<[f64; 3]>,
    /// Atom velocities in Å/fs, length = n_atoms.
    pub velocities: Vec<[f64; 3]>,
    /// Per-atom force vectors in kcal/mol/Å, length = n_atoms.
    pub forces: Vec<[f64; 3]>,
    /// Current integration step index.
    pub step: usize,
    /// Simulation time in fs.
    pub time: f64,
    /// Current kinetic energy in kcal/mol.
    pub kinetic_energy: f64,
    /// Current potential energy in kcal/mol.
    pub potential_energy: f64,
    /// Instantaneous temperature in K derived from kinetic energy.
    pub temperature: f64,
}

// ============================================================================
// Trajectory output
// ============================================================================

/// A single saved frame from a trajectory.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct TrajectoryFrame {
    /// Step index at which this frame was saved.
    pub step: usize,
    /// Simulation time in fs.
    pub time: f64,
    /// Atom positions in Å at this frame.
    pub positions: Vec<[f64; 3]>,
    /// Total energy (kinetic + potential) in kcal/mol.
    pub energy: f64,
    /// Instantaneous temperature in K.
    pub temperature: f64,
}

/// Result of a complete MD simulation run.
#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct SimulationResult {
    /// Saved trajectory frames.
    pub frames: Vec<TrajectoryFrame>,
    /// Total number of steps completed.
    pub total_steps: usize,
    /// Final total energy in kcal/mol.
    pub final_energy: f64,
}

// ============================================================================
// Minimal xorshift64 PRNG (deterministic, no external deps)
// ============================================================================

/// Minimal xorshift64 pseudo-random number generator.
///
/// Produces a deterministic sequence from a nonzero 64-bit seed.
/// Period: 2^64 - 1. Based on Marsaglia (2003).
struct Xorshift64 {
    state: u64,
}

impl Xorshift64 {
    /// Create a new PRNG with the given nonzero seed.
    /// If `seed` is zero, uses 1 to avoid the degenerate zero state.
    fn new(seed: u64) -> Self {
        Self { state: if seed == 0 { 1 } else { seed } }
    }

    /// Advance the state and return the next pseudo-random `u64`.
    fn next_u64(&mut self) -> u64 {
        self.state ^= self.state << 13;
        self.state ^= self.state >> 7;
        self.state ^= self.state << 17;
        self.state
    }

    /// Return a pseudo-random `f64` uniformly distributed in (0, 1).
    fn next_f64(&mut self) -> f64 {
        // Use the top 53 bits for a full-precision mantissa.
        let bits = self.next_u64() >> 11;
        // 2^-53 ≈ 1.11e-16; offset by 0.5 ulp to avoid exactly 0.
        (bits as f64 + 0.5) * (1.0_f64 / (1u64 << 53) as f64)
    }
}

// ============================================================================
// Physics helpers
// ============================================================================

/// Boltzmann constant in kcal/mol/K.
/// k_B = 1.380649e-23 J/K × 6.02214076e23 mol^-1 / 4184 J/kcal
const K_BOLTZMANN: f64 = 1.987_204_258e-3; // kcal/mol/K

/// Unit conversion: kcal/mol → (Å/fs)² for mass in g/mol.
/// E[kcal/mol] = 0.5 * m[g/mol] * v²[Å/fs²] × CONV
/// CONV = 1 kcal / (1 g/mol × 1 Å²/fs²)
///      = 4184 J/kcal / (1e-3 kg/g × 1e20 Å²/m² / 1e30 fs²/s²)
///      = 4184 / (1e-3 × 1e-10) = 4184 / 1e-13 ≈ ... simplified:
/// In AKMA units: 1 kcal/mol = 418.4 g/mol × Å²/ps²
/// Converting ps² → fs²: 1 ps = 1000 fs, so 1 Å²/ps² = 1e-6 Å²/fs²
/// Therefore: 1 kcal/mol = 418.4e-6 g/mol × Å²/fs² — i.e. CONV = 418.4e-6.
/// Invert to get v² from E/m: v²[Å²/fs²] = E[kcal/mol] / (m[g/mol] × 418.4e-6)
const KCAL_TO_AKMA: f64 = 418.4e-6; // (Å/fs)² per kcal/mol per (g/mol)

/// Compute the kinetic energy from velocities and masses.
///
/// KE = 0.5 * sum_i m_i * v_i^2  (kcal/mol)
#[must_use]
pub fn compute_kinetic_energy(velocities: &[[f64; 3]], masses: &[f64]) -> f64 {
    velocities
        .iter()
        .zip(masses.iter())
        .map(|(&[vx, vy, vz], &m)| {
            // v^2 in Å²/fs², m in g/mol, KE in kcal/mol via AKMA factor
            0.5 * m * (vx * vx + vy * vy + vz * vz) / KCAL_TO_AKMA
        })
        .sum()
}

/// Compute the instantaneous temperature from kinetic energy via the equipartition theorem.
///
/// T = 2 * KE / (3 * N * k_B)
///
/// For a system of N atoms with 3 degrees of freedom each.
#[must_use]
pub fn compute_temperature(kinetic_energy: f64, n_atoms: usize) -> f64 {
    if n_atoms == 0 {
        return 0.0;
    }
    let dof = 3.0 * n_atoms as f64;
    2.0 * kinetic_energy / (dof * K_BOLTZMANN)
}

// ============================================================================
// Maxwell-Boltzmann velocity initialization
// ============================================================================

/// Initialize atomic velocities from the Maxwell-Boltzmann distribution.
///
/// Uses the Box-Muller transform with an inline xorshift64 PRNG seeded at
/// `0xDEAD_BEEF_CAFE_1337` for reproducibility. Each velocity component is
/// drawn from a Gaussian with zero mean and variance k_B * T / m.
///
/// The velocities are shifted to remove net centre-of-mass momentum.
///
/// # Parameters
///
/// - `n_atoms`: number of atoms
/// - `temperature`: target temperature in K
/// - `masses`: per-atom masses in g/mol (daltons)
///
/// Returns a `Vec<[f64; 3]>` of velocities in Å/fs.
#[must_use]
pub fn initialize_velocities(n_atoms: usize, temperature: f64, masses: &[f64]) -> Vec<[f64; 3]> {
    if n_atoms == 0 || temperature <= 0.0 {
        return vec![[0.0; 3]; n_atoms];
    }

    let mut rng = Xorshift64::new(0xDEAD_BEEF_CAFE_1337_u64);
    let mut velocities = Vec::with_capacity(n_atoms);

    for i in 0..n_atoms {
        let mass = masses.get(i).copied().unwrap_or(12.0).max(0.001);
        // Thermal velocity standard deviation: sigma = sqrt(k_B * T / m) in Å/fs
        let sigma = (K_BOLTZMANN * temperature * KCAL_TO_AKMA / mass).sqrt();

        // Box-Muller: two uniform samples → two independent Gaussian samples
        let [vx, vy] = box_muller(rng.next_f64(), rng.next_f64(), sigma);
        let [vz, _] = box_muller(rng.next_f64(), rng.next_f64(), sigma);
        velocities.push([vx, vy, vz]);
    }

    // Remove net centre-of-mass momentum: v_cm = sum(m_i * v_i) / sum(m_i)
    remove_com_momentum(&mut velocities, masses);

    velocities
}

/// Box-Muller transform: two independent uniform(0,1) → two independent N(0, sigma²).
#[inline]
fn box_muller(u1: f64, u2: f64, sigma: f64) -> [f64; 2] {
    let r = (-2.0 * u1.ln()).sqrt();
    let theta = std::f64::consts::TAU * u2;
    [sigma * r * theta.cos(), sigma * r * theta.sin()]
}

/// Subtract the mass-weighted centre-of-mass velocity from all velocities.
fn remove_com_momentum(velocities: &mut [[f64; 3]], masses: &[f64]) {
    let total_mass: f64 = masses.iter().sum();
    if total_mass < 1e-30 {
        return;
    }

    let mut v_com = [0.0_f64; 3];
    for (vel, &m) in velocities.iter().zip(masses.iter()) {
        v_com[0] += m * vel[0];
        v_com[1] += m * vel[1];
        v_com[2] += m * vel[2];
    }
    v_com[0] /= total_mass;
    v_com[1] /= total_mass;
    v_com[2] /= total_mass;

    for vel in velocities.iter_mut() {
        vel[0] -= v_com[0];
        vel[1] -= v_com[1];
        vel[2] -= v_com[2];
    }
}

// ============================================================================
// Berendsen thermostat
// ============================================================================

/// Apply the Berendsen velocity-rescaling thermostat.
///
/// Rescales velocities by a factor lambda = sqrt(1 + (dt/tau) * (T0/T - 1)),
/// where dt/tau is approximated by `coupling` (the fraction of the temperature
/// difference corrected each step). When `coupling = 1`, this is full
/// instantaneous rescaling.
///
/// Updates `state.velocities`, `state.kinetic_energy`, and `state.temperature`
/// in place.
pub fn berendsen_thermostat(state: &mut SimulationState, target_temp: f64, coupling: f64) {
    let current_temp = state.temperature;
    if current_temp < 1e-10 {
        return;
    }

    // Berendsen lambda: lambda² = 1 + coupling * (T_target/T_current - 1)
    let ratio = target_temp / current_temp;
    let lambda_sq = (1.0 + coupling * (ratio - 1.0)).max(0.0);
    let lambda = lambda_sq.sqrt();

    for vel in state.velocities.iter_mut() {
        vel[0] *= lambda;
        vel[1] *= lambda;
        vel[2] *= lambda;
    }

    // Kinetic energy and temperature scale by lambda²
    state.kinetic_energy *= lambda_sq;
    state.temperature = compute_temperature(
        state.kinetic_energy,
        state.positions.len(),
    );
}

// ============================================================================
// Velocity Verlet integrator
// ============================================================================

/// Perform one velocity Verlet integration step.
///
/// The velocity Verlet algorithm:
/// 1. Half-kick velocities:   v(t + dt/2) = v(t) + 0.5 * F(t)/m * dt
/// 2. Propagate positions:    r(t + dt)   = r(t) + v(t + dt/2) * dt
/// 3. Recompute forces:       F(t + dt)
/// 4. Finish velocity kick:   v(t + dt)   = v(t + dt/2) + 0.5 * F(t+dt)/m * dt
///
/// All units are AKMA: Å, fs, g/mol, kcal/mol.
///
/// # Errors
///
/// Returns [`DynamicsError::ForceField`] if force computation fails.
pub fn velocity_verlet_step(
    state: &mut SimulationState,
    mol: &mut Molecule,
    masses: &[f64],
    ff_config: &ForceFieldConfig,
) -> Result<(), DynamicsError> {
    let dt = ff_config.gradient_step; // We'll pass dt via a wrapper; see run_simulation
    // Note: this function receives dt embedded in the calling context.
    // We use a local dt variable passed through the state for correctness.
    // The actual dt comes from DynamicsConfig.timestep injected by run_simulation.
    // For direct callers of this function, dt = ff_config.gradient_step is a placeholder;
    // prefer calling through run_simulation.
    let n = state.positions.len();

    // Step 1: half-kick + position update
    for i in 0..n {
        let m = masses.get(i).copied().unwrap_or(12.0).max(1e-30);
        // Acceleration a = F / m, unit: (kcal/mol/Å) / (g/mol) × KCAL_TO_AKMA → Å/fs²
        let ax = state.forces[i][0] / m * KCAL_TO_AKMA;
        let ay = state.forces[i][1] / m * KCAL_TO_AKMA;
        let az = state.forces[i][2] / m * KCAL_TO_AKMA;

        // Half-kick: v += 0.5 * a * dt
        state.velocities[i][0] += 0.5 * ax * dt;
        state.velocities[i][1] += 0.5 * ay * dt;
        state.velocities[i][2] += 0.5 * az * dt;

        // Position update: r += v * dt
        state.positions[i][0] += state.velocities[i][0] * dt;
        state.positions[i][1] += state.velocities[i][1] * dt;
        state.positions[i][2] += state.velocities[i][2] * dt;

        // Sync molecule positions
        if let Some(atom) = mol.atoms.get_mut(i) {
            atom.position = state.positions[i];
        }
    }

    // Step 2: recompute forces at new positions
    let new_forces = compute_forces(mol, ff_config)?;
    state.forces = new_forces.clone();

    // Step 3: finish velocity kick
    for i in 0..n {
        let m = masses.get(i).copied().unwrap_or(12.0).max(1e-30);
        let ax = new_forces[i][0] / m * KCAL_TO_AKMA;
        let ay = new_forces[i][1] / m * KCAL_TO_AKMA;
        let az = new_forces[i][2] / m * KCAL_TO_AKMA;

        state.velocities[i][0] += 0.5 * ax * dt;
        state.velocities[i][1] += 0.5 * ay * dt;
        state.velocities[i][2] += 0.5 * az * dt;
    }

    Ok(())
}

// ============================================================================
// Full simulation runner
// ============================================================================

/// Run a complete NVT molecular dynamics simulation.
///
/// Performs the following sequence:
/// 1. Validate configuration and molecule.
/// 2. Build per-atom mass array from element data.
/// 3. Initialize velocities from Maxwell-Boltzmann distribution.
/// 4. Compute initial forces.
/// 5. Integrate with velocity Verlet for `config.total_steps` steps.
/// 6. Apply Berendsen thermostat after each step.
/// 7. Save trajectory frames every `config.save_interval` steps.
///
/// Returns a [`SimulationResult`] with the trajectory and final energy.
///
/// # Errors
///
/// Returns [`DynamicsError::EmptyMolecule`] if the molecule has no atoms,
/// [`DynamicsError::InvalidTimestep`] if `config.timestep` is not positive and
/// finite, or propagates [`DynamicsError::ForceField`] on force field errors.
pub fn run_simulation(
    mol: &mut Molecule,
    config: &DynamicsConfig,
    ff_config: &ForceFieldConfig,
) -> Result<SimulationResult, DynamicsError> {
    // Validate
    if mol.atoms.is_empty() {
        return Err(DynamicsError::EmptyMolecule);
    }
    if !config.timestep.is_finite() || config.timestep <= 0.0 {
        return Err(DynamicsError::InvalidTimestep { timestep: config.timestep });
    }

    let n = mol.atoms.len();
    let dt = config.timestep;

    // Build mass array from element atomic masses
    let masses: Vec<f64> = mol.atoms.iter().map(|a| {
        let m = a.element.atomic_mass();
        if m < 0.001 { 12.0 } else { m } // fallback to carbon mass for unknown
    }).collect();

    // Initialize velocities from Maxwell-Boltzmann at target temperature
    let velocities = initialize_velocities(n, config.temperature, &masses);

    // Compute initial forces
    let initial_forces = compute_forces(mol, ff_config)?;
    let initial_potential = compute_energy(mol, ff_config)?.total;
    let initial_ke = compute_kinetic_energy(&velocities, &masses);
    let initial_temp = compute_temperature(initial_ke, n);

    // Build initial state
    let positions: Vec<[f64; 3]> = mol.atoms.iter().map(|a| a.position).collect();
    let mut state = SimulationState {
        positions,
        velocities,
        forces: initial_forces,
        step: 0,
        time: 0.0,
        kinetic_energy: initial_ke,
        potential_energy: initial_potential,
        temperature: initial_temp,
    };

    let mut frames = Vec::new();
    let save_interval = config.save_interval.max(1);

    // Save frame 0 if requested
    if config.total_steps > 0 {
        let frame0 = TrajectoryFrame {
            step: 0,
            time: 0.0,
            positions: state.positions.clone(),
            energy: state.kinetic_energy + state.potential_energy,
            temperature: state.temperature,
        };
        frames.push(frame0);
    }

    // Main MD loop
    for step in 1..=config.total_steps {
        // Build a temporary ForceFieldConfig with dt embedded via a custom wrapper.
        // velocity_verlet_step uses ff_config.gradient_step as dt; we clone and override.
        let mut step_ff = ff_config.clone();
        step_ff.gradient_step = dt;

        velocity_verlet_step(&mut state, mol, &masses, &step_ff)?;

        // Update kinetic energy and temperature
        state.kinetic_energy = compute_kinetic_energy(&state.velocities, &masses);
        state.temperature = compute_temperature(state.kinetic_energy, n);

        // Berendsen thermostat
        berendsen_thermostat(&mut state, config.temperature, config.friction);

        // Recompute potential energy
        state.potential_energy = compute_energy(mol, ff_config)?.total;
        state.step = step;
        state.time = step as f64 * dt;

        // Save trajectory frame
        if step % save_interval == 0 {
            frames.push(TrajectoryFrame {
                step,
                time: state.time,
                positions: state.positions.clone(),
                energy: state.kinetic_energy + state.potential_energy,
                temperature: state.temperature,
            });
        }
    }

    let final_energy = state.kinetic_energy + state.potential_energy;

    Ok(SimulationResult {
        frames,
        total_steps: config.total_steps,
        final_energy,
    })
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;
    use crate::force_field::ForceFieldConfig;
    use crate::molecular::{Atom, Bond, BondOrder, Element, Molecule};

    fn water() -> Molecule {
        let mut mol = Molecule::new("Water");
        mol.atoms.push(Atom::new(1, Element::O, [0.0, 0.0, 0.0]));
        mol.atoms.push(Atom::new(2, Element::H, [0.96, 0.0, 0.0]));
        mol.atoms.push(Atom::new(3, Element::H, [-0.24, 0.93, 0.0]));
        mol.bonds.push(Bond { atom1: 0, atom2: 1, order: BondOrder::Single });
        mol.bonds.push(Bond { atom1: 0, atom2: 2, order: BondOrder::Single });
        mol
    }

    fn masses_water() -> Vec<f64> {
        vec![
            Element::O.atomic_mass(),
            Element::H.atomic_mass(),
            Element::H.atomic_mass(),
        ]
    }

    fn short_config() -> DynamicsConfig {
        DynamicsConfig {
            timestep: 0.5,
            temperature: 300.0,
            friction: 0.1,
            total_steps: 20,
            save_interval: 5,
        }
    }

    #[test]
    fn xorshift64_produces_distinct_values() {
        let mut rng = Xorshift64::new(42);
        let a = rng.next_u64();
        let b = rng.next_u64();
        assert_ne!(a, b, "consecutive xorshift64 outputs should differ");
    }

    #[test]
    fn xorshift64_f64_in_unit_interval() {
        let mut rng = Xorshift64::new(99);
        for _ in 0..1000 {
            let v = rng.next_f64();
            assert!(v > 0.0 && v < 1.0, "xorshift64 f64 out of (0,1): {v}");
        }
    }

    #[test]
    fn initialize_velocities_correct_count() {
        let masses = masses_water();
        let vels = initialize_velocities(3, 300.0, &masses);
        assert_eq!(vels.len(), 3, "should have one velocity per atom");
    }

    #[test]
    fn initialize_velocities_are_finite() {
        let masses = masses_water();
        let vels = initialize_velocities(3, 300.0, &masses);
        for (i, v) in vels.iter().enumerate() {
            assert!(
                v.iter().all(|x| x.is_finite()),
                "velocity of atom {i} is not finite: {v:?}"
            );
        }
    }

    #[test]
    fn initialize_velocities_zero_temperature_gives_zero() {
        let masses = masses_water();
        let vels = initialize_velocities(3, 0.0, &masses);
        for v in &vels {
            assert_eq!(*v, [0.0; 3], "zero temperature should give zero velocities");
        }
    }

    #[test]
    fn compute_kinetic_energy_positive() {
        let masses = masses_water();
        let velocities = vec![[0.01, 0.0, 0.0], [0.0, 0.01, 0.0], [0.0, 0.0, 0.01]];
        let ke = compute_kinetic_energy(&velocities, &masses);
        assert!(ke > 0.0, "kinetic energy must be positive for moving atoms: {ke}");
        assert!(ke.is_finite(), "kinetic energy must be finite: {ke}");
    }

    #[test]
    fn compute_temperature_from_equipartition() {
        // At 300 K with N atoms: KE = 1.5 * N * k_B * T
        let n = 100_usize;
        let target_temp = 300.0;
        let ke = 1.5 * n as f64 * K_BOLTZMANN * target_temp;
        let computed_temp = compute_temperature(ke, n);
        assert!(
            (computed_temp - target_temp).abs() < 0.01,
            "temperature should be {target_temp}, got {computed_temp}"
        );
    }

    #[test]
    fn berendsen_thermostat_moves_toward_target() {
        let masses = masses_water();
        // Set velocities explicitly to guarantee a very high kinetic energy.
        // v = 1.0 Å/fs for all components; at these speeds T >> 300 K.
        let high_vels: Vec<[f64; 3]> = vec![[1.0, 1.0, 1.0]; 3];
        let ke = compute_kinetic_energy(&high_vels, &masses);
        let temp_before = compute_temperature(ke, 3);

        // Sanity: confirm this velocity gives T >> 300 K
        assert!(temp_before > 300.0, "test setup: temp should be >300 K, got {temp_before:.1}");

        let mut state = SimulationState {
            positions: vec![[0.0; 3]; 3],
            velocities: high_vels,
            forces: vec![[0.0; 3]; 3],
            step: 0,
            time: 0.0,
            kinetic_energy: ke,
            potential_energy: 0.0,
            temperature: temp_before,
        };

        berendsen_thermostat(&mut state, 300.0, 0.5);

        // After cooling, temperature must be strictly lower than before
        assert!(
            state.temperature < temp_before,
            "thermostat should cool the system: {temp_before:.1} -> {:.1}",
            state.temperature
        );
        // And it should be moving toward 300 K (not below it in one step)
        assert!(
            state.temperature > 300.0,
            "single thermostat step should not overshoot to <300 K: {:.1}",
            state.temperature
        );
    }

    #[test]
    fn run_simulation_returns_frames() {
        let mut mol = water();
        let config = short_config();
        let ff_config = ForceFieldConfig::default();
        match run_simulation(&mut mol, &config, &ff_config) {
            Ok(result) => {
                assert!(!result.frames.is_empty(), "simulation should produce frames");
                assert_eq!(result.total_steps, config.total_steps);
                assert!(result.final_energy.is_finite(), "final energy must be finite");
            }
            Err(e) => panic!("simulation failed unexpectedly: {e}"),
        }
    }

    #[test]
    fn run_simulation_empty_molecule_errors() {
        let mut mol = Molecule::new("Empty");
        let config = short_config();
        let ff_config = ForceFieldConfig::default();
        let result = run_simulation(&mut mol, &config, &ff_config);
        assert!(matches!(result, Err(DynamicsError::EmptyMolecule)));
    }

    #[test]
    fn run_simulation_invalid_timestep_errors() {
        let mut mol = water();
        let config = DynamicsConfig { timestep: -1.0, ..short_config() };
        let ff_config = ForceFieldConfig::default();
        let result = run_simulation(&mut mol, &config, &ff_config);
        assert!(
            matches!(result, Err(DynamicsError::InvalidTimestep { .. })),
            "negative timestep should return InvalidTimestep error"
        );
    }

    #[test]
    fn dynamics_error_display() {
        let e = DynamicsError::InvalidTimestep { timestep: -0.5 };
        let s = e.to_string();
        assert!(s.contains("-0.5"), "display should mention the bad timestep");
        assert!(s.contains("positive"), "display should explain the constraint");
    }
}
