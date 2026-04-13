//! Helmet subsystem — 7.1
//!
//! Typed models for the physical helmet: shell state, visor/jaw actuator,
//! AR optics, eye tracker, bone-conduction audio, beamforming mic array,
//! and face seal ventilation.

use serde::{Deserialize, Serialize};

/// 7.1.1 Shell material and state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ShellMaterial {
    /// Carbon fiber composite.
    CarbonFiber,
    /// Titanium alloy.
    Titanium,
    /// Hybrid carbon-titanium.
    Hybrid,
}

/// 7.1.2 Visor/jaw actuator state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VisorState {
    /// Fully open — face exposed.
    Open,
    /// Closing — jaw actuator in motion.
    Closing,
    /// Fully sealed — HUD active.
    Sealed,
    /// Opening — jaw actuator retracting.
    Opening,
    /// Locked — emergency seal, cannot open without override.
    Locked,
}

impl VisorState {
    /// Whether the visor allows HUD projection.
    #[must_use]
    pub fn hud_active(self) -> bool {
        matches!(self, Self::Sealed | Self::Locked)
    }

    /// Whether the face is exposed to environment.
    #[must_use]
    pub fn face_exposed(self) -> bool {
        matches!(self, Self::Open)
    }
}

/// 7.1.3 AR optics configuration.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ArOptics {
    /// Optics type.
    pub kind: OpticsKind,
    /// Field of view in degrees (horizontal).
    pub fov_horizontal: f32,
    /// Field of view in degrees (vertical).
    pub fov_vertical: f32,
    /// Resolution per eye (pixels).
    pub resolution_per_eye: (u32, u32),
    /// Refresh rate in Hz.
    pub refresh_rate: u32,
    /// Current brightness (0.0..1.0).
    pub brightness: f32,
}

/// AR optics type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum OpticsKind {
    /// Waveguide — thinner, lighter, wider FOV.
    Waveguide,
    /// Birdbath — simpler, higher contrast.
    Birdbath,
}

impl Default for ArOptics {
    fn default() -> Self {
        Self {
            kind: OpticsKind::Waveguide,
            fov_horizontal: 52.0,
            fov_vertical: 40.0,
            resolution_per_eye: (2560, 1440),
            refresh_rate: 120,
            brightness: 0.7,
        }
    }
}

/// 7.1.4 Eye tracker state — foveated rendering + gaze input.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct EyeTracker {
    /// Whether eye tracking is active.
    pub active: bool,
    /// Current gaze point (normalized 0.0..1.0 in HUD space).
    pub gaze: GazePoint,
    /// Pupil diameter in mm (stress/light indicator).
    pub pupil_diameter_mm: f32,
    /// Blink rate per minute (fatigue indicator).
    pub blink_rate: f32,
    /// Foveated rendering enabled.
    pub foveated: bool,
}

/// Normalized gaze coordinates in HUD space.
#[derive(Debug, Clone, Copy, Default, Serialize, Deserialize)]
pub struct GazePoint {
    /// Horizontal position (0.0 = left, 1.0 = right).
    pub x: f32,
    /// Vertical position (0.0 = top, 1.0 = bottom).
    pub y: f32,
    /// Confidence (0.0..1.0).
    pub confidence: f32,
}

impl Default for EyeTracker {
    fn default() -> Self {
        Self {
            active: false,
            gaze: GazePoint::default(),
            pupil_diameter_mm: 4.0,
            blink_rate: 15.0,
            foveated: true,
        }
    }
}

/// 7.1.6 Beamforming mic array.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MicArray {
    /// Number of microphone elements.
    pub element_count: u8,
    /// Whether beamforming is active (directional capture).
    pub beamforming: bool,
    /// Current beam direction in degrees (0 = forward, -90 left, +90 right).
    pub beam_direction: f32,
    /// Noise cancellation active.
    pub noise_cancel: bool,
    /// Current input level (0.0..1.0).
    pub input_level: f32,
}

impl Default for MicArray {
    fn default() -> Self {
        Self {
            element_count: 6,
            beamforming: true,
            beam_direction: 0.0,
            noise_cancel: true,
            input_level: 0.0,
        }
    }
}

/// 7.1.7 Ventilation state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum VentilationMode {
    /// Sealed — no external air.
    Sealed,
    /// Passive — vents open.
    Passive,
    /// Active — fan forced.
    Active,
    /// Emergency — full purge.
    Emergency,
}

/// Complete helmet state snapshot.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HelmetState {
    /// Shell material.
    pub shell: ShellMaterial,
    /// Visor position.
    pub visor: VisorState,
    /// AR optics config.
    pub optics: ArOptics,
    /// Eye tracker state.
    pub eye_tracker: EyeTracker,
    /// Mic array state.
    pub mic_array: MicArray,
    /// Ventilation mode.
    pub ventilation: VentilationMode,
    /// Internal temperature in Celsius.
    pub internal_temp_c: f32,
    /// Face seal integrity (0.0..1.0).
    pub seal_integrity: f32,
}

impl Default for HelmetState {
    fn default() -> Self {
        Self {
            shell: ShellMaterial::CarbonFiber,
            visor: VisorState::Open,
            optics: ArOptics::default(),
            eye_tracker: EyeTracker::default(),
            mic_array: MicArray::default(),
            ventilation: VentilationMode::Passive,
            internal_temp_c: 22.0,
            seal_integrity: 1.0,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn visor_hud_active_when_sealed() {
        assert!(VisorState::Sealed.hud_active());
        assert!(VisorState::Locked.hud_active());
        assert!(!VisorState::Open.hud_active());
        assert!(!VisorState::Closing.hud_active());
    }

    #[test]
    fn default_helmet_state() {
        let h = HelmetState::default();
        assert_eq!(h.visor, VisorState::Open);
        assert_eq!(h.shell, ShellMaterial::CarbonFiber);
        assert!(h.seal_integrity > 0.99);
        assert!(h.eye_tracker.foveated);
    }

    #[test]
    fn gaze_point_default_is_zero() {
        let g = GazePoint::default();
        assert_eq!(g.x, 0.0);
        assert_eq!(g.y, 0.0);
        assert_eq!(g.confidence, 0.0);
    }

    #[test]
    fn ar_optics_default_values() {
        let o = ArOptics::default();
        assert_eq!(o.kind, OpticsKind::Waveguide);
        assert_eq!(o.refresh_rate, 120);
        assert!(o.fov_horizontal > 50.0);
    }

    #[test]
    fn mic_array_default() {
        let m = MicArray::default();
        assert_eq!(m.element_count, 6);
        assert!(m.beamforming);
        assert!(m.noise_cancel);
    }

    #[test]
    fn helmet_serializes() {
        let h = HelmetState::default();
        let json = serde_json::to_string(&h);
        assert!(json.is_ok());
    }
}
