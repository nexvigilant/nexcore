//! Communication link types and routing.
//!
//! 7 link types across 4 domains: short-range (BLE, Wi-Fi 6E),
//! long-range (5G, LoRa, Iridium Certus), air-band (VHF, ADS-B Out).

use serde::{Deserialize, Serialize};

/// Communication link type.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Serialize, Deserialize)]
pub enum LinkType {
    /// Bluetooth Low Energy 5.0 — controller pairing, sensor sync.
    Ble5,
    /// Wi-Fi 6E (6 GHz) — ground station high-bandwidth.
    Wifi6e,
    /// 5G NR modem — primary cloud link.
    FiveGModem,
    /// LoRa SX1276 — long-range telemetry fallback.
    LoraSx1276,
    /// Iridium Certus — satellite, last resort.
    IridiumCertus,
    /// VHF aviation radio — ATC communication.
    VhfRadio,
    /// ADS-B Out transponder — position broadcast.
    AdsbOut,
}

/// Communication domain classification.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum CommsDomain {
    /// <100m range, high bandwidth.
    ShortRange,
    /// 1-1000+ km, variable bandwidth.
    LongRange,
    /// Aviation frequency bands, regulated.
    AirBand,
}

impl LinkType {
    /// Which domain this link belongs to.
    pub fn domain(&self) -> CommsDomain {
        match self {
            Self::Ble5 | Self::Wifi6e => CommsDomain::ShortRange,
            Self::FiveGModem | Self::LoraSx1276 | Self::IridiumCertus => CommsDomain::LongRange,
            Self::VhfRadio | Self::AdsbOut => CommsDomain::AirBand,
        }
    }

    /// Maximum theoretical data rate.
    pub fn max_data_rate_bps(&self) -> u64 {
        match self {
            Self::Ble5 => 2_000_000,           // 2 Mbps
            Self::Wifi6e => 2_400_000_000,     // 2.4 Gbps
            Self::FiveGModem => 1_000_000_000, // 1 Gbps
            Self::LoraSx1276 => 50_000,        // 50 kbps max
            Self::IridiumCertus => 704_000,    // 704 kbps
            Self::VhfRadio => 8_330,           // 8.33 kHz channel
            Self::AdsbOut => 1_000_000,        // 1 Mbps (1090 MHz)
        }
    }

    /// Whether encryption is allowed on this link.
    pub fn encryption_allowed(&self) -> bool {
        match self {
            // Aviation bands are unencrypted by regulation
            Self::VhfRadio | Self::AdsbOut => false,
            _ => true,
        }
    }

    /// Typical range in meters.
    pub fn typical_range_m(&self) -> u32 {
        match self {
            Self::Ble5 => 100,
            Self::Wifi6e => 200,
            Self::FiveGModem => 10_000,
            Self::LoraSx1276 => 15_000,
            Self::IridiumCertus => u32::MAX, // Global
            Self::VhfRadio => 200_000,       // Line of sight
            Self::AdsbOut => 460_000,        // ~250 NM
        }
    }
}

/// Link health state.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum LinkStatus {
    /// Connected, signal strong.
    Connected,
    /// Connected but signal weak.
    Weak,
    /// Not available (out of range, modem off, no coverage).
    Unavailable,
}

/// Data rate requirement class.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum DataRateClass {
    /// GPS + fault codes only (<1 kbps).
    TelemetryOnly,
    /// Sensor data + commands (<100 kbps).
    LowBandwidth,
    /// Logs + status updates (<10 Mbps).
    MediumBandwidth,
    /// Real-time telemetry + control (<100 Mbps).
    HighBandwidth,
    /// Live video stream (>100 Mbps).
    Video,
}

/// Message criticality level.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum Criticality {
    /// Normal operations.
    Routine,
    /// Elevated importance, expedited delivery.
    Priority,
    /// Life-safety, immediate delivery.
    Emergency,
    /// Air Traffic Control — legally mandatory, VHF only.
    AtcMandatory,
}

/// Routing decision for a communication request.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LinkRoute {
    /// Selected primary link.
    pub primary: LinkType,
    /// Fallback link if primary fails.
    pub fallback: Option<LinkType>,
    /// Why this link was selected.
    pub rationale: &'static str,
}

/// Route a communication request to the optimal link.
pub fn route_link(
    range_km: f64,
    data_rate: DataRateClass,
    criticality: Criticality,
    airborne: bool,
    five_g_status: LinkStatus,
    wifi_available: bool,
) -> LinkRoute {
    // ATC mandatory — no alternative
    if criticality == Criticality::AtcMandatory {
        return LinkRoute {
            primary: LinkType::VhfRadio,
            fallback: None,
            rationale: "ATC communication is legally mandatory — VHF only",
        };
    }

    // Airborne with 5G
    if airborne && five_g_status == LinkStatus::Connected {
        return LinkRoute {
            primary: LinkType::FiveGModem,
            fallback: Some(LinkType::LoraSx1276),
            rationale: "Airborne with 5G coverage — primary cloud link",
        };
    }

    // Airborne without 5G
    if airborne && five_g_status != LinkStatus::Connected {
        if criticality == Criticality::Emergency {
            return LinkRoute {
                primary: LinkType::IridiumCertus,
                fallback: Some(LinkType::LoraSx1276),
                rationale: "Airborne emergency, no 5G — satcom last resort",
            };
        }
        return LinkRoute {
            primary: LinkType::LoraSx1276,
            fallback: Some(LinkType::IridiumCertus),
            rationale: "Airborne, no 5G — LoRa telemetry fallback",
        };
    }

    // Ground: Wi-Fi for high bandwidth
    if wifi_available && data_rate >= DataRateClass::HighBandwidth {
        return LinkRoute {
            primary: LinkType::Wifi6e,
            fallback: Some(LinkType::FiveGModem),
            rationale: "Ground station Wi-Fi for high bandwidth / video",
        };
    }

    // Ground: 5G for medium range
    if five_g_status == LinkStatus::Connected {
        return LinkRoute {
            primary: LinkType::FiveGModem,
            fallback: Some(LinkType::LoraSx1276),
            rationale: "5G connected — primary cloud link",
        };
    }

    // Short range: BLE
    if range_km <= 0.1 {
        return LinkRoute {
            primary: LinkType::Ble5,
            fallback: Some(LinkType::Wifi6e),
            rationale: "Short range — BLE controller pairing",
        };
    }

    // Medium range: LoRa
    if range_km <= 15.0 {
        return LinkRoute {
            primary: LinkType::LoraSx1276,
            fallback: Some(LinkType::IridiumCertus),
            rationale: "Medium range, no IP links — LoRa telemetry",
        };
    }

    // Beyond terrestrial: satcom
    LinkRoute {
        primary: LinkType::IridiumCertus,
        fallback: None,
        rationale: "Beyond terrestrial coverage — satellite last resort",
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_atc_mandatory_always_vhf() {
        let r = route_link(
            5.0,
            DataRateClass::TelemetryOnly,
            Criticality::AtcMandatory,
            true,
            LinkStatus::Connected,
            false,
        );
        assert_eq!(r.primary, LinkType::VhfRadio);
        assert!(r.fallback.is_none());
    }

    #[test]
    fn test_airborne_5g_connected() {
        let r = route_link(
            50.0,
            DataRateClass::MediumBandwidth,
            Criticality::Routine,
            true,
            LinkStatus::Connected,
            false,
        );
        assert_eq!(r.primary, LinkType::FiveGModem);
    }

    #[test]
    fn test_airborne_emergency_no_5g() {
        let r = route_link(
            100.0,
            DataRateClass::TelemetryOnly,
            Criticality::Emergency,
            true,
            LinkStatus::Unavailable,
            false,
        );
        assert_eq!(r.primary, LinkType::IridiumCertus);
    }

    #[test]
    fn test_airborne_routine_no_5g() {
        let r = route_link(
            50.0,
            DataRateClass::TelemetryOnly,
            Criticality::Routine,
            true,
            LinkStatus::Unavailable,
            false,
        );
        assert_eq!(r.primary, LinkType::LoraSx1276);
    }

    #[test]
    fn test_ground_wifi_video() {
        let r = route_link(
            0.05,
            DataRateClass::Video,
            Criticality::Routine,
            false,
            LinkStatus::Unavailable,
            true,
        );
        assert_eq!(r.primary, LinkType::Wifi6e);
    }

    #[test]
    fn test_ground_5g() {
        let r = route_link(
            2.0,
            DataRateClass::LowBandwidth,
            Criticality::Routine,
            false,
            LinkStatus::Connected,
            false,
        );
        assert_eq!(r.primary, LinkType::FiveGModem);
    }

    #[test]
    fn test_short_range_ble() {
        let r = route_link(
            0.05,
            DataRateClass::LowBandwidth,
            Criticality::Routine,
            false,
            LinkStatus::Unavailable,
            false,
        );
        assert_eq!(r.primary, LinkType::Ble5);
    }

    #[test]
    fn test_beyond_terrestrial_satcom() {
        let r = route_link(
            50.0,
            DataRateClass::LowBandwidth,
            Criticality::Routine,
            false,
            LinkStatus::Unavailable,
            false,
        );
        assert_eq!(r.primary, LinkType::IridiumCertus);
    }

    #[test]
    fn test_link_properties() {
        assert_eq!(LinkType::VhfRadio.domain(), CommsDomain::AirBand);
        assert!(!LinkType::VhfRadio.encryption_allowed());
        assert!(LinkType::FiveGModem.encryption_allowed());
        assert!(LinkType::Wifi6e.max_data_rate_bps() > LinkType::FiveGModem.max_data_rate_bps());
    }
}
