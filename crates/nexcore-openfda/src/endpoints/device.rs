//! OpenFDA device endpoint functions.
//!
//! Wraps the six device endpoints:
//! - `/device/event.json` — MDR adverse event reports
//! - `/device/enforcement.json` — device recalls
//! - `/device/510k.json` — 510(k) premarket notifications
//! - `/device/pma.json` — premarket approvals
//! - `/device/classification.json` — device classification
//! - `/device/udi.json` — unique device identifiers (GUDID)

use crate::client::{OpenFdaClient, QueryParams};
use crate::types::common::OpenFdaResponse;
use crate::types::device::{
    Device510k, DeviceClass, DeviceEvent, DevicePma, DeviceRecall, DeviceUdi,
};

/// OpenFDA endpoint path for device adverse events (MDR).
pub const DEVICE_EVENT_ENDPOINT: &str = "/device/event.json";
/// OpenFDA endpoint path for device enforcement/recalls.
pub const DEVICE_RECALL_ENDPOINT: &str = "/device/enforcement.json";
/// OpenFDA endpoint path for 510(k) clearances.
pub const DEVICE_510K_ENDPOINT: &str = "/device/510k.json";
/// OpenFDA endpoint path for PMA approvals.
pub const DEVICE_PMA_ENDPOINT: &str = "/device/pma.json";
/// OpenFDA endpoint path for device classification.
pub const DEVICE_CLASS_ENDPOINT: &str = "/device/classification.json";
/// OpenFDA endpoint path for UDI/GUDID records.
pub const DEVICE_UDI_ENDPOINT: &str = "/device/udi.json";

// =============================================================================
// Endpoint Functions
// =============================================================================

/// Fetch Medical Device Report (MDR) adverse event records.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_device_events(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<DeviceEvent>, nexcore_error::NexError> {
    client
        .fetch::<DeviceEvent>(DEVICE_EVENT_ENDPOINT, params)
        .await
}

/// Fetch device enforcement/recall records.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_device_recalls(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<DeviceRecall>, nexcore_error::NexError> {
    client
        .fetch::<DeviceRecall>(DEVICE_RECALL_ENDPOINT, params)
        .await
}

/// Fetch 510(k) premarket notification clearance records.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_device_510k(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<Device510k>, nexcore_error::NexError> {
    client
        .fetch::<Device510k>(DEVICE_510K_ENDPOINT, params)
        .await
}

/// Fetch Premarket Approval (PMA) records for Class III devices.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_device_pma(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<DevicePma>, nexcore_error::NexError> {
    client
        .fetch::<DevicePma>(DEVICE_PMA_ENDPOINT, params)
        .await
}

/// Fetch device classification records.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_device_class(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<DeviceClass>, nexcore_error::NexError> {
    client
        .fetch::<DeviceClass>(DEVICE_CLASS_ENDPOINT, params)
        .await
}

/// Fetch Unique Device Identifier (UDI) records from the GUDID.
///
/// # Errors
///
/// Returns error if the API is unreachable and no cache exists.
pub async fn fetch_device_udi(
    client: &OpenFdaClient,
    params: &QueryParams,
) -> Result<OpenFdaResponse<DeviceUdi>, nexcore_error::NexError> {
    client
        .fetch::<DeviceUdi>(DEVICE_UDI_ENDPOINT, params)
        .await
}

// =============================================================================
// Query Builders
// =============================================================================

/// Build a device event search string matching by brand name.
#[must_use]
pub fn device_event_search_by_name(name: &str) -> String {
    format!("device.brand_name:\"{}\"", name)
}

/// Build a device event search string for a specific event type.
///
/// Common values: "Malfunction", "Injury", "Death", "Other".
#[must_use]
pub fn device_event_search_by_type(event_type: &str) -> String {
    format!("event_type:\"{}\"", event_type)
}

/// Build a 510(k) search string by device name.
#[must_use]
pub fn device_510k_search_by_name(name: &str) -> String {
    format!("device_name:\"{}\"", name)
}

/// Build a classification search string by product code.
#[must_use]
pub fn device_class_search_by_product_code(code: &str) -> String {
    format!("product_code:\"{}\"", code)
}

/// Build a UDI search by company name.
#[must_use]
pub fn device_udi_search_by_company(company: &str) -> String {
    format!("company_name:\"{}\"", company)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn device_event_search_contains_name() {
        let q = device_event_search_by_name("PACEMAKER");
        assert!(q.contains("brand_name"));
        assert!(q.contains("PACEMAKER"));
    }

    #[test]
    fn device_510k_search_contains_name() {
        let q = device_510k_search_by_name("catheter");
        assert!(q.contains("device_name"));
        assert!(q.contains("catheter"));
    }

    #[test]
    fn device_class_search_product_code() {
        let q = device_class_search_by_product_code("FPA");
        assert!(q.contains("product_code"));
        assert!(q.contains("FPA"));
    }
}
