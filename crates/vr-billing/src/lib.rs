//! # vr-billing — PRPaaS Billing Engine
//!
//! Usage metering, pricing calculations, invoice generation, and
//! marketplace commission logic for the Pharmaceutical Research Platform.
//!
//! ## Modules
//!
//! - [`metering`] — Track and aggregate platform usage events.
//! - [`pricing`] — Calculate charges from usage with volume discounts.
//! - [`invoicing`] — Generate itemized invoices.
//! - [`commission`] — Marketplace commission calculations (CRO, AI models, experts, deals).
//!
//! ## Design Principles
//!
//! All financial amounts use [`vr_core::Money`] (integer cents). No floating point
//! is used in any monetary calculation. Commission rates use basis points via
//! [`Money::percent_bps`] to maintain precision.

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod commission;
pub mod invoicing;
pub mod metering;
pub mod pricing;

// Re-export key types at crate root.
pub use commission::{
    CommissionRate, CommissionSummary, calculate_cro_commission, calculate_deal_commission,
    calculate_expert_commission, calculate_model_commission,
};
pub use invoicing::{Invoice, InvoiceLineItem, InvoiceStatus, generate_invoice};
pub use metering::{MeterEvent, MeterType, UsageAggregation, aggregate_events};
pub use pricing::{
    UsageRates, VolumeDiscount, apply_volume_discounts, calculate_subscription_charge,
    calculate_usage_charges, default_rates, default_volume_discounts,
};
