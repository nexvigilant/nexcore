//! Provider management — CROs, model creators, experts, and data providers.
//!
//! Providers are third-party organizations or individuals who offer services
//! through the marketplace. Each provider goes through a review process before
//! becoming active, and maintains a rating based on customer reviews.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use vr_core::{ProviderId, VrError};

/// The type of service a provider offers.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderType {
    /// Contract Research Organization — performs wet-lab experiments.
    Cro,
    /// Creates and maintains ML/AI models for the platform.
    ModelCreator,
    /// Subject-matter expert offering consulting services.
    Expert,
    /// Provides licensed datasets (compound libraries, assay data, etc.).
    DataProvider,
}

/// Review status for provider applications.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ProviderStatus {
    /// Application submitted, awaiting platform review.
    PendingReview,
    /// Approved and visible in the marketplace.
    Active,
    /// Temporarily suspended (quality issues, complaints, etc.).
    Suspended,
    /// Application denied.
    Rejected,
}

/// A marketplace provider.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Provider {
    /// Unique provider identifier.
    pub id: ProviderId,
    /// What kind of services this provider offers.
    pub provider_type: ProviderType,
    /// Display name.
    pub name: String,
    /// Detailed description of capabilities and specialties.
    pub description: String,
    /// List of specific capabilities (e.g., "ADME screening", "kinase assays").
    pub capabilities: Vec<String>,
    /// Average rating from 0.0 to 5.0.
    pub rating: f64,
    /// Total number of reviews received.
    pub review_count: u32,
    /// Current marketplace status.
    pub status: ProviderStatus,
    /// Primary contact email.
    pub contact_email: String,
    /// When the provider was registered.
    pub created_at: DateTime,
}

impl Provider {
    /// Create a new provider application (starts as `PendingReview`).
    #[must_use]
    pub fn new(
        provider_type: ProviderType,
        name: String,
        description: String,
        capabilities: Vec<String>,
        contact_email: String,
    ) -> Self {
        Self {
            id: ProviderId::new(),
            provider_type,
            name,
            description,
            capabilities,
            rating: 0.0,
            review_count: 0,
            status: ProviderStatus::PendingReview,
            contact_email,
            created_at: DateTime::now(),
        }
    }

    /// Whether this provider is visible and can accept orders.
    #[must_use]
    pub fn is_active(&self) -> bool {
        self.status == ProviderStatus::Active
    }

    /// Validate that the provider fields are acceptable.
    pub fn validate(&self) -> Result<(), VrError> {
        if self.name.trim().is_empty() {
            return Err(VrError::InvalidInput {
                message: "provider name cannot be empty".to_string(),
            });
        }
        if self.contact_email.trim().is_empty() || !self.contact_email.contains('@') {
            return Err(VrError::InvalidInput {
                message: "provider contact_email must be a valid email".to_string(),
            });
        }
        Ok(())
    }
}

/// Calculate an updated weighted running average rating.
///
/// Uses the standard incremental mean formula:
/// `new_avg = (old_avg * old_count + new_score) / (old_count + 1)`
///
/// The `new_score` is clamped to `[0.0, 5.0]`.
///
/// Returns `(new_rating, new_count)`.
#[must_use]
pub fn update_rating(current_rating: f64, current_count: u32, new_score: f64) -> (f64, u32) {
    let clamped = new_score.clamp(0.0, 5.0);
    let new_count = current_count + 1;
    let total = current_rating * f64::from(current_count) + clamped;
    let new_rating = total / f64::from(new_count);
    (new_rating, new_count)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn new_provider_starts_pending() {
        let provider = Provider::new(
            ProviderType::Cro,
            "PharmaCRO Inc.".to_string(),
            "Full-service CRO".to_string(),
            vec!["ADME".to_string(), "PK studies".to_string()],
            "contact@pharmacro.com".to_string(),
        );
        assert_eq!(provider.status, ProviderStatus::PendingReview);
        assert_eq!(provider.rating, 0.0);
        assert_eq!(provider.review_count, 0);
        assert!(!provider.is_active());
    }

    #[test]
    fn validate_rejects_empty_name() {
        let provider = Provider::new(
            ProviderType::Expert,
            "".to_string(),
            "desc".to_string(),
            vec![],
            "a@b.com".to_string(),
        );
        let err = provider.validate().unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("name"), "expected name error, got: {msg}");
    }

    #[test]
    fn validate_rejects_invalid_email() {
        let provider = Provider::new(
            ProviderType::DataProvider,
            "DataCo".to_string(),
            "desc".to_string(),
            vec![],
            "not-an-email".to_string(),
        );
        let err = provider.validate().unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("email"), "expected email error, got: {msg}");
    }

    #[test]
    fn update_rating_first_review() {
        let (rating, count) = update_rating(0.0, 0, 4.0);
        assert_eq!(count, 1);
        assert!((rating - 4.0).abs() < f64::EPSILON);
    }

    #[test]
    fn update_rating_running_average() {
        // Start with 4.0 from 1 review, add 2.0 → expect 3.0
        let (rating, count) = update_rating(4.0, 1, 2.0);
        assert_eq!(count, 2);
        assert!((rating - 3.0).abs() < f64::EPSILON);
    }

    #[test]
    fn update_rating_many_reviews() {
        // 10 reviews at 4.5 average, add a 5.0 → (4.5*10 + 5.0) / 11 = 50/11 ≈ 4.5454...
        let (rating, count) = update_rating(4.5, 10, 5.0);
        assert_eq!(count, 11);
        let expected = 50.0 / 11.0;
        assert!((rating - expected).abs() < 1e-10);
    }

    #[test]
    fn update_rating_clamps_score() {
        // Score above 5.0 gets clamped to 5.0
        let (rating, count) = update_rating(0.0, 0, 10.0);
        assert_eq!(count, 1);
        assert!((rating - 5.0).abs() < f64::EPSILON);

        // Score below 0.0 gets clamped to 0.0
        let (rating, count) = update_rating(3.0, 1, -2.0);
        assert_eq!(count, 2);
        assert!((rating - 1.5).abs() < f64::EPSILON);
    }

    #[test]
    fn provider_type_serialization() {
        let json = serde_json::to_string(&ProviderType::Cro).unwrap();
        assert_eq!(json, "\"Cro\"");
        let back: ProviderType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ProviderType::Cro);
    }
}
