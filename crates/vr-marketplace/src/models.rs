//! ML model marketplace — listing, pricing, and revenue sharing for models.
//!
//! The platform hosts both first-party (platform) models and third-party models
//! from external creators. Revenue sharing differs: platform models generate
//! 100% platform revenue, while third-party models split 75/25 in favor of the creator.

use nexcore_chrono::DateTime;
use serde::{Deserialize, Serialize};
use vr_core::{ModelId, Money, ProviderId, VrError};

/// The domain of a marketplace ML model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelType {
    /// ADME property prediction.
    Adme,
    /// Biological activity / potency prediction.
    Activity,
    /// Toxicity risk prediction.
    Toxicity,
    /// Generative chemistry (de novo design).
    Generative,
    /// Target identification and prioritization.
    TargetPrioritization,
}

/// Publication status of a marketplace model.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ModelStatus {
    /// Uploaded, awaiting review.
    PendingReview,
    /// Approved and available for use.
    Active,
    /// Temporarily unavailable.
    Suspended,
    /// Permanently removed.
    Retired,
}

/// A machine learning model listed in the marketplace.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MarketplaceModel {
    /// Unique model identifier.
    pub id: ModelId,
    /// The external creator, if any. `None` for platform-owned models.
    pub creator_id: Option<ProviderId>,
    /// Display name.
    pub name: String,
    /// Domain of the model.
    pub model_type: ModelType,
    /// Description of what the model predicts and its training data.
    pub description: String,
    /// Semantic version string (e.g., "2.1.0").
    pub version: String,
    /// Path to the model artifact in storage.
    pub artifact_path: String,
    /// Benchmark results (metrics, dataset names, scores).
    pub benchmark_results: serde_json::Value,
    /// Whether this is a platform-owned model (vs. third-party).
    pub is_platform_model: bool,
    /// Price per prediction in cents. `None` for free/bundled models.
    pub pricing_cents: Option<u64>,
    /// Total number of predictions made with this model.
    pub usage_count: u64,
    /// Average rating from 0.0 to 5.0.
    pub rating: f64,
    /// Current listing status.
    pub status: ModelStatus,
    /// When the model was first published.
    pub created_at: DateTime,
}

impl MarketplaceModel {
    /// Create a new platform-owned model.
    #[must_use]
    pub fn new_platform_model(
        name: String,
        model_type: ModelType,
        description: String,
        version: String,
        artifact_path: String,
        pricing_cents: Option<u64>,
    ) -> Self {
        Self {
            id: ModelId::new(),
            creator_id: None,
            name,
            model_type,
            description,
            version,
            artifact_path,
            benchmark_results: serde_json::Value::Null,
            is_platform_model: true,
            pricing_cents,
            usage_count: 0,
            rating: 0.0,
            status: ModelStatus::Active,
            created_at: DateTime::now(),
        }
    }

    /// Create a new third-party model (starts as PendingReview).
    #[must_use]
    pub fn new_third_party_model(
        creator_id: ProviderId,
        name: String,
        model_type: ModelType,
        description: String,
        version: String,
        artifact_path: String,
        pricing_cents: Option<u64>,
    ) -> Self {
        Self {
            id: ModelId::new(),
            creator_id: Some(creator_id),
            name,
            model_type,
            description,
            version,
            artifact_path,
            benchmark_results: serde_json::Value::Null,
            is_platform_model: false,
            pricing_cents,
            usage_count: 0,
            rating: 0.0,
            status: ModelStatus::PendingReview,
            created_at: DateTime::now(),
        }
    }

    /// Whether the model is available for predictions.
    #[must_use]
    pub fn is_available(&self) -> bool {
        self.status == ModelStatus::Active
    }

    /// Validate model fields.
    pub fn validate(&self) -> Result<(), VrError> {
        if self.name.trim().is_empty() {
            return Err(VrError::InvalidInput {
                message: "model name cannot be empty".to_string(),
            });
        }
        if self.version.trim().is_empty() {
            return Err(VrError::InvalidInput {
                message: "model version cannot be empty".to_string(),
            });
        }
        if self.artifact_path.trim().is_empty() {
            return Err(VrError::InvalidInput {
                message: "model artifact_path cannot be empty".to_string(),
            });
        }
        Ok(())
    }
}

/// Calculate revenue share for a single model prediction.
///
/// Returns `(creator_share, platform_share)` where the sum equals the prediction price.
///
/// - **Platform models** (`is_platform_model = true`): 100% goes to platform.
///   Returns `(0, prediction_price)`.
/// - **Third-party models**: 75% to creator, 25% to platform.
///   Returns `(75% of price, 25% of price)`.
///
/// Any remainder from integer division goes to the platform (rounding in platform's favor).
#[must_use]
pub fn calculate_model_revenue_share(
    prediction_price: Money,
    is_platform_model: bool,
) -> (Money, Money) {
    if is_platform_model {
        let zero = Money::from_cents(0, prediction_price.currency());
        return (zero, prediction_price);
    }

    // Third-party: 75% creator, 25% platform.
    // creator_share = price * 7500 / 10000
    let creator_share = prediction_price.percent_bps(7500);
    // platform gets the remainder to avoid rounding loss
    let platform_share = Money::from_cents(
        prediction_price.cents() - creator_share.cents(),
        prediction_price.currency(),
    );
    (creator_share, platform_share)
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    #[test]
    fn platform_model_full_revenue() {
        let price = Money::usd(500); // $5.00
        let (creator, platform) = calculate_model_revenue_share(price, true);
        assert_eq!(creator.cents(), 0);
        assert_eq!(platform.cents(), 500);
    }

    #[test]
    fn third_party_75_25_split() {
        let price = Money::usd(1000); // $10.00
        let (creator, platform) = calculate_model_revenue_share(price, false);
        // 75% of 1000 = 750, 25% = 250
        assert_eq!(creator.cents(), 750);
        assert_eq!(platform.cents(), 250);
    }

    #[test]
    fn third_party_split_sums_to_total() {
        let price = Money::usd(333); // $3.33 — odd amount to test rounding
        let (creator, platform) = calculate_model_revenue_share(price, false);
        // 75% of 333 = 249.75 → 249 (integer truncation via bps)
        // platform gets remainder: 333 - 249 = 84
        assert_eq!(creator.cents(), 249);
        assert_eq!(platform.cents(), 84);
        assert_eq!(creator.cents() + platform.cents(), price.cents());
    }

    #[test]
    fn third_party_split_zero_price() {
        let price = Money::usd(0);
        let (creator, platform) = calculate_model_revenue_share(price, false);
        assert_eq!(creator.cents(), 0);
        assert_eq!(platform.cents(), 0);
    }

    #[test]
    fn third_party_split_small_price() {
        let price = Money::usd(1); // $0.01 — smallest possible
        let (creator, platform) = calculate_model_revenue_share(price, false);
        // 75% of 1 = 0 (integer truncation)
        // platform gets remainder: 1 - 0 = 1
        assert_eq!(creator.cents(), 0);
        assert_eq!(platform.cents(), 1);
        assert_eq!(creator.cents() + platform.cents(), price.cents());
    }

    #[test]
    fn new_platform_model_is_active() {
        let model = MarketplaceModel::new_platform_model(
            "ADME Predictor".to_string(),
            ModelType::Adme,
            "Predicts ADME properties".to_string(),
            "1.0.0".to_string(),
            "/models/adme-v1".to_string(),
            Some(5),
        );
        assert!(model.is_platform_model);
        assert!(model.is_available());
        assert!(model.creator_id.is_none());
        assert_eq!(model.status, ModelStatus::Active);
    }

    #[test]
    fn new_third_party_model_is_pending() {
        let creator = ProviderId::new();
        let model = MarketplaceModel::new_third_party_model(
            creator,
            "ToxNet".to_string(),
            ModelType::Toxicity,
            "Toxicity prediction".to_string(),
            "2.0.0".to_string(),
            "/models/toxnet-v2".to_string(),
            Some(10),
        );
        assert!(!model.is_platform_model);
        assert!(!model.is_available()); // PendingReview
        assert_eq!(model.creator_id, Some(creator));
        assert_eq!(model.status, ModelStatus::PendingReview);
    }

    #[test]
    fn validate_rejects_empty_name() {
        let model = MarketplaceModel::new_platform_model(
            "".to_string(),
            ModelType::Activity,
            "desc".to_string(),
            "1.0.0".to_string(),
            "/path".to_string(),
            None,
        );
        assert!(model.validate().is_err());
    }

    #[test]
    fn validate_rejects_empty_version() {
        let model = MarketplaceModel::new_platform_model(
            "Model".to_string(),
            ModelType::Activity,
            "desc".to_string(),
            "".to_string(),
            "/path".to_string(),
            None,
        );
        assert!(model.validate().is_err());
    }

    #[test]
    fn model_type_serialization() {
        let json = serde_json::to_string(&ModelType::Generative).unwrap();
        assert_eq!(json, "\"Generative\"");
        let back: ModelType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ModelType::Generative);
    }
}
