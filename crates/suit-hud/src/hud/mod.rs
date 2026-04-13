//! HUD Software — 7.2
//!
//! The AR overlay rendered onto the visor. Composites multiple layers:
//! attitude indicator, threat overlay, systems health, nav waypoints,
//! and voice agent response panel.

use serde::{Deserialize, Serialize};

/// A single HUD frame — all layers composited for one render tick.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HudFrame {
    /// Frame sequence number.
    pub seq: u64,
    /// Timestamp in milliseconds since boot.
    pub timestamp_ms: u64,
    /// Active layers (ordered back-to-front).
    pub layers: Vec<HudLayer>,
    /// Overall HUD opacity (0.0..1.0).
    pub opacity: f32,
    /// Whether foveated rendering is applied.
    pub foveated: bool,
}

/// Named HUD layers — each renders independently, composited in order.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum HudLayer {
    /// 7.2.1 Attitude indicator (pitch, roll, yaw, altitude, speed).
    Attitude(AttitudeIndicator),
    /// 7.2.2 Threat/obstacle overlay (bounding boxes, distance, severity).
    Threat(ThreatOverlay),
    /// 7.2.3 Systems health glanceable (power, thermal, integrity).
    Systems(SystemsGlanceable),
    /// 7.2.4 Map/nav waypoints.
    Nav(NavLayer),
    /// 7.2.5 Voice agent subtitle/response panel.
    Voice(VoicePanel),
}

/// 7.2.1 Attitude indicator — aircraft-style artificial horizon.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct AttitudeIndicator {
    /// Pitch in degrees (-90 to +90).
    pub pitch_deg: f32,
    /// Roll in degrees (-180 to +180).
    pub roll_deg: f32,
    /// Heading/yaw in degrees (0-360, 0=North).
    pub heading_deg: f32,
    /// Altitude in meters above ground.
    pub altitude_m: f32,
    /// Ground speed in m/s.
    pub speed_mps: f32,
    /// Vertical speed in m/s (positive = ascending).
    pub vertical_speed_mps: f32,
}

/// 7.2.2 Threat/obstacle overlay.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ThreatOverlay {
    /// Active threat markers.
    pub threats: Vec<ThreatMarker>,
    /// Obstacle proximity warnings.
    pub obstacles: Vec<ObstacleMarker>,
}

/// A single threat marker on the HUD.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ThreatMarker {
    /// Screen-space position (normalized 0.0..1.0).
    pub x: f32,
    /// Screen-space position.
    pub y: f32,
    /// Distance in meters.
    pub distance_m: f32,
    /// Threat severity.
    pub severity: ThreatSeverity,
    /// Label (e.g., "hostile", "SAM", "debris").
    pub label: String,
    /// Whether currently tracked (locked on).
    pub tracked: bool,
}

/// Threat severity levels.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThreatSeverity {
    /// Information only.
    Info,
    /// Caution — monitor.
    Caution,
    /// Warning — take action.
    Warning,
    /// Critical — immediate evasion.
    Critical,
}

/// Obstacle proximity marker.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ObstacleMarker {
    /// Bearing in degrees relative to heading.
    pub bearing_deg: f32,
    /// Distance in meters.
    pub distance_m: f32,
    /// Obstacle type.
    pub kind: String,
}

/// 7.2.3 Systems health glanceable — top-of-visor summary.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SystemsGlanceable {
    /// Power level (0.0..1.0).
    pub power: f32,
    /// Thermal status.
    pub thermal: ThermalStatus,
    /// Suit integrity (0.0..1.0).
    pub integrity: f32,
    /// Oxygen remaining in minutes (if sealed environment).
    pub o2_minutes: Option<f32>,
    /// Comms link quality (0.0..1.0).
    pub comms_quality: f32,
    /// Active subsystem count / total.
    pub subsystems_active: u8,
    /// Total subsystems.
    pub subsystems_total: u8,
}

/// Thermal status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ThermalStatus {
    /// Normal operating range.
    Nominal,
    /// Elevated — cooling active.
    Elevated,
    /// Warning — performance throttling.
    Warning,
    /// Critical — shutdown imminent.
    Critical,
}

impl Default for SystemsGlanceable {
    fn default() -> Self {
        Self {
            power: 1.0,
            thermal: ThermalStatus::Nominal,
            integrity: 1.0,
            o2_minutes: None,
            comms_quality: 1.0,
            subsystems_active: 12,
            subsystems_total: 12,
        }
    }
}

/// 7.2.4 Navigation layer.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct NavLayer {
    /// Active waypoints.
    pub waypoints: Vec<Waypoint>,
    /// Current route leg index.
    pub current_leg: usize,
    /// ETA to next waypoint in seconds.
    pub eta_seconds: Option<f32>,
}

/// A navigation waypoint.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Waypoint {
    /// Waypoint name.
    pub name: String,
    /// Latitude.
    pub lat: f64,
    /// Longitude.
    pub lon: f64,
    /// Altitude in meters (optional).
    pub alt_m: Option<f32>,
    /// Bearing from current position in degrees.
    pub bearing_deg: f32,
    /// Distance from current position in meters.
    pub distance_m: f32,
    /// Whether this is the active target.
    pub active: bool,
}

/// 7.2.5 Voice agent subtitle/response panel.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct VoicePanel {
    /// Current subtitle text (what user said).
    pub user_text: String,
    /// Agent response text.
    pub agent_text: String,
    /// Whether agent is currently processing.
    pub processing: bool,
    /// Panel visibility (0.0..1.0, fades in/out).
    pub visibility: f32,
}

/// Data sources that feed into HUD compositing.
/// Each source provides one layer of the composite frame.
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct HudDataSources {
    /// IMU attitude data.
    pub attitude: Option<AttitudeIndicator>,
    /// Threat sensor data.
    pub threats: Option<ThreatOverlay>,
    /// Systems telemetry.
    pub systems: Option<SystemsGlanceable>,
    /// Navigation waypoints.
    pub nav: Option<NavLayer>,
    /// Voice agent panel.
    pub voice: Option<VoicePanel>,
}

impl HudFrame {
    /// Compose a complete frame from multiple data sources.
    ///
    /// Each present source becomes a layer. Missing sources are skipped.
    /// Layers are ordered: attitude (back) → threats → systems → nav → voice (front).
    #[must_use]
    pub fn compose(seq: u64, timestamp_ms: u64, sources: &HudDataSources) -> Self {
        let mut frame = Self::new(seq, timestamp_ms);

        if let Some(att) = &sources.attitude {
            frame.add_layer(HudLayer::Attitude(att.clone()));
        }
        if let Some(threats) = &sources.threats {
            frame.add_layer(HudLayer::Threat(threats.clone()));
        }
        if let Some(sys) = &sources.systems {
            frame.add_layer(HudLayer::Systems(sys.clone()));
        }
        if let Some(nav) = &sources.nav {
            frame.add_layer(HudLayer::Nav(nav.clone()));
        }
        if let Some(voice) = &sources.voice {
            frame.add_layer(HudLayer::Voice(voice.clone()));
        }

        frame
    }

    /// Merge another frame's layers into this one (additive compositing).
    /// Duplicate layer types are kept — caller is responsible for dedup if needed.
    pub fn merge(&mut self, other: &HudFrame) {
        self.layers.extend(other.layers.iter().cloned());
    }

    /// Whether all systems are nominal (no warnings/critical).
    #[must_use]
    pub fn all_systems_nominal(&self) -> bool {
        self.systems_health()
            .map(|s| s.thermal == ThermalStatus::Nominal && s.integrity > 0.9 && s.power > 0.2)
            .unwrap_or(true) // no systems layer = assume nominal
    }

    /// Highest threat severity in the frame.
    #[must_use]
    pub fn max_threat_severity(&self) -> Option<ThreatSeverity> {
        self.layers
            .iter()
            .filter_map(|l| match l {
                HudLayer::Threat(t) => {
                    t.threats
                        .iter()
                        .map(|m| m.severity)
                        .max_by_key(|s| match s {
                            ThreatSeverity::Info => 0,
                            ThreatSeverity::Caution => 1,
                            ThreatSeverity::Warning => 2,
                            ThreatSeverity::Critical => 3,
                        })
                }
                _ => None,
            })
            .max_by_key(|s| match s {
                ThreatSeverity::Info => 0,
                ThreatSeverity::Caution => 1,
                ThreatSeverity::Warning => 2,
                ThreatSeverity::Critical => 3,
            })
    }

    /// Create an empty frame.
    #[must_use]
    pub fn new(seq: u64, timestamp_ms: u64) -> Self {
        Self {
            seq,
            timestamp_ms,
            layers: Vec::new(),
            opacity: 1.0,
            foveated: true,
        }
    }

    /// Add a layer to the frame.
    pub fn add_layer(&mut self, layer: HudLayer) {
        self.layers.push(layer);
    }

    /// Count active threat markers across all threat layers.
    #[must_use]
    pub fn threat_count(&self) -> usize {
        self.layers
            .iter()
            .filter_map(|l| match l {
                HudLayer::Threat(t) => Some(t.threats.len()),
                _ => None,
            })
            .sum()
    }

    /// Get the systems health if present.
    #[must_use]
    pub fn systems_health(&self) -> Option<&SystemsGlanceable> {
        self.layers.iter().find_map(|l| match l {
            HudLayer::Systems(s) => Some(s),
            _ => None,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn empty_frame() {
        let f = HudFrame::new(0, 1000);
        assert_eq!(f.threat_count(), 0);
        assert!(f.systems_health().is_none());
        assert!(f.foveated);
    }

    #[test]
    fn add_layers_and_query() {
        let mut f = HudFrame::new(1, 2000);
        f.add_layer(HudLayer::Attitude(AttitudeIndicator {
            pitch_deg: 5.0,
            roll_deg: -2.0,
            heading_deg: 270.0,
            ..AttitudeIndicator::default()
        }));
        f.add_layer(HudLayer::Threat(ThreatOverlay {
            threats: vec![
                ThreatMarker {
                    x: 0.5,
                    y: 0.3,
                    distance_m: 500.0,
                    severity: ThreatSeverity::Warning,
                    label: "hostile".into(),
                    tracked: true,
                },
                ThreatMarker {
                    x: 0.8,
                    y: 0.6,
                    distance_m: 1200.0,
                    severity: ThreatSeverity::Info,
                    label: "unknown".into(),
                    tracked: false,
                },
            ],
            obstacles: vec![],
        }));
        f.add_layer(HudLayer::Systems(SystemsGlanceable::default()));

        assert_eq!(f.threat_count(), 2);
        assert!(f.systems_health().is_some());
        let sys = f.systems_health();
        assert!(sys.is_some());
        if let Some(s) = sys {
            assert_eq!(s.thermal, ThermalStatus::Nominal);
            assert_eq!(s.subsystems_active, 12);
        }
    }

    #[test]
    fn voice_panel_default() {
        let v = VoicePanel::default();
        assert!(v.user_text.is_empty());
        assert!(!v.processing);
    }

    #[test]
    fn waypoint_navigation() {
        let nav = NavLayer {
            waypoints: vec![Waypoint {
                name: "Alpha".into(),
                lat: 40.7128,
                lon: -74.0060,
                alt_m: Some(100.0),
                bearing_deg: 45.0,
                distance_m: 2000.0,
                active: true,
            }],
            current_leg: 0,
            eta_seconds: Some(120.0),
        };
        assert_eq!(nav.waypoints.len(), 1);
        assert!(nav.waypoints[0].active);
    }

    #[test]
    fn threat_severity_ordering() {
        // Verify variants exist and are distinct
        assert_ne!(ThreatSeverity::Info, ThreatSeverity::Critical);
        assert_ne!(ThreatSeverity::Caution, ThreatSeverity::Warning);
    }

    #[test]
    fn hud_frame_serializes() {
        let mut f = HudFrame::new(42, 5000);
        f.add_layer(HudLayer::Systems(SystemsGlanceable::default()));
        let json = serde_json::to_string(&f);
        assert!(json.is_ok());
    }

    #[test]
    fn compose_from_data_sources() {
        let sources = HudDataSources {
            attitude: Some(AttitudeIndicator {
                pitch_deg: 10.0,
                heading_deg: 90.0,
                ..AttitudeIndicator::default()
            }),
            systems: Some(SystemsGlanceable::default()),
            threats: None,
            nav: None,
            voice: Some(VoicePanel {
                agent_text: "Systems nominal".into(),
                visibility: 1.0,
                ..VoicePanel::default()
            }),
        };

        let frame = HudFrame::compose(1, 1000, &sources);
        assert_eq!(frame.layers.len(), 3); // attitude + systems + voice
        assert!(frame.all_systems_nominal());
        assert_eq!(frame.threat_count(), 0);
    }

    #[test]
    fn compose_with_threats() {
        let sources = HudDataSources {
            threats: Some(ThreatOverlay {
                threats: vec![ThreatMarker {
                    x: 0.5,
                    y: 0.3,
                    distance_m: 200.0,
                    severity: ThreatSeverity::Critical,
                    label: "incoming".into(),
                    tracked: true,
                }],
                obstacles: vec![],
            }),
            ..HudDataSources::default()
        };

        let frame = HudFrame::compose(2, 2000, &sources);
        assert_eq!(frame.threat_count(), 1);
        assert_eq!(frame.max_threat_severity(), Some(ThreatSeverity::Critical));
    }

    #[test]
    fn merge_frames() {
        let mut f1 = HudFrame::new(1, 100);
        f1.add_layer(HudLayer::Attitude(AttitudeIndicator::default()));

        let mut f2 = HudFrame::new(2, 200);
        f2.add_layer(HudLayer::Systems(SystemsGlanceable::default()));

        f1.merge(&f2);
        assert_eq!(f1.layers.len(), 2);
        assert!(f1.systems_health().is_some());
    }

    #[test]
    fn all_systems_nominal_detects_degradation() {
        let mut f = HudFrame::new(0, 0);
        f.add_layer(HudLayer::Systems(SystemsGlanceable {
            thermal: ThermalStatus::Critical,
            ..SystemsGlanceable::default()
        }));
        assert!(!f.all_systems_nominal());
    }

    #[test]
    fn max_threat_severity_none_when_empty() {
        let f = HudFrame::new(0, 0);
        assert!(f.max_threat_severity().is_none());
    }
}
