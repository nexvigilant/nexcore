//! # NexVigilant Core — Integumentary System
//!
//! Boundary protection layer mapping to Claude Code's permission and settings infrastructure.
//! The skin of the system — first line of defense and primary external interface.
//!
//! ## Biological Alignment (v2.0 §2)
//!
//! ```text
//! Epidermis  = Permission Rules (deny → ask → allow cascade)
//! Dermis     = Settings Precedence Stack (managed → cli → local → project → user)
//! Hypodermis = Sandboxing (Docker isolation, network restrictions)
//! Scarring   = Adaptive deny rules post-incident (system strengthens at break point)
//! ```
//!
//! ## Claude Code Infrastructure Mapping
//!
//! | Biological Layer | Claude Code Mechanism | Module |
//! |-----------------|----------------------|--------|
//! | Epidermis (auth/validate) | `permissions.deny/ask/allow` cascade | [`claude_code::PermissionCascade`] |
//! | Dermis (sensors) | Settings precedence: managed→cli→local→project→user | [`claude_code::SettingsPrecedence`] |
//! | Hypodermis (fat/insulation) | Docker sandbox + network restrictions | [`claude_code::SandboxLayer`] |
//! | Scarring (reinforcement) | Adaptive deny rules after security incidents | [`claude_code::ScarringMechanism`] |
//! | Sweat glands (cooling) | Rate limiting under load | [`SweatGlands`] |
//! | Sensors (pain/temp/pressure) | System vital sign monitoring | [`DermisSensors`] |
//!
//! ## Generic Biology Layer
//!
//! The base types (`Epidermis`, `DermisSensors`, `SweatGlands`, `ProtectionLayer`)
//! model biological primitives. The [`claude_code`] module maps these to Claude Code
//! infrastructure with full SSA naming and alignment doc references.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod claude_code;
pub mod grounding;

use serde::{Deserialize, Serialize};

// ============================================================================
// Error Type
// ============================================================================

/// Errors in the integumentary system.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub enum IntegumentaryError {
    /// Authentication failed
    AuthFailed(String),
    /// Validation rejected input
    ValidationFailed(String),
    /// Wound could not be repaired
    RepairFailed(String),
}

impl core::fmt::Display for IntegumentaryError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        match self {
            Self::AuthFailed(msg) => write!(f, "auth failed: {msg}"),
            Self::ValidationFailed(msg) => write!(f, "validation failed: {msg}"),
            Self::RepairFailed(msg) => write!(f, "repair failed: {msg}"),
        }
    }
}

impl std::error::Error for IntegumentaryError {}

// ============================================================================
// AuthResult — Epidermis authentication
// ============================================================================

/// Result of epidermis authentication check.
/// Maps JS: `epidermis.authenticate()` → { user, authorized, timestamp }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct AuthResult {
    /// Identified user/entity
    pub identity: String,
    /// Whether access is authorized
    pub authorized: bool,
    /// Timestamp of authentication
    pub authenticated_at: String,
}

// ============================================================================
// Sensor Types — Dermis
// ============================================================================

/// Classification of dermis sensors.
/// Maps JS: `dermis.sensors.touch/temperature/pressure/pain`
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum SensorKind {
    /// System load / temperature
    Temperature,
    /// System pressure (trigger count, queue depth)
    Pressure,
    /// Error/pain detection
    Pain,
    /// User interaction detection
    Touch,
}

/// A reading from a dermis sensor.
/// Maps JS: `sensors.temperature()` → { value, status }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SensorReading {
    /// Sensor type
    pub kind: SensorKind,
    /// Numeric value
    pub value: f64,
    /// Derived status
    pub status: SensorStatus,
}

/// Status derived from a sensor reading.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SensorStatus {
    /// Within normal range
    Normal,
    /// Above normal (hot, high pressure)
    High,
    /// Below normal (cold, low pressure)
    Low,
    /// Pain/error detected
    Alert,
}

// ============================================================================
// SkinCondition — Composite state
// ============================================================================

/// Overall skin condition from all sensors.
/// Maps JS: aggregated dermis sensor readings
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SkinCondition {
    /// Temperature reading
    pub temperature: SensorReading,
    /// Pressure reading
    pub pressure: SensorReading,
    /// Pain reading
    pub pain: SensorReading,
    /// Whether any sensor is in alert
    pub has_alert: bool,
}

// ============================================================================
// Shield — Protection layer
// ============================================================================

/// Active shield protecting the system boundary.
/// Maps JS: `protect.shield()` → { authentication, validation, monitoring }
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Shield {
    /// Authentication active
    pub auth_active: bool,
    /// Validation active
    pub validation_active: bool,
    /// Monitoring active
    pub monitoring_active: bool,
}

// ============================================================================
// WoundRepair — Healing
// ============================================================================

/// Record of a wound repair action.
/// Maps JS: `protect.heal(wound)` → repair UI or data
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WoundRepair {
    /// What was damaged
    pub location: String,
    /// Type of wound
    pub wound_type: WoundType,
    /// Whether repair succeeded
    pub healed: bool,
}

/// Types of wounds the skin can sustain.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum WoundType {
    /// UI/display damage
    Interface,
    /// Data corruption
    Data,
    /// Configuration damage
    Config,
}

// ============================================================================
// CoolingAction — Sweat glands
// ============================================================================

/// Cooling action taken when system is overheated.
/// Maps JS: `sweat.activate()` → `coolDown()`
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CoolingAction {
    /// What was cooled
    pub target: String,
    /// Load before cooling
    pub load_before: f64,
    /// Load after cooling
    pub load_after: f64,
    /// Whether cooling was effective
    pub effective: bool,
}

// ============================================================================
// Epidermis — Authentication and validation barrier
// ============================================================================

/// The epidermis: outermost barrier for auth and validation.
/// Maps JS: `INTEGUMENTARY.epidermis`
pub struct Epidermis {
    /// Required fields for validation
    pub required_fields: Vec<String>,
}

impl Default for Epidermis {
    fn default() -> Self {
        Self {
            required_fields: vec![
                "id".to_string(),
                "timestamp".to_string(),
                "data".to_string(),
            ],
        }
    }
}

impl Epidermis {
    /// Authenticate an entity.
    /// Maps JS: `epidermis.authenticate()`
    pub fn authenticate(&self, identity: &str) -> AuthResult {
        AuthResult {
            identity: identity.to_string(),
            authorized: !identity.is_empty(),
            authenticated_at: nexcore_chrono::DateTime::now().to_rfc3339(),
        }
    }

    /// Validate input data has required fields.
    /// Maps JS: `epidermis.validate()`
    pub fn validate(&self, data: &serde_json::Value) -> Result<(), IntegumentaryError> {
        if let serde_json::Value::Object(map) = data {
            for field in &self.required_fields {
                if !map.contains_key(field) {
                    return Err(IntegumentaryError::ValidationFailed(format!(
                        "missing required field: {field}"
                    )));
                }
            }
            Ok(())
        } else {
            Err(IntegumentaryError::ValidationFailed(
                "expected object".to_string(),
            ))
        }
    }
}

// ============================================================================
// DermisSensors — Sensor array
// ============================================================================

/// The dermis sensor array: monitors system vital signs.
/// Maps JS: `dermis.sensors`
pub struct DermisSensors;

impl DermisSensors {
    /// Read temperature (system load).
    /// Maps JS: `dermis.sensors.temperature()`
    pub fn temperature(&self, load: f64) -> SensorReading {
        let status = if load > 80.0 {
            SensorStatus::High
        } else if load < 20.0 {
            SensorStatus::Low
        } else {
            SensorStatus::Normal
        };
        SensorReading {
            kind: SensorKind::Temperature,
            value: load,
            status,
        }
    }

    /// Read pressure (queue depth / trigger count).
    /// Maps JS: `dermis.sensors.pressure()`
    pub fn pressure(&self, trigger_count: f64) -> SensorReading {
        let status = if trigger_count > 15.0 {
            SensorStatus::High
        } else if trigger_count < 5.0 {
            SensorStatus::Low
        } else {
            SensorStatus::Normal
        };
        SensorReading {
            kind: SensorKind::Pressure,
            value: trigger_count,
            status,
        }
    }

    /// Read pain (error count).
    /// Maps JS: `dermis.sensors.pain()`
    pub fn pain(&self, error_count: f64) -> SensorReading {
        let status = if error_count > 0.0 {
            SensorStatus::Alert
        } else {
            SensorStatus::Normal
        };
        SensorReading {
            kind: SensorKind::Pain,
            value: error_count,
            status,
        }
    }

    /// Full skin condition check.
    pub fn full_check(&self, load: f64, triggers: f64, errors: f64) -> SkinCondition {
        let temp = self.temperature(load);
        let press = self.pressure(triggers);
        let pain = self.pain(errors);
        let has_alert = temp.status == SensorStatus::High
            || press.status == SensorStatus::High
            || pain.status == SensorStatus::Alert;

        SkinCondition {
            temperature: temp,
            pressure: press,
            pain,
            has_alert,
        }
    }
}

// ============================================================================
// SweatGlands — Cooling system
// ============================================================================

/// Sweat glands: cool the system when overheated.
/// Maps JS: `INTEGUMENTARY.sweat`
pub struct SweatGlands;

impl SweatGlands {
    /// Attempt to cool the system.
    /// Maps JS: `sweat.activate()` + `coolDown()`
    pub fn cool(&self, target: &str, current_load: f64) -> CoolingAction {
        let load_after = (current_load * 0.7).max(0.0);
        CoolingAction {
            target: target.to_string(),
            load_before: current_load,
            load_after,
            effective: load_after < 80.0,
        }
    }
}

// ============================================================================
// ProtectionLayer — Shield and repair
// ============================================================================

/// Protection layer: activates shields and repairs wounds.
/// Maps JS: `INTEGUMENTARY.protect`
pub struct ProtectionLayer;

impl ProtectionLayer {
    /// Activate all barriers.
    /// Maps JS: `protect.shield()`
    pub fn shield(&self) -> Shield {
        Shield {
            auth_active: true,
            validation_active: true,
            monitoring_active: true,
        }
    }

    /// Repair a wound.
    /// Maps JS: `protect.heal(wound)`
    pub fn heal(&self, location: &str, wound_type: WoundType) -> WoundRepair {
        WoundRepair {
            location: location.to_string(),
            wound_type,
            healed: true, // Simple repair always succeeds
        }
    }
}

// ============================================================================
// IntegumentarySystem — Full orchestrator
// ============================================================================

/// The complete integumentary system.
pub struct IntegumentarySystem {
    pub epidermis: Epidermis,
    pub sensors: DermisSensors,
    pub sweat: SweatGlands,
    pub protection: ProtectionLayer,
}

impl Default for IntegumentarySystem {
    fn default() -> Self {
        Self {
            epidermis: Epidermis::default(),
            sensors: DermisSensors,
            sweat: SweatGlands,
            protection: ProtectionLayer,
        }
    }
}

impl IntegumentarySystem {
    /// Full skin care routine: check sensors, cool if needed, update interface.
    /// Maps JS: `skinCare()`
    pub fn skin_care(&self, load: f64, triggers: f64, errors: f64) -> SkinCondition {
        let condition = self.sensors.full_check(load, triggers, errors);

        if condition.temperature.status == SensorStatus::High {
            let _cooling = self.sweat.cool("system", load);
        }

        condition
    }
}

// ============================================================================
// Tests
// ============================================================================

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn epidermis_auth_valid() {
        let epidermis = Epidermis::default();
        let result = epidermis.authenticate("user@example.com");
        assert!(result.authorized);
    }

    #[test]
    fn epidermis_auth_empty_rejects() {
        let epidermis = Epidermis::default();
        let result = epidermis.authenticate("");
        assert!(!result.authorized);
    }

    #[test]
    fn epidermis_validate_passes() {
        let epidermis = Epidermis::default();
        let data = serde_json::json!({
            "id": "123",
            "timestamp": "2026-02-10",
            "data": "payload"
        });
        assert!(epidermis.validate(&data).is_ok());
    }

    #[test]
    fn epidermis_validate_missing_field() {
        let epidermis = Epidermis::default();
        let data = serde_json::json!({"id": "123"});
        assert!(epidermis.validate(&data).is_err());
    }

    #[test]
    fn epidermis_validate_non_object() {
        let epidermis = Epidermis::default();
        let data = serde_json::json!("not an object");
        assert!(epidermis.validate(&data).is_err());
    }

    #[test]
    fn sensors_temperature_ranges() {
        let sensors = DermisSensors;
        assert_eq!(sensors.temperature(90.0).status, SensorStatus::High);
        assert_eq!(sensors.temperature(50.0).status, SensorStatus::Normal);
        assert_eq!(sensors.temperature(10.0).status, SensorStatus::Low);
    }

    #[test]
    fn sensors_pain_detects_errors() {
        let sensors = DermisSensors;
        assert_eq!(sensors.pain(0.0).status, SensorStatus::Normal);
        assert_eq!(sensors.pain(1.0).status, SensorStatus::Alert);
    }

    #[test]
    fn full_check_detects_alert() {
        let sensors = DermisSensors;
        let condition = sensors.full_check(90.0, 5.0, 0.0);
        assert!(condition.has_alert);

        let condition = sensors.full_check(50.0, 10.0, 0.0);
        assert!(!condition.has_alert);
    }

    #[test]
    fn sweat_cooling_reduces_load() {
        let sweat = SweatGlands;
        let action = sweat.cool("system", 100.0);
        assert!(action.load_after < action.load_before);
    }

    #[test]
    fn shield_activates_all() {
        let protection = ProtectionLayer;
        let shield = protection.shield();
        assert!(shield.auth_active);
        assert!(shield.validation_active);
        assert!(shield.monitoring_active);
    }

    #[test]
    fn wound_repair_succeeds() {
        let protection = ProtectionLayer;
        let repair = protection.heal("dashboard", WoundType::Interface);
        assert!(repair.healed);
    }

    #[test]
    fn full_skin_care_routine() {
        let system = IntegumentarySystem::default();
        let condition = system.skin_care(50.0, 10.0, 0.0);
        assert!(!condition.has_alert);
    }

    #[test]
    fn full_skin_care_hot() {
        let system = IntegumentarySystem::default();
        let condition = system.skin_care(95.0, 10.0, 3.0);
        assert!(condition.has_alert);
    }
}
