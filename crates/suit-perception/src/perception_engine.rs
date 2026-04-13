//! # Perception Software Pipeline
//!
//! Orchestrates high-level perception, including sensor fusion, VIO,
//! occupancy mapping, and ML-based intent classification.

use crate::exteroceptive::{Obstacle, PointCloud};
use crate::fusion::{FusionEngine, WorldState};
use crate::intent::IntentEngine;
use crate::proprioceptive::BodyState;
use crate::vestibular::{BarometricData, InertialState, MagnetometerData};
use heapless::Vec as HVec;

/// The central perception engine for the suit.
pub struct PerceptionEngine {
    /// Fusion engine for global world state estimation.
    pub fusion: FusionEngine,
    /// ML-based intent classification engine.
    pub intent: IntentEngine,
    /// 3D occupancy map (fixed capacity for hard-real-time).
    pub occupancy_map: HVec<Obstacle, 128>,
}

impl PerceptionEngine {
    /// Create a new perception engine.
    pub fn new(model_path: String) -> Self {
        Self {
            fusion: FusionEngine::new(),
            intent: IntentEngine::new(model_path),
            occupancy_map: HVec::new(),
        }
    }

    /// Orchestrates a perception update loop.
    pub fn update(
        &mut self,
        vest: &InertialState,
        mag: &MagnetometerData,
        baro: &BarometricData,
        body: &BodyState,
        pc: &Option<PointCloud>,
    ) -> WorldState {
        // 1. Run Fusion (World State update)
        self.fusion.update(vest, mag, baro, body, pc);

        // 2. Perform Visual-Inertial Odometry / SLAM refinement
        if let Some(pc_data) = pc {
            self.update_occupancy_map(pc_data);
        }

        // 3. Classify user intent
        let _intent = self.intent.predict(body, vest);

        // Return current fusion result
        self.fusion.current_state
    }

    fn update_occupancy_map(&mut self, _pc: &PointCloud) {
        // TODO: OctoMap-style occupancy grid integration using HVec.
    }
}
