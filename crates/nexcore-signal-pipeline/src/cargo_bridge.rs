//! # Cargo ↔ Relay Bridge
//!
//! Bridges the two fidelity tracking systems:
//! - **Cargo system** (`nexcore-cargo`): `StationStamp` / `CustodyChain` — logistics-operational
//! - **Relay system** (`nexcore-primitives`): `RelayHop` / `RelayChain` — information-theoretic
//!
//! Both track the same phenomenon (information preservation per processing hop)
//! at different abstraction levels. The cargo system adds provenance, destination,
//! and perishability on top of relay's fidelity math.
//!
//! ## Direction
//!
//! ```text
//! RelayHop → StationStamp  (enrich: add timestamp + operation context)
//! StationStamp → RelayHop  (project: extract fidelity for axiom verification)
//! CustodyChain → RelayChain (project: verify A1-A5 axioms on cargo's chain)
//! FreightRoute → RelayChain (plan: build expected chain from planned route)
//! ```

use nexcore_cargo::{CustodyChain, FreightRoute, StationStamp, Waypoint};
use nexcore_primitives::relay::{Fidelity, RelayChain, RelayHop};

/// Convert a `StationStamp` to a `RelayHop` for axiom verification.
///
/// Projects the logistics stamp into the information-theoretic domain.
/// Threshold defaults to 0.0 (no activation gating in cargo model).
#[must_use]
pub fn stamp_to_hop(stamp: &StationStamp) -> RelayHop {
    RelayHop::new(
        &stamp.station_id,
        Fidelity::new(stamp.fidelity),
        0.0, // Cargo stamps don't carry activation thresholds
    )
}

/// Convert a `RelayHop` to a `StationStamp`.
///
/// Enriches the relay hop with a timestamp and operation name.
/// Since `RelayHop` has no timestamp, `stamped_at` must be provided.
#[must_use]
pub fn hop_to_stamp(hop: &RelayHop, operation: &str, stamped_at: i64) -> StationStamp {
    StationStamp::new(&hop.stage, operation, stamped_at, hop.fidelity.value())
}

/// Project a `CustodyChain` into a `RelayChain` for axiom verification.
///
/// This lets you verify the 5 relay axioms (A1-A5) on cargo that has
/// already transited through multiple stations. The relay chain inherits
/// the safety-critical minimum fidelity threshold (0.80).
#[must_use]
pub fn custody_to_relay(custody: &CustodyChain) -> RelayChain {
    let mut chain = RelayChain::safety_critical();
    for stamp in custody.stamps() {
        chain.add_hop(stamp_to_hop(stamp));
    }
    chain
}

/// Build a `RelayChain` from a `FreightRoute`'s planned waypoints.
///
/// This projects the planned route into relay-space so you can verify
/// whether the expected fidelity will satisfy safety-critical axioms
/// BEFORE the cargo transits.
#[must_use]
pub fn route_to_relay(route: &FreightRoute) -> RelayChain {
    let mut chain = RelayChain::safety_critical();
    for waypoint in &route.waypoints {
        chain.add_hop(RelayHop::new(
            &waypoint.station_id,
            Fidelity::new(waypoint.expected_fidelity),
            0.0,
        ));
    }
    chain
}

/// Verify that a cargo's custody chain passes the 5 relay axioms.
///
/// Returns `true` if all axioms pass. For detailed verification results,
/// use `custody_to_relay()` then call `.verify()` on the chain.
#[must_use]
pub fn verify_custody_axioms(custody: &CustodyChain) -> bool {
    let chain = custody_to_relay(custody);
    chain.verify().is_valid()
}

/// Check if a planned freight route will satisfy safety-critical fidelity.
///
/// Returns `true` if the product of all waypoint fidelities ≥ 0.80.
/// Use this to reject routes that would degrade signal below threshold.
#[must_use]
pub fn route_passes_safety_critical(route: &FreightRoute) -> bool {
    let chain = route_to_relay(route);
    chain.verify_preservation()
}

/// Build a `FreightRoute`-compatible waypoint from a relay stage's fidelity.
///
/// Useful when constructing cargo routes from relay pipeline definitions.
#[must_use]
pub fn stage_to_waypoint(stage_name: &str, fidelity: f64, description: &str) -> Waypoint {
    Waypoint::new(stage_name, fidelity, description)
}

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_cargo::{Destination, Perishability};

    #[test]
    fn stamp_roundtrips_through_hop() {
        let stamp = StationStamp::new("openfda", "search_events", 1709856000, 0.98);
        let hop = stamp_to_hop(&stamp);
        let back = hop_to_stamp(&hop, "search_events", 1709856000);

        assert_eq!(back.station_id, stamp.station_id);
        assert!((back.fidelity - stamp.fidelity).abs() < 1e-10);
    }

    #[test]
    fn custody_chain_projects_to_relay() {
        let mut custody = CustodyChain::new();
        custody.stamp(StationStamp::new("ingest", "parse", 1000, 0.98));
        custody.stamp(StationStamp::new("detect", "prr", 1001, 0.93));
        custody.stamp(StationStamp::new("threshold", "evans", 1002, 0.97));

        let chain = custody_to_relay(&custody);
        assert_eq!(chain.hop_count(), 3);

        // Verify fidelity products match
        let cargo_f = custody.cumulative_fidelity();
        let relay_f = chain.total_fidelity().value();
        assert!(
            (cargo_f - relay_f).abs() < 1e-10,
            "Cargo fidelity {cargo_f} should match relay fidelity {relay_f}"
        );
    }

    #[test]
    fn route_safety_critical_check() {
        // 3-hop route with good fidelity — should pass
        let mut route = FreightRoute::new(
            "openfda",
            Destination::SignalDetection,
            Perishability::Periodic,
        );
        route.add_waypoint(Waypoint::new("ingest", 0.98, "FAERS parse"));
        route.add_waypoint(Waypoint::new("detect", 0.93, "PRR compute"));
        route.add_waypoint(Waypoint::new("threshold", 0.97, "Evans gate"));

        assert!(
            route_passes_safety_critical(&route),
            "3-hop high-fidelity route should pass safety-critical"
        );
    }

    #[test]
    fn long_route_fails_safety_critical() {
        // 7-hop route — degradation law should push below 0.80
        let mut route = FreightRoute::new(
            "openfda",
            Destination::SignalDetection,
            Perishability::Periodic,
        );
        for (name, f, desc) in [
            ("ingest", 0.98, "parse"),
            ("normalize", 0.96, "standardize"),
            ("detect", 0.93, "PRR"),
            ("threshold", 0.97, "Evans"),
            ("store", 0.99, "persist"),
            ("alert", 0.95, "lifecycle"),
            ("report", 0.96, "format"),
        ] {
            route.add_waypoint(Waypoint::new(name, f, desc));
        }

        assert!(
            !route_passes_safety_critical(&route),
            "7-hop route should fail safety-critical — degradation law"
        );
    }

    #[test]
    fn custody_axiom_verification() {
        let mut custody = CustodyChain::new();
        custody.stamp(StationStamp::new("ingest", "parse", 1000, 0.98));
        custody.stamp(StationStamp::new("detect", "prr", 1001, 0.93));
        custody.stamp(StationStamp::new("threshold", "evans", 1002, 0.97));
        custody.stamp(StationStamp::new("alert", "lifecycle", 1003, 0.95));

        // 4-hop chain: 0.98 * 0.93 * 0.97 * 0.95 ≈ 0.840 — should pass
        assert!(verify_custody_axioms(&custody));
    }

    #[test]
    fn pipeline_stages_to_freight_route() {
        use crate::relay::stage_fidelity;

        let mut route = FreightRoute::new(
            "openfda",
            Destination::SignalDetection,
            Perishability::Periodic,
        );

        // Build route from pipeline stage constants
        route.add_waypoint(stage_to_waypoint(
            "ingest",
            stage_fidelity::INGEST,
            "FAERS data parsing",
        ));
        route.add_waypoint(stage_to_waypoint(
            "detect",
            stage_fidelity::DETECT,
            "PRR/ROR computation",
        ));
        route.add_waypoint(stage_to_waypoint(
            "threshold",
            stage_fidelity::THRESHOLD,
            "Evans criteria gating",
        ));
        route.add_waypoint(stage_to_waypoint(
            "alert",
            stage_fidelity::ALERT,
            "Alert lifecycle wrapping",
        ));

        // Core 4-stage route should pass safety-critical
        assert!(route_passes_safety_critical(&route));
        assert_eq!(route.hop_count(), 4);
    }
}
