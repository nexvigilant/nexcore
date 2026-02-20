//! # nexcore-openfda
//!
//! Generalized OpenFDA REST API client covering all endpoints.
//!
//! ## Endpoints
//!
//! | Endpoint | Function |
//! |---|---|
//! | `/drug/event.json` | [`endpoints::fetch_drug_events`] |
//! | `/drug/label.json` | [`endpoints::fetch_drug_labels`] |
//! | `/drug/enforcement.json` | [`endpoints::fetch_drug_recalls`] |
//! | `/drug/ndc.json` | [`endpoints::fetch_drug_ndc`] |
//! | `/drug/drugsfda.json` | [`endpoints::fetch_drugs_at_fda`] |
//! | `/device/event.json` | [`endpoints::fetch_device_events`] |
//! | `/device/enforcement.json` | [`endpoints::fetch_device_recalls`] |
//! | `/device/510k.json` | [`endpoints::fetch_device_510k`] |
//! | `/device/pma.json` | [`endpoints::fetch_device_pma`] |
//! | `/device/classification.json` | [`endpoints::fetch_device_class`] |
//! | `/device/udi.json` | [`endpoints::fetch_device_udi`] |
//! | `/food/enforcement.json` | [`endpoints::fetch_food_recalls`] |
//! | `/food/event.json` | [`endpoints::fetch_food_events`] |
//! | `/other/substance.json` | [`endpoints::fetch_substances`] |
//!
//! ## Fan-out Search
//!
//! [`search::fan_out_search`] queries all major endpoints concurrently and
//! returns a [`search::FanOutResults`] with per-endpoint results.
//!
//! ## Quick Start
//!
//! ```rust,ignore
//! use nexcore_openfda::{OpenFdaClient, QueryParams};
//! use nexcore_openfda::endpoints::fetch_drug_events;
//! use nexcore_openfda::endpoints::drug::drug_event_search_by_drug;
//!
//! #[tokio::main]
//! async fn main() -> anyhow::Result<()> {
//!     let client = OpenFdaClient::new()?;
//!     let params = QueryParams::search(drug_event_search_by_drug("aspirin"), 20);
//!     let response = fetch_drug_events(&client, &params).await?;
//!     println!("{} events found", response.meta.results.total);
//!     Ok(())
//! }
//! ```

#![forbid(unsafe_code)]
#![deny(clippy::unwrap_used, clippy::expect_used, clippy::panic)]

pub mod client;
pub mod endpoints;
pub mod error;
pub mod search;
pub mod types;

// Top-level re-exports for the most-used items.
pub use client::{OpenFdaClient, QueryParams, MAX_LIMIT};
pub use error::OpenFdaError;
pub use search::{fan_out_search, FanOutResults};
pub use types::common::{OpenFdaEnrichment, OpenFdaMeta, OpenFdaResponse, ResultsMeta};
