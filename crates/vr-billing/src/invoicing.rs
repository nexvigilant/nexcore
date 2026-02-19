//! Invoice generation — builds itemized invoices from usage and subscription data.
//!
//! An [`Invoice`] combines subscription charges, metered usage charges,
//! and marketplace commissions into a single billing document with
//! line-item detail.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use vr_core::{Currency, InvoiceId, Money, SubscriptionTier, TenantId};

use crate::metering::UsageAggregation;
use crate::pricing::{self, UsageRates};

/// A single line item on an invoice.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct InvoiceLineItem {
    /// Human-readable description.
    pub description: String,
    /// Number of units.
    pub quantity: u64,
    /// Price per unit.
    pub unit_price: Money,
    /// Line total (quantity * unit_price).
    pub total: Money,
}

/// Invoice status lifecycle.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum InvoiceStatus {
    /// Being prepared, not yet sent.
    Draft,
    /// Sent to tenant, awaiting payment.
    Issued,
    /// Payment received.
    Paid,
    /// Past due date, payment not received.
    Overdue,
    /// Cancelled / credited.
    Void,
}

/// A complete invoice for a billing period.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Invoice {
    /// Unique invoice identifier.
    pub id: InvoiceId,
    /// Tenant being billed.
    pub tenant_id: TenantId,
    /// Billing period start (inclusive).
    pub period_start: DateTime<Utc>,
    /// Billing period end (exclusive).
    pub period_end: DateTime<Utc>,
    /// Itemized charges.
    pub line_items: Vec<InvoiceLineItem>,
    /// Subscription subtotal.
    pub subscription_total: Money,
    /// Usage subtotal.
    pub usage_total: Money,
    /// Marketplace commission subtotal.
    pub marketplace_total: Money,
    /// Grand total (subscription + usage + marketplace).
    pub grand_total: Money,
    /// Current invoice status.
    pub status: InvoiceStatus,
    /// When the invoice was created.
    pub issued_at: Option<DateTime<Utc>>,
    /// Payment due date.
    pub due_at: Option<DateTime<Utc>>,
}

/// Generate a complete invoice from subscription tier and usage data.
///
/// Builds line items for:
/// 1. Subscription charge (monthly or annual rate)
/// 2. Compound scoring usage
/// 3. ML prediction usage
/// 4. Storage overage (if applicable)
/// 5. CRO order marketplace commissions (from aggregation totals)
/// 6. Marketplace model usage (from aggregation totals)
///
/// Line items with zero quantity are omitted.
///
/// # Arguments
/// * `tenant_id` - The tenant being billed
/// * `tier` - Current subscription tier
/// * `annual` - Whether the tenant is on annual billing
/// * `aggregation` - Usage data for the billing period
/// * `rates` - Per-unit usage rates
/// * `marketplace_commission_cents` - Pre-calculated marketplace commissions
#[must_use]
pub fn generate_invoice(
    tenant_id: TenantId,
    tier: &SubscriptionTier,
    annual: bool,
    aggregation: &UsageAggregation,
    rates: &UsageRates,
    marketplace_commission_cents: u64,
) -> Invoice {
    let mut line_items = Vec::new();

    // 1. Subscription charge
    let subscription_charge = pricing::calculate_subscription_charge(tier, annual);
    let billing_label = if annual { "Annual" } else { "Monthly" };
    line_items.push(InvoiceLineItem {
        description: format!("{tier:?} Plan ({billing_label})"),
        quantity: 1,
        unit_price: subscription_charge,
        total: subscription_charge,
    });

    // 2. Compound scoring
    if aggregation.compounds_scored > 0 {
        let unit_price = Money::usd(rates.compound_scoring_cents);
        let total = unit_price.times(aggregation.compounds_scored);
        line_items.push(InvoiceLineItem {
            description: "Compound Scoring".to_string(),
            quantity: aggregation.compounds_scored,
            unit_price,
            total,
        });
    }

    // 3. ML predictions
    if aggregation.ml_predictions > 0 {
        let unit_price = Money::usd(rates.ml_prediction_cents);
        let total = unit_price.times(aggregation.ml_predictions);
        line_items.push(InvoiceLineItem {
            description: "ML Predictions".to_string(),
            quantity: aggregation.ml_predictions,
            unit_price,
            total,
        });
    }

    // 4. Storage overage
    let tier_storage = tier.storage_bytes();
    let overage_bytes = aggregation.storage_bytes.saturating_sub(tier_storage);
    let overage_gb = overage_bytes / 1_073_741_824;
    if overage_gb > 0 {
        let unit_price = Money::usd(rates.storage_overage_per_gb_cents);
        let total = unit_price.times(overage_gb);
        line_items.push(InvoiceLineItem {
            description: "Storage Overage (per GB/month)".to_string(),
            quantity: overage_gb,
            unit_price,
            total,
        });
    }

    // 5. Marketplace commissions (pre-calculated, added as single line item)
    let marketplace_total = Money::usd(marketplace_commission_cents);
    if marketplace_commission_cents > 0 {
        line_items.push(InvoiceLineItem {
            description: "Marketplace Commission".to_string(),
            quantity: 1,
            unit_price: marketplace_total,
            total: marketplace_total,
        });
    }

    // Calculate subtotals
    let usage_total = pricing::calculate_usage_charges(rates, aggregation, Some(tier_storage));

    let grand_total = subscription_charge + usage_total + marketplace_total;

    Invoice {
        id: InvoiceId::new(),
        tenant_id,
        period_start: aggregation.period_start,
        period_end: aggregation.period_end,
        line_items,
        subscription_total: subscription_charge,
        usage_total,
        marketplace_total,
        grand_total,
        status: InvoiceStatus::Draft,
        issued_at: None,
        due_at: None,
    }
}

#[cfg(test)]
#[allow(clippy::unwrap_used)]
mod tests {
    use super::*;
    use chrono::TimeZone;

    fn make_aggregation(
        compounds: u64,
        ml: u64,
        storage_bytes: u64,
        cro_cents: u64,
    ) -> UsageAggregation {
        UsageAggregation {
            tenant_id: TenantId::new(),
            period_start: Utc.with_ymd_and_hms(2025, 7, 1, 0, 0, 0).unwrap(),
            period_end: Utc.with_ymd_and_hms(2025, 7, 31, 23, 59, 59).unwrap(),
            compounds_scored: compounds,
            ml_predictions: ml,
            virtual_screens: 0,
            storage_bytes,
            api_calls: 0,
            cro_order_total_cents: cro_cents,
            marketplace_model_uses: 0,
        }
    }

    #[test]
    fn invoice_subscription_only() {
        let tid = TenantId::new();
        let rates = pricing::default_rates();
        let agg = make_aggregation(0, 0, 0, 0);
        let inv = generate_invoice(tid, &SubscriptionTier::Explorer, false, &agg, &rates, 0);

        assert_eq!(inv.status, InvoiceStatus::Draft);
        assert_eq!(inv.subscription_total.cents(), 50_000); // $500
        assert_eq!(inv.usage_total.cents(), 0);
        assert_eq!(inv.marketplace_total.cents(), 0);
        assert_eq!(inv.grand_total.cents(), 50_000);
        // Only subscription line item.
        assert_eq!(inv.line_items.len(), 1);
        assert!(inv.line_items[0].description.contains("Explorer"));
    }

    #[test]
    fn invoice_accelerator_with_usage() {
        let tid = TenantId::new();
        let rates = pricing::default_rates();
        // 10K compounds, 2K ML, 60GB storage (Accelerator has 50GB)
        let storage_60gb = 60 * 1_073_741_824_u64;
        let agg = make_aggregation(10_000, 2_000, storage_60gb, 0);
        let inv = generate_invoice(tid, &SubscriptionTier::Accelerator, false, &agg, &rates, 0);

        // Subscription: $2,500
        assert_eq!(inv.subscription_total.cents(), 250_000);
        // Usage: 10K * $0.01 + 2K * $0.05 + 10GB * $0.10
        //      = $100 + $100 + $1.00 = $201.00
        assert_eq!(inv.usage_total.cents(), 20_100);
        // Grand total: $2,500 + $201 = $2,701
        assert_eq!(inv.grand_total.cents(), 270_100);
        // 4 line items: subscription + compounds + ML + storage
        assert_eq!(inv.line_items.len(), 4);
    }

    #[test]
    fn invoice_with_marketplace_commission() {
        let tid = TenantId::new();
        let rates = pricing::default_rates();
        let agg = make_aggregation(5_000, 0, 0, 0);
        // $500 in marketplace commissions
        let inv = generate_invoice(
            tid,
            &SubscriptionTier::Accelerator,
            false,
            &agg,
            &rates,
            50_000,
        );

        assert_eq!(inv.marketplace_total.cents(), 50_000); // $500
        // Subscription: 250,000 cents ($2,500)
        // Usage: 5,000 * 1 = 5,000 cents ($50)
        // Marketplace: 50,000 cents ($500)
        // Grand: 250,000 + 5,000 + 50,000 = 305,000
        assert_eq!(inv.grand_total.cents(), 305_000);
    }

    #[test]
    fn invoice_annual_billing() {
        let tid = TenantId::new();
        let rates = pricing::default_rates();
        let agg = make_aggregation(0, 0, 0, 0);
        let inv = generate_invoice(tid, &SubscriptionTier::Enterprise, true, &agg, &rates, 0);

        // Annual Enterprise: (1,000,000 * 10) / 12 = 833,333 cents per month
        let expected = (1_000_000_u64 * 10) / 12;
        assert_eq!(inv.subscription_total.cents(), expected);
        assert_eq!(expected, 833_333);
        assert!(inv.line_items[0].description.contains("Annual"));
    }

    #[test]
    fn invoice_zero_quantity_items_omitted() {
        let tid = TenantId::new();
        let rates = pricing::default_rates();
        // Only compound scoring, no ML, no storage overage, no marketplace
        let agg = make_aggregation(1_000, 0, 0, 0);
        let inv = generate_invoice(tid, &SubscriptionTier::Accelerator, false, &agg, &rates, 0);

        // Should have 2 items: subscription + compounds (no ML, no storage, no marketplace)
        assert_eq!(inv.line_items.len(), 2);
    }

    #[test]
    fn invoice_enterprise_high_volume() {
        let tid = TenantId::new();
        let rates = pricing::default_rates();
        // Enterprise: 500K compounds, 50K ML, 600GB storage (500GB included), $200K CRO
        let storage_600gb = 600 * 1_073_741_824_u64;
        let agg = make_aggregation(500_000, 50_000, storage_600gb, 20_000_000);
        // Marketplace commission on $200K CRO at ~6% = $12,000 = 1,200,000 cents
        let inv = generate_invoice(
            tid,
            &SubscriptionTier::Enterprise,
            false,
            &agg,
            &rates,
            1_200_000,
        );

        // Subscription: $10,000 (1,000,000 cents)
        assert_eq!(inv.subscription_total.cents(), 1_000_000);
        // Usage:
        //   Compounds: 500,000 * 1 = 500,000 cents ($5,000)
        //   ML: 50,000 * 5 = 250,000 cents ($2,500)
        //   Storage: 100GB overage * 10 = 1,000 cents ($10)
        //   Total usage: 751,000 cents ($7,510)
        assert_eq!(inv.usage_total.cents(), 751_000);
        // Marketplace: $12,000 (1,200,000 cents)
        assert_eq!(inv.marketplace_total.cents(), 1_200_000);
        // Grand total: $10,000 + $7,510 + $12,000 = $29,510 (2,951,000 cents)
        assert_eq!(inv.grand_total.cents(), 2_951_000);
        // 5 line items: subscription + compounds + ML + storage + marketplace
        assert_eq!(inv.line_items.len(), 5);
    }

    #[test]
    fn invoice_period_matches_aggregation() {
        let tid = TenantId::new();
        let rates = pricing::default_rates();
        let agg = make_aggregation(0, 0, 0, 0);
        let inv = generate_invoice(tid, &SubscriptionTier::Explorer, false, &agg, &rates, 0);

        assert_eq!(inv.period_start, agg.period_start);
        assert_eq!(inv.period_end, agg.period_end);
        assert_eq!(inv.tenant_id, tid);
        assert!(inv.issued_at.is_none());
        assert!(inv.due_at.is_none());
    }
}
