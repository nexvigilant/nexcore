//! Safety communication types.
//!
//! Models regulatory safety communications issued for pharmaceutical
//! products: Dear Healthcare Professional letters, safety updates,
//! risk communications, field alerts, and recalls.

use std::fmt;

use serde::{Deserialize, Serialize};

/// A regulatory safety communication issued for a pharmaceutical product.
///
/// Safety communications represent formal notifications to healthcare
/// professionals, patients, or the public about product safety issues.
///
/// # Examples
///
/// ```
/// use nexcore_pharma::{SafetyCommunication, CommType};
///
/// let comm = SafetyCommunication {
///     title: "Important Safety Information: Updated Dosing Guidance".to_string(),
///     date: "2024-03-15".to_string(),
///     comm_type: CommType::DearHcpLetter,
///     product: "atorvastatin".to_string(),
///     summary: "Updated dosing recommendations for patients with renal impairment.".to_string(),
/// };
/// assert!(comm.is_urgent());
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SafetyCommunication {
    /// Title of the communication as issued by the sponsor or regulator
    pub title: String,
    /// ISO 8601 date string (YYYY-MM-DD) of issue
    pub date: String,
    /// Type of safety communication
    pub comm_type: CommType,
    /// Generic name of the affected product
    pub product: String,
    /// Plain-language summary of the safety concern
    pub summary: String,
}

impl SafetyCommunication {
    /// Returns `true` for communication types that require immediate
    /// healthcare professional action (Dear HCP letters and recalls).
    pub fn is_urgent(&self) -> bool {
        matches!(self.comm_type, CommType::DearHcpLetter | CommType::Recall)
    }

    /// Returns `true` if this communication indicates a product recall.
    pub fn is_recall(&self) -> bool {
        matches!(self.comm_type, CommType::Recall)
    }
}

/// Type of regulatory safety communication.
///
/// Ordered roughly by ascending urgency/severity for display purposes,
/// though `PartialOrd` is not derived — urgency is assessed via
/// [`SafetyCommunication::is_urgent`].
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum CommType {
    /// Direct notification letter to healthcare professionals
    DearHcpLetter,
    /// Labelling or prescribing information update
    SafetyUpdate,
    /// Broad public or professional risk communication
    RiskCommunication,
    /// Field safety notice for distributed product
    FieldAlert,
    /// Product recall (voluntary or mandated)
    Recall,
}

impl fmt::Display for CommType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let s = match self {
            Self::DearHcpLetter => "Dear HCP Letter",
            Self::SafetyUpdate => "Safety Update",
            Self::RiskCommunication => "Risk Communication",
            Self::FieldAlert => "Field Alert",
            Self::Recall => "Recall",
        };
        f.write_str(s)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn make_comm(comm_type: CommType) -> SafetyCommunication {
        SafetyCommunication {
            title: "Test communication".to_string(),
            date: "2024-01-15".to_string(),
            comm_type,
            product: "testdrug".to_string(),
            summary: "Summary text.".to_string(),
        }
    }

    #[test]
    fn dear_hcp_letter_is_urgent() {
        assert!(make_comm(CommType::DearHcpLetter).is_urgent());
    }

    #[test]
    fn recall_is_urgent_and_is_recall() {
        let comm = make_comm(CommType::Recall);
        assert!(comm.is_urgent());
        assert!(comm.is_recall());
    }

    #[test]
    fn safety_update_is_not_urgent() {
        assert!(!make_comm(CommType::SafetyUpdate).is_urgent());
    }

    #[test]
    fn risk_communication_is_not_urgent() {
        assert!(!make_comm(CommType::RiskCommunication).is_urgent());
    }

    #[test]
    fn field_alert_is_not_urgent() {
        assert!(!make_comm(CommType::FieldAlert).is_urgent());
    }

    #[test]
    fn only_recall_is_recall() {
        for comm_type in [
            CommType::DearHcpLetter,
            CommType::SafetyUpdate,
            CommType::RiskCommunication,
            CommType::FieldAlert,
        ] {
            assert!(!make_comm(comm_type).is_recall());
        }
    }

    #[test]
    fn comm_type_display() {
        assert_eq!(CommType::DearHcpLetter.to_string(), "Dear HCP Letter");
        assert_eq!(CommType::SafetyUpdate.to_string(), "Safety Update");
        assert_eq!(
            CommType::RiskCommunication.to_string(),
            "Risk Communication"
        );
        assert_eq!(CommType::FieldAlert.to_string(), "Field Alert");
        assert_eq!(CommType::Recall.to_string(), "Recall");
    }

    #[test]
    fn safety_communication_serializes_round_trip() {
        let comm = make_comm(CommType::DearHcpLetter);
        let json = serde_json::to_string(&comm).expect("serialization cannot fail");
        let parsed: SafetyCommunication =
            serde_json::from_str(&json).expect("deserialization cannot fail");
        assert_eq!(parsed.product, "testdrug");
        assert_eq!(parsed.comm_type, CommType::DearHcpLetter);
    }

    #[test]
    fn comm_type_serializes_round_trip() {
        for comm_type in [
            CommType::DearHcpLetter,
            CommType::SafetyUpdate,
            CommType::RiskCommunication,
            CommType::FieldAlert,
            CommType::Recall,
        ] {
            let json =
                serde_json::to_string(&comm_type).expect("serialization cannot fail on valid enum");
            let parsed: CommType =
                serde_json::from_str(&json).expect("deserialization cannot fail on valid JSON");
            assert_eq!(comm_type, parsed);
        }
    }
}
