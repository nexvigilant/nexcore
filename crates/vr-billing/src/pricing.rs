//! Pricing engine — calculates charges from usage aggregations.
//!
//! Rates match the PRPaaS architecture:
//! - $0.01/compound scored
//! - $0.05/ML prediction
//! - $0.10/GB/month storage overage
//! - ~$0.001/KG query (rounded to 0 at cent granularity)
//!
//! Volume discounts apply for high-usage tenants.

use serde::{Deserialize, Serialize};
use vr_core::{Currency, Money, SubscriptionTier};

use crate::metering::UsageAggregation;

/// Per-unit usage rates in cents.
#[derive(Debug, Clone, Copy, Serialize, Deserialize)]
pub struct UsageRates {
    /// Cents per compound scored ($0.01 = 1 cent).
    pub compound_scoring_cents: u64,
    /// Cents per ML prediction ($0.05 = 5 cents).
    pub ml_prediction_cents: u64,
    /// Cents per GB per month of storage overage ($0.10 = 10 cents).
    pub storage_overage_per_gb_cents: u64,
    /// Cents per KG query (rounded to 0 — actually ~$0.001).
    pub kg_query_cents: u64,
}

/// Default platform usage rates.
#[must_use]
pub fn default_rates() -> UsageRates {
    UsageRates {
        compound_scoring_cents: 1,        // $0.01
        ml_prediction_cents: 5,           // $0.05
        storage_overage_per_gb_cents: 10, // $0.10/GB/month
        kg_query_cents: 0,                // ~$0.001, rounds to 0 at cent granularity
    }
}

/// Calculate raw usage charges (no discounts) from aggregated usage.
///
/// Storage overage is calculated by subtracting the tier's included storage
/// allocation. If `tier_storage_bytes` is `None`, no storage overage is charged.
///
/// # Arguments
/// * `rates` - Per-unit pricing
/// * `aggregation` - Aggregated usage for the billing period
/// * `tier_storage_bytes` - Included storage for the tenant's tier
#[must_use]
pub fn calculate_usage_charges(
    rates: &UsageRates,
    aggregation: &UsageAggregation,
    tier_storage_bytes: Option<u64>,
) -> Money {
    // Compound scoring: rate * count
    let compound_charge =
        Money::usd(rates.compound_scoring_cents).times(aggregation.compounds_scored);

    // ML predictions: rate * count
    let ml_charge = Money::usd(rates.ml_prediction_cents).times(aggregation.ml_predictions);

    // Storage overage: only charge for bytes above tier allocation.
    let storage_charge = if let Some(included_bytes) = tier_storage_bytes {
        let overage_bytes = aggregation.storage_bytes.saturating_sub(included_bytes);
        // Convert bytes to GB (integer division, rounded down).
        let overage_gb = overage_bytes / 1_073_741_824;
        Money::usd(rates.storage_overage_per_gb_cents).times(overage_gb)
    } else {
        Money::zero(Currency::USD)
    };

    // KG queries: rate * count (currently 0 cents per query).
    let kg_charge = Money::usd(rates.kg_query_cents).times(aggregation.api_calls);

    compound_charge + ml_charge + storage_charge + kg_charge
}

/// Calculate monthly subscription charge for a given tier.
///
/// Annual billing gets a 16.7% discount (10 months for 12).
#[must_use]
pub fn calculate_subscription_charge(tier: &SubscriptionTier, annual: bool) -> Money {
    if annual {
        // Annual price is for 12 months but costs 10 months' worth.
        // Per-month charge = annual_price / 12.
        // But we bill the monthly installment of the annual commitment.
        let annual_total = tier.annual_price_cents();
        // Monthly installment = annual_total / 12 (integer division).
        let monthly_installment = annual_total / 12;
        Money::usd(monthly_installment)
    } else {
        Money::usd(tier.monthly_price_cents())
    }
}

/// Volume discount thresholds.
///
/// - Compounds scored > 100,000/month: 20% discount on compound scoring charges.
/// - ML predictions > 10,000/month: 15% discount on ML prediction charges.
#[derive(Debug, Clone, Copy)]
pub struct VolumeDiscount {
    /// Compound scoring threshold for discount.
    pub compound_threshold: u64,
    /// Compound scoring discount in basis points (2000 = 20%).
    pub compound_discount_bps: u64,
    /// ML prediction threshold for discount.
    pub ml_threshold: u64,
    /// ML prediction discount in basis points (1500 = 15%).
    pub ml_discount_bps: u64,
}

/// Default volume discount configuration.
#[must_use]
pub fn default_volume_discounts() -> VolumeDiscount {
    VolumeDiscount {
        compound_threshold: 100_000,
        compound_discount_bps: 2_000, // 20%
        ml_threshold: 10_000,
        ml_discount_bps: 1_500, // 15%
    }
}

/// Calculate usage charges with volume discounts applied.
///
/// The discount is applied per-category: if a tenant exceeds the threshold
/// for compound scoring, only the compound scoring line gets discounted.
/// Storage and other charges are unaffected by volume discounts.
#[must_use]
pub fn apply_volume_discounts(
    rates: &UsageRates,
    aggregation: &UsageAggregation,
    tier_storage_bytes: Option<u64>,
) -> Money {
    let discounts = default_volume_discounts();

    // Compound scoring with potential discount.
    let compound_base =
        Money::usd(rates.compound_scoring_cents).times(aggregation.compounds_scored);
    let compound_charge = if aggregation.compounds_scored > discounts.compound_threshold {
        // Apply discount: charge = base * (10000 - discount_bps) / 10000
        let discount_amount = compound_base.percent_bps(discounts.compound_discount_bps);
        compound_base - discount_amount
    } else {
        compound_base
    };

    // ML predictions with potential discount.
    let ml_base = Money::usd(rates.ml_prediction_cents).times(aggregation.ml_predictions);
    let ml_charge = if aggregation.ml_predictions > discounts.ml_threshold {
        let discount_amount = ml_base.percent_bps(discounts.ml_discount_bps);
        ml_base - discount_amount
    } else {
        ml_base
    };

    // Storage overage (no volume discount).
    let storage_charge = if let Some(included_bytes) = tier_storage_bytes {
        let overage_bytes = aggregation.storage_bytes.saturating_sub(included_bytes);
        let overage_gb = overage_bytes / 1_073_741_824;
        Money::usd(rates.storage_overage_per_gb_cents).times(overage_gb)
    } else {
        Money::zero(Currency::USD)
    };

    // KG queries (no volume discount).
    let kg_charge = Money::usd(rates.kg_query_cents).times(aggregation.api_calls);

    compound_charge + ml_charge + storage_charge + kg_charge
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use nexcore_chrono::DateTime;
    use vr_core::TenantId;

    fn make_aggregation(compounds: u64, ml: u64, storage: u64, api: u64) -> UsageAggregation {
        UsageAggregation {
            tenant_id: TenantId::new(),
            period_start: DateTime::from_ymd_hms(2025, 6, 1, 0, 0, 0)
                .unwrap_or_else(|_| DateTime::now()),
            period_end: DateTime::from_ymd_hms(2025, 6, 30, 23, 59, 59)
                .unwrap_or_else(|_| DateTime::now()),
            compounds_scored: compounds,
            ml_predictions: ml,
            virtual_screens: 0,
            storage_bytes: storage,
            api_calls: api,
            cro_order_total_cents: 0,
            marketplace_model_uses: 0,
        }
    }

    #[test]
    fn default_rates_match_architecture() {
        let rates = default_rates();
        assert_eq!(rates.compound_scoring_cents, 1); // $0.01
        assert_eq!(rates.ml_prediction_cents, 5); // $0.05
        assert_eq!(rates.storage_overage_per_gb_cents, 10); // $0.10/GB
        assert_eq!(rates.kg_query_cents, 0);
    }

    #[test]
    fn usage_charges_compound_scoring() {
        let rates = default_rates();
        let agg = make_aggregation(10_000, 0, 0, 0);
        let charge = calculate_usage_charges(&rates, &agg, None);
        // 10,000 compounds * $0.01 = $100.00 = 10,000 cents
        assert_eq!(charge.cents(), 10_000);
        assert_eq!(charge.to_string(), "$100.00");
    }

    #[test]
    fn usage_charges_ml_predictions() {
        let rates = default_rates();
        let agg = make_aggregation(0, 5_000, 0, 0);
        let charge = calculate_usage_charges(&rates, &agg, None);
        // 5,000 predictions * $0.05 = $250.00 = 25,000 cents
        assert_eq!(charge.cents(), 25_000);
        assert_eq!(charge.to_string(), "$250.00");
    }

    #[test]
    fn usage_charges_storage_overage() {
        let rates = default_rates();
        // 75 GB used, tier includes 50 GB (Accelerator)
        let storage_75gb = 75 * 1_073_741_824_u64;
        let tier_50gb = 50 * 1_073_741_824_u64;
        let agg = make_aggregation(0, 0, storage_75gb, 0);
        let charge = calculate_usage_charges(&rates, &agg, Some(tier_50gb));
        // 25 GB overage * $0.10/GB = $2.50 = 250 cents
        assert_eq!(charge.cents(), 250);
        assert_eq!(charge.to_string(), "$2.50");
    }

    #[test]
    fn usage_charges_no_storage_overage_within_tier() {
        let rates = default_rates();
        // 30 GB used, tier includes 50 GB
        let storage_30gb = 30 * 1_073_741_824_u64;
        let tier_50gb = 50 * 1_073_741_824_u64;
        let agg = make_aggregation(0, 0, storage_30gb, 0);
        let charge = calculate_usage_charges(&rates, &agg, Some(tier_50gb));
        assert_eq!(charge.cents(), 0);
    }

    #[test]
    fn usage_charges_combined_realistic() {
        let rates = default_rates();
        // Accelerator tenant: 5,000 compounds, 1,000 ML, 60GB storage, 500 API calls
        let storage_60gb = 60 * 1_073_741_824_u64;
        let tier_50gb = 50 * 1_073_741_824_u64;
        let agg = make_aggregation(5_000, 1_000, storage_60gb, 500);
        let charge = calculate_usage_charges(&rates, &agg, Some(tier_50gb));
        // Compounds: 5,000 * $0.01 = $50.00 (5,000 cents)
        // ML:        1,000 * $0.05 = $50.00 (5,000 cents)
        // Storage:   10 GB over * $0.10 = $1.00 (100 cents)
        // API:       500 * $0.00 = $0.00
        // Total: $101.00 (10,100 cents)
        assert_eq!(charge.cents(), 10_100);
        assert_eq!(charge.to_string(), "$101.00");
    }

    #[test]
    fn subscription_charge_monthly() {
        let charge = calculate_subscription_charge(&SubscriptionTier::Accelerator, false);
        assert_eq!(charge.cents(), 250_000); // $2,500
    }

    #[test]
    fn subscription_charge_annual() {
        let charge = calculate_subscription_charge(&SubscriptionTier::Accelerator, true);
        // Annual = 10 months' worth. Monthly installment = (250,000 * 10) / 12 = 208,333 cents
        let annual_total = 250_000_u64 * 10;
        let expected_monthly = annual_total / 12;
        assert_eq!(charge.cents(), expected_monthly);
        assert_eq!(expected_monthly, 208_333); // $2,083.33
    }

    #[test]
    fn subscription_charge_explorer_monthly() {
        let charge = calculate_subscription_charge(&SubscriptionTier::Explorer, false);
        assert_eq!(charge.cents(), 50_000); // $500
    }

    #[test]
    fn subscription_charge_enterprise_monthly() {
        let charge = calculate_subscription_charge(&SubscriptionTier::Enterprise, false);
        assert_eq!(charge.cents(), 1_000_000); // $10,000
    }

    #[test]
    fn volume_discount_below_threshold_no_discount() {
        let rates = default_rates();
        let agg = make_aggregation(50_000, 5_000, 0, 0);
        let without_discount = calculate_usage_charges(&rates, &agg, None);
        let with_discount = apply_volume_discounts(&rates, &agg, None);
        // Below thresholds: 50K < 100K compounds, 5K < 10K ML
        assert_eq!(without_discount.cents(), with_discount.cents());
    }

    #[test]
    fn volume_discount_compounds_above_threshold() {
        let rates = default_rates();
        // 200,000 compounds (above 100K threshold), 0 ML
        let agg = make_aggregation(200_000, 0, 0, 0);
        let without_discount = calculate_usage_charges(&rates, &agg, None);
        let with_discount = apply_volume_discounts(&rates, &agg, None);
        // Without: 200,000 * 1 cent = $2,000 (200,000 cents)
        assert_eq!(without_discount.cents(), 200_000);
        // With 20% discount: $2,000 - $400 = $1,600 (160,000 cents)
        assert_eq!(with_discount.cents(), 160_000);
    }

    #[test]
    fn volume_discount_ml_above_threshold() {
        let rates = default_rates();
        // 0 compounds, 20,000 ML predictions (above 10K threshold)
        let agg = make_aggregation(0, 20_000, 0, 0);
        let without_discount = calculate_usage_charges(&rates, &agg, None);
        let with_discount = apply_volume_discounts(&rates, &agg, None);
        // Without: 20,000 * 5 cents = $1,000 (100,000 cents)
        assert_eq!(without_discount.cents(), 100_000);
        // With 15% discount: $1,000 - $150 = $850 (85,000 cents)
        assert_eq!(with_discount.cents(), 85_000);
    }

    #[test]
    fn volume_discount_both_above_threshold() {
        let rates = default_rates();
        // 150,000 compounds + 15,000 ML predictions
        let agg = make_aggregation(150_000, 15_000, 0, 0);
        let with_discount = apply_volume_discounts(&rates, &agg, None);
        // Compounds: 150,000 * 1 = 150,000 cents, 20% off = 120,000 cents
        // ML: 15,000 * 5 = 75,000 cents, 15% off = 63,750 cents
        // Total: 120,000 + 63,750 = 183,750 cents = $1,837.50
        assert_eq!(with_discount.cents(), 183_750);
    }

    #[test]
    fn volume_discount_does_not_affect_storage() {
        let rates = default_rates();
        let storage_75gb = 75 * 1_073_741_824_u64;
        let tier_50gb = 50 * 1_073_741_824_u64;
        // 200K compounds + 25GB storage overage
        let agg = make_aggregation(200_000, 0, storage_75gb, 0);
        let with_discount = apply_volume_discounts(&rates, &agg, Some(tier_50gb));
        // Compounds: 200,000 * 1 = 200,000 cents, 20% off = 160,000 cents
        // Storage: 25 GB * 10 cents = 250 cents (no discount)
        // Total: 160,000 + 250 = 160,250 cents
        assert_eq!(with_discount.cents(), 160_250);
    }
}
