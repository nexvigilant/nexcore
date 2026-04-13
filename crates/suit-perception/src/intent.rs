use serde::{Deserialize, Serialize};

/// High-level user intent classified from sensor data.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum Intent {
    /// Static standing or slight swaying
    Standing,
    /// Low-velocity forward movement
    Walking,
    /// High-velocity forward movement
    Running,
    /// Vertical acceleration event detected
    Jumping,
    /// Preparing for high-impact or external force
    Bracing,
    /// Moving to a seated or crouching position
    Crouching,
}

/// Logic for running intent classification models.
pub struct IntentEngine {
    /// TODO: Placeholder for the loaded candle-core model.
    pub model_path: String,
}

impl IntentEngine {
    /// Create a new IntentEngine.
    pub fn new(model_path: String) -> Self {
        Self { model_path }
    }

    /// Predict user intent based on the current body and inertial state.
    ///
    /// TODO: Implement model inference using the loaded model.
    pub fn predict(
        &self,
        _body: &crate::proprioceptive::BodyState,
        _vestibular: &crate::vestibular::InertialState,
    ) -> Intent {
        // Placeholder for ML inference
        Intent::Standing
    }
}
