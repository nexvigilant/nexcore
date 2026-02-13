//! 3D Voxel Cube: the ISA as a 4×4×4 chemical structure.
//!
//! Maps all 64 instructions into a 3D cube where each axis corresponds
//! to a chemical property derived from the amino acid classification:
//!
//! - **X axis (0-3): Charge** — electrochemical (Nernst potential)
//! - **Y axis (0-3): Energy** — activation threshold (Arrhenius rate)
//! - **Z axis (0-3): Stability** — equilibrium constant (Keq)
//!
//! Index = x + 4y + 16z (bijective 0-63)
//!
//! ## Beer-Lambert Projection
//!
//! `A = Σ(ε_i × c_i)` along any axis yields a 4×4 absorbance image
//! representing computational cost from three orthogonal perspectives.
//!
//! ## Color
//!
//! RGB = f(charge, energy, stability) — colors emerge from position:
//! - R = energy (high energy = more red)
//! - G = stability_inverse (volatile = more green)
//! - B = charge (basic/positive = more blue)
//!
//! Tier: T3 (σ + μ + ∂ + N + λ + κ + → + ∃)

use crate::isa::{self, Instruction};

// ---------------------------------------------------------------------------
// Chemical Axes
// ---------------------------------------------------------------------------

/// Charge state of an instruction (X axis).
///
/// Derived from amino acid chemical property:
/// acidic → negative, polar → neutral-polar,
/// nonpolar → neutral-nonpolar, basic → positive.
///
/// Tier: T1 (κ Comparison)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum ChargeState {
    /// Acidic amino acids: observation ops (Asp, Glu).
    Negative = 0,
    /// Polar amino acids: transformation ops (Ser, Thr, Cys, Tyr, Asn, Gln).
    NeutralPolar = 1,
    /// Nonpolar amino acids: data ops (Gly, Ala, Val, Ile, Phe, Pro, Trp).
    NeutralNonpolar = 2,
    /// Basic amino acids + control: control ops (Lys, Arg, His, Leu, Stop, Met).
    Positive = 3,
}

/// Energy level of an instruction (Y axis).
///
/// Derived from Arrhenius activation: how much state change does this
/// instruction cause?
///
/// Tier: T1 (N Quantity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum EnergyLevel {
    /// Ground state: constants, nop, stack inspection.
    Ground = 0,
    /// Activated: arithmetic, logic, comparison.
    Activated = 1,
    /// Excited: control flow, jumps, calls, returns.
    Excited = 2,
    /// Ionized: lifecycle transitions (entry, halt, yield).
    Ionized = 3,
}

/// Stability of an instruction (Z axis).
///
/// Derived from equilibrium: how predictable is the outcome?
///
/// Tier: T1 (∂ Boundary)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum StabilityClass {
    /// Deterministic, no side effects. Keq → ∞.
    Stable = 0,
    /// Conditional: outcome depends on input state.
    Metastable = 1,
    /// State-modifying: writes to memory or accumulator.
    Reactive = 2,
    /// Side-effecting: I/O, lifecycle transitions.
    Volatile = 3,
}

/// A position in the 4×4×4 voxel cube.
///
/// Tier: T2-P (λ Location + N Quantity)
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct VoxelPos {
    /// X: charge state (0-3).
    pub x: u8,
    /// Y: energy level (0-3).
    pub y: u8,
    /// Z: stability class (0-3).
    pub z: u8,
}

impl VoxelPos {
    /// Flatten to a linear index: x + 4y + 16z (0-63).
    #[must_use]
    pub fn index(&self) -> u8 {
        self.x + 4 * self.y + 16 * self.z
    }

    /// Create from a linear index.
    #[must_use]
    pub fn from_index(idx: u8) -> Self {
        Self {
            x: idx % 4,
            y: (idx / 4) % 4,
            z: idx / 16,
        }
    }

    /// RGB color from 3D position.
    ///
    /// R = energy level (high energy = more red).
    /// G = stability inverse (volatile = more green).
    /// B = charge (positive = more blue).
    #[must_use]
    pub fn color(&self) -> (u8, u8, u8) {
        let r = self.y * 85; // 0, 85, 170, 255
        let g = self.z * 85;
        let b = self.x * 85;
        (r, g, b)
    }

    /// Molar absorptivity (ε) for Beer-Lambert projection.
    ///
    /// Higher energy + lower stability = higher absorbance.
    #[must_use]
    pub fn absorptivity(&self) -> f64 {
        let energy = f64::from(self.y) / 3.0; // 0.0 - 1.0
        let instability = f64::from(self.z) / 3.0; // 0.0 - 1.0
        0.5 * energy + 0.5 * instability // weighted average
    }
}

// ---------------------------------------------------------------------------
// Classification
// ---------------------------------------------------------------------------

/// Classify an instruction's charge state from its amino acid family.
#[must_use]
pub fn charge_of(instr: &Instruction) -> ChargeState {
    match instr {
        // Acidic (Asp, Glu): observation
        Instruction::Peek | Instruction::IsEmpty | Instruction::CntInc | Instruction::CntRead => {
            ChargeState::Negative
        }

        // Polar (Ser, Thr, Cys, Tyr, Asn, Gln): transformation
        Instruction::Mod
        | Instruction::Abs
        | Instruction::Neg
        | Instruction::Inc
        | Instruction::Dec
        | Instruction::Max
        | Instruction::Rot
        | Instruction::Over
        | Instruction::Pick
        | Instruction::Depth
        | Instruction::IfElse
        | Instruction::Assert
        | Instruction::Dup2
        | Instruction::Cmp
        | Instruction::Shl
        | Instruction::Shr
        | Instruction::And
        | Instruction::Or => ChargeState::NeutralPolar,

        // Nonpolar (Gly, Ala, Val, Ile, Phe, Pro, Trp): data
        Instruction::Nop
        | Instruction::Dup
        | Instruction::Swap
        | Instruction::Pop
        | Instruction::Add
        | Instruction::Sub
        | Instruction::Mul
        | Instruction::Div
        | Instruction::Eq
        | Instruction::Lt
        | Instruction::Gt
        | Instruction::Neq
        | Instruction::Push0
        | Instruction::Push1
        | Instruction::PushNeg1
        | Instruction::Load
        | Instruction::Store
        | Instruction::BitAnd
        | Instruction::BitOr
        | Instruction::BitXor
        | Instruction::BitNot
        | Instruction::Output => ChargeState::NeutralNonpolar,

        // Basic (Lys, Arg, His, Leu) + lifecycle (Stop, Met): control
        Instruction::PushAcc
        | Instruction::StoreAcc
        | Instruction::Sign
        | Instruction::Clamp
        | Instruction::Min
        | Instruction::Pow
        | Instruction::Sqrt
        | Instruction::Log2
        | Instruction::MemSize
        | Instruction::MemClear
        | Instruction::Jmp
        | Instruction::JmpIf
        | Instruction::JmpBack
        | Instruction::Call
        | Instruction::Ret
        | Instruction::JmpIfZ
        | Instruction::Halt
        | Instruction::HaltErr
        | Instruction::HaltYield
        | Instruction::Entry => ChargeState::Positive,

        Instruction::Lit(_) => ChargeState::NeutralNonpolar,
    }
}

/// Classify an instruction's energy level.
#[must_use]
pub fn energy_of(instr: &Instruction) -> EnergyLevel {
    match instr {
        // Ground: constants, nop, stack inspection, load
        Instruction::Nop
        | Instruction::Push0
        | Instruction::Push1
        | Instruction::PushNeg1
        | Instruction::PushAcc
        | Instruction::Dup
        | Instruction::Dup2
        | Instruction::Swap
        | Instruction::Pop
        | Instruction::Rot
        | Instruction::Over
        | Instruction::Pick
        | Instruction::Depth
        | Instruction::Peek
        | Instruction::IsEmpty
        | Instruction::Load
        | Instruction::CntRead => EnergyLevel::Ground,

        // Activated: arithmetic, logic, comparison, math, bitwise
        Instruction::Add
        | Instruction::Sub
        | Instruction::Mul
        | Instruction::Div
        | Instruction::Mod
        | Instruction::Abs
        | Instruction::Neg
        | Instruction::Inc
        | Instruction::Dec
        | Instruction::Sign
        | Instruction::Max
        | Instruction::Min
        | Instruction::Pow
        | Instruction::Sqrt
        | Instruction::Log2
        | Instruction::Clamp
        | Instruction::Eq
        | Instruction::Lt
        | Instruction::Gt
        | Instruction::Neq
        | Instruction::Cmp
        | Instruction::And
        | Instruction::Or
        | Instruction::Shl
        | Instruction::Shr
        | Instruction::BitAnd
        | Instruction::BitOr
        | Instruction::BitXor
        | Instruction::BitNot => EnergyLevel::Activated,

        // Excited: control flow, state mutation
        Instruction::Jmp
        | Instruction::JmpIf
        | Instruction::JmpBack
        | Instruction::JmpIfZ
        | Instruction::Call
        | Instruction::Ret
        | Instruction::IfElse
        | Instruction::Store
        | Instruction::StoreAcc
        | Instruction::CntInc
        | Instruction::MemClear => EnergyLevel::Excited,

        // Ionized: lifecycle transitions
        Instruction::Entry
        | Instruction::Halt
        | Instruction::HaltErr
        | Instruction::HaltYield
        | Instruction::Output
        | Instruction::Assert
        | Instruction::MemSize => EnergyLevel::Ionized,

        Instruction::Lit(_) => EnergyLevel::Ground,
    }
}

/// Classify an instruction's stability.
#[must_use]
pub fn stability_of(instr: &Instruction) -> StabilityClass {
    match instr {
        // Stable: pure functions, no state dependency
        Instruction::Nop
        | Instruction::Push0
        | Instruction::Push1
        | Instruction::PushNeg1
        | Instruction::Add
        | Instruction::Sub
        | Instruction::Mul
        | Instruction::Div
        | Instruction::Mod
        | Instruction::Abs
        | Instruction::Neg
        | Instruction::Inc
        | Instruction::Dec
        | Instruction::Sign
        | Instruction::Max
        | Instruction::Min
        | Instruction::Pow
        | Instruction::Sqrt
        | Instruction::Log2
        | Instruction::Clamp
        | Instruction::Dup
        | Instruction::Dup2
        | Instruction::Swap
        | Instruction::Pop
        | Instruction::Rot
        | Instruction::Over
        | Instruction::Depth
        | Instruction::Shl
        | Instruction::Shr
        | Instruction::BitAnd
        | Instruction::BitOr
        | Instruction::BitXor
        | Instruction::BitNot => StabilityClass::Stable,

        // Metastable: outcome depends on runtime state
        Instruction::Eq
        | Instruction::Lt
        | Instruction::Gt
        | Instruction::Neq
        | Instruction::Cmp
        | Instruction::And
        | Instruction::Or
        | Instruction::Peek
        | Instruction::IsEmpty
        | Instruction::Pick
        | Instruction::Load
        | Instruction::PushAcc
        | Instruction::CntRead
        | Instruction::MemSize
        | Instruction::IfElse => StabilityClass::Metastable,

        // Reactive: writes to memory/state
        Instruction::Store
        | Instruction::StoreAcc
        | Instruction::CntInc
        | Instruction::MemClear
        | Instruction::Jmp
        | Instruction::JmpIf
        | Instruction::JmpBack
        | Instruction::JmpIfZ
        | Instruction::Call
        | Instruction::Ret => StabilityClass::Reactive,

        // Volatile: side effects, lifecycle
        Instruction::Output
        | Instruction::Assert
        | Instruction::Entry
        | Instruction::Halt
        | Instruction::HaltErr
        | Instruction::HaltYield => StabilityClass::Volatile,

        Instruction::Lit(_) => StabilityClass::Stable,
    }
}

/// Get the voxel position for an instruction based on chemical classification.
#[must_use]
pub fn classify(instr: &Instruction) -> VoxelPos {
    VoxelPos {
        x: charge_of(instr) as u8,
        y: energy_of(instr) as u8,
        z: stability_of(instr) as u8,
    }
}

// ---------------------------------------------------------------------------
// VoxelCube
// ---------------------------------------------------------------------------

/// The 4×4×4 voxel cube mapping instructions to 3D chemical space.
///
/// Each voxel holds an instruction count (concentration) for Beer-Lambert.
///
/// Tier: T3 (σ + μ + ∂ + N + λ + κ + → + ∃)
#[derive(Debug, Clone)]
pub struct VoxelCube {
    /// Instruction count at each voxel position.
    /// Index = x + 4y + 16z (0-63).
    concentrations: [f64; 64],
}

impl VoxelCube {
    /// Create an empty cube (all concentrations zero).
    #[must_use]
    pub fn empty() -> Self {
        Self {
            concentrations: [0.0; 64],
        }
    }

    /// Build a cube from a sequence of instructions.
    ///
    /// Each instruction is classified into (charge, energy, stability) and
    /// its voxel's concentration is incremented.
    #[must_use]
    pub fn from_instructions(instrs: &[Instruction]) -> Self {
        let mut cube = Self::empty();
        for instr in instrs {
            let pos = classify(instr);
            let idx = pos.index() as usize;
            if idx < 64 {
                cube.concentrations[idx] += 1.0;
            }
        }
        cube
    }

    /// Get the concentration at a voxel position.
    #[must_use]
    pub fn concentration_at(&self, pos: VoxelPos) -> f64 {
        let idx = pos.index() as usize;
        if idx < 64 {
            self.concentrations[idx]
        } else {
            0.0
        }
    }

    /// Total concentration (total instruction count).
    #[must_use]
    pub fn total_concentration(&self) -> f64 {
        self.concentrations.iter().sum()
    }

    /// Beer-Lambert projection along the X axis.
    ///
    /// Returns a 4×4 array: `[y][z] = Σ(ε(x,y,z) × c(x,y,z))` for x=0..3.
    ///
    /// Shows the energy × stability profile of the program.
    #[must_use]
    pub fn project_x(&self) -> [[f64; 4]; 4] {
        let mut image = [[0.0f64; 4]; 4];
        for y in 0..4u8 {
            for z in 0..4u8 {
                let mut absorbance = 0.0;
                for x in 0..4u8 {
                    let pos = VoxelPos { x, y, z };
                    let eps = pos.absorptivity();
                    let c = self.concentration_at(pos);
                    absorbance += eps * c;
                }
                image[y as usize][z as usize] = absorbance;
            }
        }
        image
    }

    /// Beer-Lambert projection along the Y axis.
    ///
    /// Returns a 4×4 array: `[x][z]` — charge × stability profile.
    #[must_use]
    pub fn project_y(&self) -> [[f64; 4]; 4] {
        let mut image = [[0.0f64; 4]; 4];
        for x in 0..4u8 {
            for z in 0..4u8 {
                let mut absorbance = 0.0;
                for y in 0..4u8 {
                    let pos = VoxelPos { x, y, z };
                    let eps = pos.absorptivity();
                    let c = self.concentration_at(pos);
                    absorbance += eps * c;
                }
                image[x as usize][z as usize] = absorbance;
            }
        }
        image
    }

    /// Beer-Lambert projection along the Z axis.
    ///
    /// Returns a 4×4 array: `[x][y]` — charge × energy profile.
    #[must_use]
    pub fn project_z(&self) -> [[f64; 4]; 4] {
        let mut image = [[0.0f64; 4]; 4];
        for x in 0..4u8 {
            for y in 0..4u8 {
                let mut absorbance = 0.0;
                for z in 0..4u8 {
                    let pos = VoxelPos { x, y, z };
                    let eps = pos.absorptivity();
                    let c = self.concentration_at(pos);
                    absorbance += eps * c;
                }
                image[x as usize][y as usize] = absorbance;
            }
        }
        image
    }

    /// Total absorbance across the entire cube (program complexity metric).
    #[must_use]
    pub fn total_absorbance(&self) -> f64 {
        let mut total = 0.0;
        for idx in 0..64u8 {
            let pos = VoxelPos::from_index(idx);
            total += pos.absorptivity() * self.concentrations[idx as usize];
        }
        total
    }

    /// Charge distribution: count of instructions per charge state.
    #[must_use]
    pub fn charge_distribution(&self) -> [f64; 4] {
        let mut dist = [0.0f64; 4];
        for idx in 0..64u8 {
            let pos = VoxelPos::from_index(idx);
            dist[pos.x as usize] += self.concentrations[idx as usize];
        }
        dist
    }

    /// Energy distribution: count of instructions per energy level.
    #[must_use]
    pub fn energy_distribution(&self) -> [f64; 4] {
        let mut dist = [0.0f64; 4];
        for idx in 0..64u8 {
            let pos = VoxelPos::from_index(idx);
            dist[pos.y as usize] += self.concentrations[idx as usize];
        }
        dist
    }

    /// Stability distribution: count of instructions per stability class.
    #[must_use]
    pub fn stability_distribution(&self) -> [f64; 4] {
        let mut dist = [0.0f64; 4];
        for idx in 0..64u8 {
            let pos = VoxelPos::from_index(idx);
            dist[pos.z as usize] += self.concentrations[idx as usize];
        }
        dist
    }
}

// ---------------------------------------------------------------------------
// ISA Cube: static analysis of the full instruction set
// ---------------------------------------------------------------------------

/// Build the ISA cube: every instruction at concentration 1.0.
///
/// Useful for visualizing the instruction set's chemical structure.
#[must_use]
pub fn isa_cube() -> VoxelCube {
    let all_instrs: Vec<Instruction> = (0..64u8).map(isa::decode_index).collect();
    VoxelCube::from_instructions(&all_instrs)
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn voxel_pos_bijection() {
        // All 64 positions must round-trip through index
        for idx in 0..64u8 {
            let pos = VoxelPos::from_index(idx);
            assert_eq!(pos.index(), idx, "round-trip failed for index {idx}");
            assert!(pos.x < 4);
            assert!(pos.y < 4);
            assert!(pos.z < 4);
        }
    }

    #[test]
    fn voxel_pos_formula() {
        // index = x + 4y + 16z
        let pos = VoxelPos { x: 2, y: 3, z: 1 };
        assert_eq!(pos.index(), 2 + 4 * 3 + 16 * 1); // 2 + 12 + 16 = 30
    }

    #[test]
    fn voxel_pos_color() {
        let pos = VoxelPos { x: 0, y: 3, z: 0 };
        let (r, g, b) = pos.color();
        assert_eq!(r, 255); // max energy → max red
        assert_eq!(g, 0); // stable → min green
        assert_eq!(b, 0); // negative charge → min blue
    }

    #[test]
    fn voxel_pos_absorptivity() {
        // Ground state, stable → low absorptivity
        let low = VoxelPos { x: 0, y: 0, z: 0 };
        assert!(low.absorptivity() < 0.01);

        // Ionized, volatile → high absorptivity
        let high = VoxelPos { x: 3, y: 3, z: 3 };
        assert!((high.absorptivity() - 1.0).abs() < f64::EPSILON);
    }

    #[test]
    fn classify_nop() {
        let pos = classify(&Instruction::Nop);
        assert_eq!(pos.x, ChargeState::NeutralNonpolar as u8);
        assert_eq!(pos.y, EnergyLevel::Ground as u8);
        assert_eq!(pos.z, StabilityClass::Stable as u8);
    }

    #[test]
    fn classify_halt() {
        let pos = classify(&Instruction::Halt);
        assert_eq!(pos.x, ChargeState::Positive as u8); // lifecycle
        assert_eq!(pos.y, EnergyLevel::Ionized as u8); // lifecycle transition
        assert_eq!(pos.z, StabilityClass::Volatile as u8);
    }

    #[test]
    fn classify_add() {
        let pos = classify(&Instruction::Add);
        assert_eq!(pos.x, ChargeState::NeutralNonpolar as u8);
        assert_eq!(pos.y, EnergyLevel::Activated as u8);
        assert_eq!(pos.z, StabilityClass::Stable as u8);
    }

    #[test]
    fn classify_call() {
        let pos = classify(&Instruction::Call);
        assert_eq!(pos.x, ChargeState::Positive as u8);
        assert_eq!(pos.y, EnergyLevel::Excited as u8);
        assert_eq!(pos.z, StabilityClass::Reactive as u8);
    }

    #[test]
    fn classify_output() {
        let pos = classify(&Instruction::Output);
        assert_eq!(pos.y, EnergyLevel::Ionized as u8);
        assert_eq!(pos.z, StabilityClass::Volatile as u8);
    }

    #[test]
    fn classify_all_64_instructions() {
        // Every instruction must classify without panic
        for idx in 0..64u8 {
            let instr = isa::decode_index(idx);
            let pos = classify(&instr);
            assert!(pos.x < 4, "bad charge for {instr:?}");
            assert!(pos.y < 4, "bad energy for {instr:?}");
            assert!(pos.z < 4, "bad stability for {instr:?}");
        }
    }

    #[test]
    fn cube_from_simple_program() {
        let instrs = vec![
            Instruction::Push0,
            Instruction::Push1,
            Instruction::Add,
            Instruction::Output,
            Instruction::Halt,
        ];
        let cube = VoxelCube::from_instructions(&instrs);
        assert!((cube.total_concentration() - 5.0).abs() < f64::EPSILON);
    }

    #[test]
    fn cube_empty() {
        let cube = VoxelCube::empty();
        assert!((cube.total_concentration()).abs() < f64::EPSILON);
        assert!((cube.total_absorbance()).abs() < f64::EPSILON);
    }

    #[test]
    fn cube_beer_lambert_projection() {
        // A program with only Halt (ionized, volatile) should have
        // high absorbance in the high-energy, high-instability region
        let instrs = vec![Instruction::Halt; 10];
        let cube = VoxelCube::from_instructions(&instrs);
        let proj_x = cube.project_x();
        let proj_z = cube.project_z();

        // Total absorbance should be > 0
        assert!(cube.total_absorbance() > 0.0);

        // The Halt instruction is ionized (y=3) + volatile (z=3)
        // So projection along X should have high values at y=3, z=3
        assert!(proj_x[3][3] > 0.0);

        // And projection along Z should have non-zero values
        let total_z: f64 = proj_z.iter().flat_map(|row| row.iter()).sum();
        assert!(total_z > 0.0);
    }

    #[test]
    fn cube_charge_distribution() {
        let instrs = vec![
            Instruction::Peek,    // Negative (x=0)
            Instruction::IsEmpty, // Negative (x=0)
            Instruction::Add,     // NeutralNonpolar (x=2)
            Instruction::Call,    // Positive (x=3)
        ];
        let cube = VoxelCube::from_instructions(&instrs);
        let dist = cube.charge_distribution();
        assert!((dist[0] - 2.0).abs() < f64::EPSILON); // 2 negative
        assert!((dist[2] - 1.0).abs() < f64::EPSILON); // 1 nonpolar
        assert!((dist[3] - 1.0).abs() < f64::EPSILON); // 1 positive
    }

    #[test]
    fn cube_energy_distribution() {
        let instrs = vec![
            Instruction::Nop,   // Ground
            Instruction::Push0, // Ground
            Instruction::Add,   // Activated
            Instruction::Halt,  // Ionized
        ];
        let cube = VoxelCube::from_instructions(&instrs);
        let dist = cube.energy_distribution();
        assert!((dist[0] - 2.0).abs() < f64::EPSILON); // 2 ground
        assert!((dist[1] - 1.0).abs() < f64::EPSILON); // 1 activated
        assert!((dist[3] - 1.0).abs() < f64::EPSILON); // 1 ionized
    }

    #[test]
    fn cube_stability_distribution() {
        let instrs = vec![
            Instruction::Add,    // Stable
            Instruction::Sub,    // Stable
            Instruction::Eq,     // Metastable
            Instruction::Store,  // Reactive
            Instruction::Output, // Volatile
        ];
        let cube = VoxelCube::from_instructions(&instrs);
        let dist = cube.stability_distribution();
        assert!((dist[0] - 2.0).abs() < f64::EPSILON); // 2 stable
        assert!((dist[1] - 1.0).abs() < f64::EPSILON); // 1 metastable
        assert!((dist[2] - 1.0).abs() < f64::EPSILON); // 1 reactive
        assert!((dist[3] - 1.0).abs() < f64::EPSILON); // 1 volatile
    }

    #[test]
    fn isa_cube_has_64_instructions() {
        let cube = isa_cube();
        assert!((cube.total_concentration() - 64.0).abs() < f64::EPSILON);
    }

    #[test]
    fn isa_cube_nonzero_absorbance() {
        let cube = isa_cube();
        // The ISA includes high-energy instructions, so total absorbance > 0
        assert!(cube.total_absorbance() > 0.0);
    }

    #[test]
    fn isa_cube_all_charges_represented() {
        let cube = isa_cube();
        let dist = cube.charge_distribution();
        for (i, &count) in dist.iter().enumerate() {
            assert!(count > 0.0, "charge state {i} has no instructions");
        }
    }

    #[test]
    fn isa_cube_all_energies_represented() {
        let cube = isa_cube();
        let dist = cube.energy_distribution();
        for (i, &count) in dist.iter().enumerate() {
            assert!(count > 0.0, "energy level {i} has no instructions");
        }
    }

    #[test]
    fn isa_cube_all_stabilities_represented() {
        let cube = isa_cube();
        let dist = cube.stability_distribution();
        for (i, &count) in dist.iter().enumerate() {
            assert!(count > 0.0, "stability class {i} has no instructions");
        }
    }

    #[test]
    fn projection_symmetry() {
        // For the ISA cube, all three projections should have the same total
        let cube = isa_cube();
        let sum_x: f64 = cube.project_x().iter().flat_map(|r| r.iter()).sum();
        let sum_y: f64 = cube.project_y().iter().flat_map(|r| r.iter()).sum();
        let sum_z: f64 = cube.project_z().iter().flat_map(|r| r.iter()).sum();

        // All projections sum to total absorbance
        let total = cube.total_absorbance();
        assert!(
            (sum_x - total).abs() < 0.001,
            "X projection sum {sum_x} != total {total}"
        );
        assert!(
            (sum_y - total).abs() < 0.001,
            "Y projection sum {sum_y} != total {total}"
        );
        assert!(
            (sum_z - total).abs() < 0.001,
            "Z projection sum {sum_z} != total {total}"
        );
    }
}
