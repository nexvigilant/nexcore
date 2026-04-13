//! Thevenin 2-RC Battery Model implementation.

use serde::{Deserialize, Serialize};

/// State of the Thevenin 2-RC model [SoC, V_RC1, V_RC2]
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct ModelState {
    pub soc: f32,
    pub v_rc1: f32,
    pub v_rc2: f32,
}

/// Parameters for a specific chemistry
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct CellParams {
    pub r0: f32,
    pub r1: f32,
    pub c1: f32,
    pub r2: f32,
    pub c2: f32,
    pub capacity_ah: f32,
}

impl Default for CellParams {
    fn default() -> Self {
        // Representative values for Samsung 50S (NMC)
        Self {
            r0: 0.02,
            r1: 0.01,
            c1: 1000.0,
            r2: 0.005,
            c2: 5000.0,
            capacity_ah: 5.0,
        }
    }
}

/// Thevenin2RC_Update implementation
pub fn update(state: ModelState, i_load: f32, dt: f32, p: &CellParams) -> (ModelState, f32) {
    let soc_next = state.soc - (i_load * dt) / (p.capacity_ah * 3600.0);

    // Euler integration of RC branches: v_rc_next = v_rc_prev * exp(-dt/RC) + i*R*(1 - exp(-dt/RC))
    let exp1 = (-dt / (p.r1 * p.c1)).exp();
    let v_rc1_next = state.v_rc1 * exp1 + i_load * p.r1 * (1.0 - exp1);

    let exp2 = (-dt / (p.r2 * p.c2)).exp();
    let v_rc2_next = state.v_rc2 * exp2 + i_load * p.r2 * (1.0 - exp2);

    // OCV placeholder: Linear approximation 3.0V to 4.2V
    let v_oc = 3.0 + (soc_next * 1.2);
    let v_terminal = v_oc - (i_load * p.r0) - v_rc1_next - v_rc2_next;

    (
        ModelState {
            soc: soc_next,
            v_rc1: v_rc1_next,
            v_rc2: v_rc2_next,
        },
        v_terminal,
    )
}
