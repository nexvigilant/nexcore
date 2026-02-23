//! Service catalog — listings of services available through the marketplace.
//!
//! Each catalog entry represents a specific service offered by a provider,
//! with pricing, turnaround, and specification details. Tenants browse the
//! catalog to find and order services.

use nexcore_id::NexId;
use serde::{Deserialize, Serialize};
use vr_core::{Currency, Money, ProviderId, VrError};

/// Unique identifier for a catalog entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
#[serde(transparent)]
pub struct CatalogEntryId(NexId);

impl CatalogEntryId {
    /// Create a new random catalog entry ID.
    #[must_use]
    pub fn new() -> Self {
        Self(NexId::v4())
    }
}

impl Default for CatalogEntryId {
    fn default() -> Self {
        Self::new()
    }
}

/// The category of service being offered.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum ServiceType {
    /// Chemical synthesis (compound manufacturing).
    Synthesis,
    /// Biological or biochemical assays.
    Assay,
    /// ADME (absorption, distribution, metabolism, excretion) panel.
    AdmePanel,
    /// In vivo animal studies.
    InVivoStudy,
    /// Analytical chemistry (mass spec, NMR, etc.).
    Analytical,
    /// Machine learning model access.
    MlModel,
    /// Expert consulting services.
    Consulting,
    /// Licensed dataset access.
    DataLicense,
}

/// How the service is priced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum PricingModel {
    /// Fixed price per unit (e.g., per compound, per assay plate).
    PerUnit,
    /// Fixed price for the entire project scope.
    PerProject,
    /// Recurring subscription fee.
    Subscription,
    /// Pay per use (e.g., per API call, per prediction).
    Usage,
}

/// Catalog entry status.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum CatalogEntryStatus {
    /// Visible and orderable.
    Active,
    /// Temporarily unavailable (provider paused it).
    Paused,
    /// Permanently removed.
    Retired,
}

/// A service listing in the marketplace catalog.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CatalogEntry {
    /// Unique entry identifier.
    pub id: CatalogEntryId,
    /// The provider offering this service.
    pub provider_id: ProviderId,
    /// Category of service.
    pub service_type: ServiceType,
    /// Display name for this service listing.
    pub name: String,
    /// Detailed description of what the service includes.
    pub description: String,
    /// How the service is priced.
    pub pricing_model: PricingModel,
    /// Base price per unit or per project (in cents).
    pub base_price: Money,
    /// Currency for pricing.
    pub currency: Currency,
    /// Estimated turnaround in business days.
    pub turnaround_days: u32,
    /// Additional specifications (assay conditions, formats, limits, etc.).
    pub specifications: serde_json::Value,
    /// Current listing status.
    pub status: CatalogEntryStatus,
}

impl CatalogEntry {
    /// Whether this entry can be ordered.
    #[must_use]
    pub fn is_orderable(&self) -> bool {
        self.status == CatalogEntryStatus::Active
    }

    /// Validate catalog entry fields.
    pub fn validate(&self) -> Result<(), VrError> {
        if self.name.trim().is_empty() {
            return Err(VrError::InvalidInput {
                message: "catalog entry name cannot be empty".to_string(),
            });
        }
        if self.turnaround_days == 0 {
            return Err(VrError::InvalidInput {
                message: "turnaround_days must be at least 1".to_string(),
            });
        }
        Ok(())
    }
}

/// Estimate the total cost for an order of the given quantity.
///
/// For `PerUnit` and `Usage` pricing, multiplies base_price by quantity.
/// For `PerProject` and `Subscription`, the quantity is ignored and the
/// base price is returned as-is (one project = one price).
#[must_use]
pub fn estimate_order_cost(entry: &CatalogEntry, quantity: u32) -> Money {
    match entry.pricing_model {
        PricingModel::PerUnit | PricingModel::Usage => entry.base_price.times(u64::from(quantity)),
        PricingModel::PerProject | PricingModel::Subscription => entry.base_price,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;

    fn make_entry(pricing_model: PricingModel, base_price_cents: u64) -> CatalogEntry {
        CatalogEntry {
            id: CatalogEntryId::new(),
            provider_id: ProviderId::new(),
            service_type: ServiceType::Assay,
            name: "Kinase Assay Panel".to_string(),
            description: "Full kinase selectivity panel".to_string(),
            pricing_model,
            base_price: Money::usd(base_price_cents),
            currency: Currency::USD,
            turnaround_days: 14,
            specifications: serde_json::json!({ "kinases": 468 }),
            status: CatalogEntryStatus::Active,
        }
    }

    #[test]
    fn estimate_per_unit_cost() {
        let entry = make_entry(PricingModel::PerUnit, 500); // $5.00 per unit
        let cost = estimate_order_cost(&entry, 10);
        assert_eq!(cost.cents(), 5000); // $50.00
    }

    #[test]
    fn estimate_per_project_ignores_quantity() {
        let entry = make_entry(PricingModel::PerProject, 500_000); // $5,000.00
        let cost = estimate_order_cost(&entry, 100);
        assert_eq!(cost.cents(), 500_000); // still $5,000.00
    }

    #[test]
    fn estimate_usage_pricing() {
        let entry = make_entry(PricingModel::Usage, 5); // $0.05 per use
        let cost = estimate_order_cost(&entry, 1000);
        assert_eq!(cost.cents(), 5000); // $50.00
    }

    #[test]
    fn estimate_subscription_ignores_quantity() {
        let entry = make_entry(PricingModel::Subscription, 99_900); // $999/month
        let cost = estimate_order_cost(&entry, 12);
        assert_eq!(cost.cents(), 99_900); // still $999.00 (single period)
    }

    #[test]
    fn validate_rejects_empty_name() {
        let mut entry = make_entry(PricingModel::PerUnit, 100);
        entry.name = "".to_string();
        let err = entry.validate().unwrap_err();
        let msg = format!("{err}");
        assert!(msg.contains("name"), "expected name error, got: {msg}");
    }

    #[test]
    fn validate_rejects_zero_turnaround() {
        let mut entry = make_entry(PricingModel::PerUnit, 100);
        entry.turnaround_days = 0;
        let err = entry.validate().unwrap_err();
        let msg = format!("{err}");
        assert!(
            msg.contains("turnaround"),
            "expected turnaround error, got: {msg}"
        );
    }

    #[test]
    fn active_entry_is_orderable() {
        let entry = make_entry(PricingModel::PerUnit, 100);
        assert!(entry.is_orderable());
    }

    #[test]
    fn paused_entry_is_not_orderable() {
        let mut entry = make_entry(PricingModel::PerUnit, 100);
        entry.status = CatalogEntryStatus::Paused;
        assert!(!entry.is_orderable());
    }

    #[test]
    fn service_type_serialization() {
        let json = serde_json::to_string(&ServiceType::AdmePanel).unwrap();
        assert_eq!(json, "\"AdmePanel\"");
        let back: ServiceType = serde_json::from_str(&json).unwrap();
        assert_eq!(back, ServiceType::AdmePanel);
    }
}
