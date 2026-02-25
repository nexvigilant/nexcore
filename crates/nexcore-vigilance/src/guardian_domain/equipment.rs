//! GMP Equipment Calibration Models.
//!
//! Equipment management and calibration tracking per GMP requirements.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};

/// Types of equipment requiring calibration.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EquipmentType {
    #[default]
    Analytical,
    Production,
    Utility,
    Weighing,
    Environmental,
    Cleaning,
}

/// Equipment operational status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum EquipmentStatus {
    #[default]
    Active,
    Inactive,
    CalibrationDue,
    OutOfService,
    Retired,
}

/// Calibration record status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize, Default)]
#[serde(rename_all = "SCREAMING_SNAKE_CASE")]
pub enum CalibrationStatus {
    #[default]
    Scheduled,
    InProgress,
    Passed,
    Failed,
    Cancelled,
}

/// Equipment record for GMP calibration tracking.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Equipment {
    pub id: String,
    pub name: String,
    pub equipment_type: EquipmentType,
    pub manufacturer: String,
    pub model: String,
    pub serial_number: String,
    pub location: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub department: Option<String>,
    pub calibration_frequency_days: i32,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub last_calibration_date: Option<DateTime>,
    pub next_calibration_date: DateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub installation_qualification_date: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub operational_qualification_date: Option<DateTime>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub performance_qualification_date: Option<DateTime>,
    #[serde(default)]
    pub status: EquipmentStatus,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_id: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub owner_name: Option<String>,
    #[serde(default = "DateTime::now")]
    pub created_at: DateTime,
    #[serde(default = "DateTime::now")]
    pub updated_at: DateTime,
    pub tenant_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

impl Equipment {
    /// Check if calibration is overdue.
    pub fn is_calibration_due(&self) -> bool {
        DateTime::now() >= self.next_calibration_date
    }

    /// Get days until next calibration (negative if overdue).
    pub fn days_until_calibration(&self) -> i64 {
        (self.next_calibration_date - DateTime::now()).num_days()
    }
}

/// Calibration record for GMP compliance.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationRecord {
    pub id: String,
    pub equipment_id: String,
    pub equipment_name: String,
    pub calibration_date: DateTime,
    pub calibration_type: String,
    pub performed_by: String,
    pub performed_by_name: String,
    pub method: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method_reference: Option<String>,
    pub status: CalibrationStatus,
    pub passed: bool,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub results: Option<String>,
    #[serde(default)]
    pub measurements: Vec<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub acceptance_criteria: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tolerance: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_url: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub certificate_number: Option<String>,
    pub next_calibration_date: DateTime,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_by: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_by_name: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub reviewed_date: Option<DateTime>,
    #[serde(default)]
    pub approved: bool,
    #[serde(default = "DateTime::now")]
    pub created_at: DateTime,
    pub tenant_id: String,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub notes: Option<String>,
}

/// Calibration schedule for planning.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CalibrationSchedule {
    pub tenant_id: String,
    pub start_date: DateTime,
    pub end_date: DateTime,
    pub equipment_count: i64,
    pub calibrations_due: i64,
    pub calibrations_overdue: i64,
    #[serde(default)]
    pub schedule: Vec<serde_json::Value>,
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_chrono::Duration;

    #[test]
    fn test_equipment_calibration_due() {
        let equipment = Equipment {
            id: "eq-1".to_string(),
            name: "HPLC".to_string(),
            equipment_type: EquipmentType::Analytical,
            manufacturer: "Waters".to_string(),
            model: "Alliance".to_string(),
            serial_number: "12345".to_string(),
            location: "Lab A".to_string(),
            department: None,
            calibration_frequency_days: 180,
            last_calibration_date: None,
            next_calibration_date: DateTime::now() - Duration::days(1),
            installation_qualification_date: None,
            operational_qualification_date: None,
            performance_qualification_date: None,
            status: EquipmentStatus::Active,
            owner_id: None,
            owner_name: None,
            created_at: DateTime::now(),
            updated_at: DateTime::now(),
            tenant_id: "tenant-1".to_string(),
            notes: None,
        };

        assert!(equipment.is_calibration_due());
        assert!(equipment.days_until_calibration() < 0);
    }

    #[test]
    fn test_equipment_type_default() {
        assert_eq!(EquipmentType::default(), EquipmentType::Analytical);
    }
}
