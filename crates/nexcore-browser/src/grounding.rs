//! # GroundsTo implementations for nexcore-browser types
//!
//! Connects browser automation types to the Lex Primitiva type system.
//!
//! ## Sequence (sigma) Focus
//!
//! Browser automation is fundamentally a rendering pipeline:
//! navigate -> load -> render -> collect -> broadcast.
//! The dominant primitive is sigma (Sequence) for the pipeline,
//! with State (varsigma) for browser state management.

use nexcore_lex_primitiva::grounding::GroundsTo;
use nexcore_lex_primitiva::primitiva::{LexPrimitiva, PrimitiveComposition};

use crate::{
    BrowserError, BrowserEvent, BrowserSettings, ConsoleCollector, ConsoleEntry, ConsoleLevel,
    McpBridge, McpBridgeConfig, McpBridgeError, McpBridgeStats, McpConsoleMessage,
    McpNetworkRequest, NetworkCollector, NetworkEntry, NetworkStatus, PageInfo,
};

// ---------------------------------------------------------------------------
// Browser state types -- varsigma (State) dominant
// ---------------------------------------------------------------------------

/// BrowserSettings: T2-P (varsigma + N), dominant varsigma
///
/// Configuration state for browser instance (headless, width, height, etc).
/// State-dominant: the struct IS configuration state for the browser.
/// Quantity is secondary (width, height are numeric dimensions).
impl GroundsTo for BrowserSettings {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- browser configuration state
            LexPrimitiva::Quantity, // N -- numeric dimensions (width, height)
        ])
        .with_dominant(LexPrimitiva::State, 0.85)
    }
}

/// PageInfo: T2-C (varsigma + lambda + sigma + exists), dominant varsigma
///
/// Represents a browser page with ID, URL, title, and status.
/// State-dominant: it's a snapshot of page state.
/// Location is secondary (URL is a location).
/// Sequence is tertiary (page lifecycle progression).
/// Existence is quaternary (page may or may not exist).
impl GroundsTo for PageInfo {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,     // varsigma -- page state snapshot
            LexPrimitiva::Location,  // lambda -- URL location
            LexPrimitiva::Sequence,  // sigma -- page lifecycle progression
            LexPrimitiva::Existence, // exists -- page existence check
        ])
        .with_dominant(LexPrimitiva::State, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Event types -- sigma (Sequence) dominant
// ---------------------------------------------------------------------------

/// BrowserEvent: T3 (sigma + Sigma + varsigma + lambda + N + nu), dominant sigma
///
/// Multi-variant enum of browser events (console, network, page lifecycle).
/// Sequence-dominant: events form an ordered stream (event pipeline).
/// Sum is secondary (enum variants: ConsoleMessage | NetworkFailure | ...).
/// State is tertiary (each event captures a moment of browser state).
impl GroundsTo for BrowserEvent {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence,  // sigma -- ordered event stream
            LexPrimitiva::Sum,       // Sigma -- variant alternation
            LexPrimitiva::State,     // varsigma -- state snapshot per event
            LexPrimitiva::Location,  // lambda -- URL/source locations
            LexPrimitiva::Quantity,  // N -- status codes, sizes, durations
            LexPrimitiva::Frequency, // nu -- event frequency/timing
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

// ---------------------------------------------------------------------------
// Collector types -- sigma (Sequence) dominant
// ---------------------------------------------------------------------------

/// ConsoleCollector: T2-C (sigma + varsigma + partial + N), dominant sigma
///
/// FIFO-bounded collection of console messages.
/// Sequence-dominant: it collects an ordered sequence of messages.
/// State is secondary (mutable collection state).
/// Boundary is tertiary (FIFO capacity bound).
/// Quantity is quaternary (message count).
impl GroundsTo for ConsoleCollector {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered message collection
            LexPrimitiva::State,    // varsigma -- mutable collection state
            LexPrimitiva::Boundary, // partial -- FIFO capacity bound
            LexPrimitiva::Quantity, // N -- message count
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// ConsoleEntry: T2-P (sigma + Sigma + N), dominant sigma
///
/// A single console log entry with level, text, and timestamp.
/// Sequence-dominant: part of an ordered log stream.
impl GroundsTo for ConsoleEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered in log stream
            LexPrimitiva::Sum,      // Sigma -- level variant (log/warn/error)
            LexPrimitiva::Quantity, // N -- line/column numbers
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// ConsoleLevel: T1 (Sigma), pure sum
///
/// Enum classifying console message severity (Log, Warn, Error, etc).
impl GroundsTo for ConsoleLevel {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Sigma -- categorical alternation
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

/// NetworkCollector: T2-C (sigma + varsigma + partial + N), dominant sigma
///
/// FIFO-bounded collection of network requests.
impl GroundsTo for NetworkCollector {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered request collection
            LexPrimitiva::State,    // varsigma -- mutable collection state
            LexPrimitiva::Boundary, // partial -- FIFO capacity bound
            LexPrimitiva::Quantity, // N -- request count
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.85)
    }
}

/// NetworkEntry: T2-P (sigma + lambda + N), dominant sigma
///
/// A single network request/response entry.
impl GroundsTo for NetworkEntry {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sequence, // sigma -- ordered in request stream
            LexPrimitiva::Location, // lambda -- URL location
            LexPrimitiva::Quantity, // N -- status code, size, duration
        ])
        .with_dominant(LexPrimitiva::Sequence, 0.80)
    }
}

/// NetworkStatus: T1 (Sigma), pure sum
///
/// Enum classifying network request status (Success, Failure, Pending).
impl GroundsTo for NetworkStatus {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Sum, // Sigma -- categorical alternation
        ])
        .with_dominant(LexPrimitiva::Sum, 1.0)
    }
}

// ---------------------------------------------------------------------------
// MCP Bridge types -- mu (Mapping) dominant
// ---------------------------------------------------------------------------

/// McpBridge: T2-C (mu + sigma + varsigma + partial), dominant mu
///
/// Bridges browser events to MCP tool protocol.
/// Mapping-dominant: transforms browser events into MCP-consumable format.
impl GroundsTo for McpBridge {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- browser events -> MCP format
            LexPrimitiva::Sequence, // sigma -- event stream processing
            LexPrimitiva::State,    // varsigma -- bridge state
            LexPrimitiva::Boundary, // partial -- protocol boundaries
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.80)
    }
}

/// McpBridgeConfig: T2-P (varsigma + partial), dominant varsigma
///
/// Configuration for the MCP bridge.
impl GroundsTo for McpBridgeConfig {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::State,    // varsigma -- configuration state
            LexPrimitiva::Boundary, // partial -- capacity limits
        ])
        .with_dominant(LexPrimitiva::State, 0.90)
    }
}

/// McpBridgeError: T2-P (partial + Sigma), dominant partial
///
/// Error type for MCP bridge operations.
impl GroundsTo for McpBridgeError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- error boundary
            LexPrimitiva::Sum,      // Sigma -- error variant alternation
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

/// McpBridgeStats: T2-P (N + varsigma), dominant N
///
/// Statistics for the MCP bridge (message counts, error counts).
impl GroundsTo for McpBridgeStats {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Quantity, // N -- numeric counters
            LexPrimitiva::State,    // varsigma -- stats snapshot
        ])
        .with_dominant(LexPrimitiva::Quantity, 0.90)
    }
}

/// McpConsoleMessage: T2-P (mu + sigma), dominant mu
///
/// Console message in MCP-compatible format.
impl GroundsTo for McpConsoleMessage {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- transformed from ConsoleEntry
            LexPrimitiva::Sequence, // sigma -- part of ordered stream
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

/// McpNetworkRequest: T2-P (mu + lambda), dominant mu
///
/// Network request in MCP-compatible format.
impl GroundsTo for McpNetworkRequest {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Mapping,  // mu -- transformed from NetworkEntry
            LexPrimitiva::Location, // lambda -- URL location
        ])
        .with_dominant(LexPrimitiva::Mapping, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Error types -- partial (Boundary) dominant
// ---------------------------------------------------------------------------

/// BrowserError: T2-P (partial + Sigma), dominant partial
///
/// Error type for browser operations.
impl GroundsTo for BrowserError {
    fn primitive_composition() -> PrimitiveComposition {
        PrimitiveComposition::new(vec![
            LexPrimitiva::Boundary, // partial -- error boundary
            LexPrimitiva::Sum,      // Sigma -- error variant alternation
        ])
        .with_dominant(LexPrimitiva::Boundary, 0.85)
    }
}

// ---------------------------------------------------------------------------
// Tests
// ---------------------------------------------------------------------------

#[cfg(test)]
mod tests {
    use super::*;
    use nexcore_lex_primitiva::tier::Tier;

    #[test]
    fn browser_settings_is_t2p() {
        assert_eq!(BrowserSettings::tier(), Tier::T2Primitive);
        assert_eq!(
            BrowserSettings::dominant_primitive(),
            Some(LexPrimitiva::State)
        );
    }

    #[test]
    fn page_info_is_t2c() {
        assert_eq!(PageInfo::tier(), Tier::T2Composite);
        assert_eq!(PageInfo::dominant_primitive(), Some(LexPrimitiva::State));
    }

    #[test]
    fn browser_event_is_t3() {
        assert_eq!(BrowserEvent::tier(), Tier::T3DomainSpecific);
        assert_eq!(
            BrowserEvent::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn console_level_is_t1() {
        assert_eq!(ConsoleLevel::tier(), Tier::T1Universal);
        assert!(ConsoleLevel::is_pure_primitive());
    }

    #[test]
    fn network_status_is_t1() {
        assert_eq!(NetworkStatus::tier(), Tier::T1Universal);
        assert!(NetworkStatus::is_pure_primitive());
    }

    #[test]
    fn console_collector_is_t2c() {
        assert_eq!(ConsoleCollector::tier(), Tier::T2Composite);
        assert_eq!(
            ConsoleCollector::dominant_primitive(),
            Some(LexPrimitiva::Sequence)
        );
    }

    #[test]
    fn mcp_bridge_is_t2c() {
        assert_eq!(McpBridge::tier(), Tier::T2Composite);
        assert_eq!(McpBridge::dominant_primitive(), Some(LexPrimitiva::Mapping));
    }

    #[test]
    fn browser_error_is_t2p() {
        assert_eq!(BrowserError::tier(), Tier::T2Primitive);
        assert_eq!(
            BrowserError::dominant_primitive(),
            Some(LexPrimitiva::Boundary)
        );
    }

    #[test]
    fn all_confidences_valid() {
        let compositions = [
            BrowserSettings::primitive_composition(),
            PageInfo::primitive_composition(),
            BrowserEvent::primitive_composition(),
            ConsoleCollector::primitive_composition(),
            ConsoleEntry::primitive_composition(),
            ConsoleLevel::primitive_composition(),
            NetworkCollector::primitive_composition(),
            NetworkEntry::primitive_composition(),
            NetworkStatus::primitive_composition(),
            McpBridge::primitive_composition(),
            McpBridgeConfig::primitive_composition(),
            McpBridgeError::primitive_composition(),
            McpBridgeStats::primitive_composition(),
            McpConsoleMessage::primitive_composition(),
            McpNetworkRequest::primitive_composition(),
            BrowserError::primitive_composition(),
        ];
        for comp in &compositions {
            assert!(comp.confidence >= 0.80);
            assert!(comp.confidence <= 1.0);
        }
    }
}
