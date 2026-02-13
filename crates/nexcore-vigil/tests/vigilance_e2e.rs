//! # End-to-End Vigilance Subsystem Test — π(∂·ν)|∝
//!
//! Tests the complete vigilance pipeline:
//! 1. Start daemon with sources, boundaries, consequences
//! 2. Run daemon loop → events flow through all 4 layers
//! 3. Verify ledger records all activity
//! 4. Verify hash chain integrity
//! 5. Graceful shutdown via ShutdownHandle
//! 6. WAL recovery and re-verification

use nexcore_vigil::vigilance::sources::TimerSource;
use nexcore_vigil::vigilance::{
    BoundarySpec, EscalationLevel, EventKind, EventSeverity, LedgerEntryType, LedgerQuery,
    LogConsequence, NotifyConsequence, ThresholdCheck, VigilConfig, VigilDaemon, VigilanceLedger,
};
use std::path::PathBuf;
use std::time::Duration;

fn e2e_config(wal_suffix: &str) -> VigilConfig {
    VigilConfig {
        wal_path: PathBuf::from(format!("/tmp/vigil-e2e-{wal_suffix}.wal")),
        poll_interval_ms: 50,
        max_event_buffer: 1000,
        sources: Vec::new(),
        boundaries: Vec::new(),
        consequences: Vec::new(),
        sync_wal: false,
    }
}

/// Clean up WAL file before test
fn cleanup_wal(path: &std::path::Path) {
    let _ = std::fs::remove_file(path);
}

// ============================================================================
// Test 1: Full pipeline — start → events → boundary → consequence → ledger → stop
// ============================================================================

#[tokio::test]
async fn e2e_full_pipeline() {
    let config = e2e_config("full");
    let wal_path = config.wal_path.clone();
    cleanup_wal(&wal_path);

    let mut daemon = VigilDaemon::new(config);

    // ν Layer: Add a fast heartbeat source
    daemon.add_source(Box::new(TimerSource::new(
        "heartbeat",
        Duration::from_millis(10),
    )));

    // ∂ Layer: Add a catch-all boundary that fires on every event
    daemon.add_boundary(BoundarySpec {
        name: "catch-all".to_string(),
        source_filter: None,
        kind_filter: None,
        threshold: ThresholdCheck::Always,
        cooldown: Duration::from_millis(0),
    });

    // ∝ Layer: Add a log consequence
    daemon.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Observe)));

    // Get shutdown handle before run
    let handle = daemon.shutdown_handle();

    // Schedule shutdown after enough cycles
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        handle.shutdown();
    });

    // Run the daemon — blocks until shutdown
    let result = daemon.run().await;
    assert!(result.is_ok(), "Daemon run should succeed");

    let cycles = result.unwrap_or(0);
    assert!(
        cycles >= 2,
        "Should complete at least 2 cycles, got {cycles}"
    );

    // Verify daemon stopped
    assert!(!daemon.is_running(), "Daemon should be stopped");

    // Verify stats show activity
    let stats = daemon.stats();
    assert!(
        stats.total_events >= 2,
        "Should have at least 2 events, got {}",
        stats.total_events
    );
    assert!(
        stats.total_violations >= 2,
        "Catch-all boundary should produce violations, got {}",
        stats.total_violations
    );
    assert!(
        stats.total_consequences >= 2,
        "Log consequence should fire, got {}",
        stats.total_consequences
    );

    // π Layer: Verify ledger
    let ledger = daemon.ledger().lock().expect("Ledger lock should succeed");

    // Must have: DaemonStarted + EventObserved(s) + BoundaryViolation(s) +
    // ConsequenceScheduled(s) + ConsequenceExecuted(s) + DaemonStopped
    assert!(
        ledger.len() >= 6,
        "Ledger should have at least 6 entries, got {}",
        ledger.len()
    );

    // Verify hash chain integrity
    let chain_ok = ledger
        .verify_chain()
        .expect("Chain verification should succeed");
    assert!(chain_ok, "Hash chain should be intact");

    // Verify DaemonStarted is first entry
    let started_query = LedgerQuery {
        entry_type: Some(LedgerEntryType::DaemonStarted),
        since: None,
        limit: Some(1),
    };
    let started = ledger.query(&started_query);
    assert_eq!(started.len(), 1, "Should have exactly 1 DaemonStarted");
    assert_eq!(started[0].sequence, 0, "DaemonStarted should be sequence 0");

    // Verify DaemonStopped is last entry
    let stopped_query = LedgerQuery {
        entry_type: Some(LedgerEntryType::DaemonStopped),
        since: None,
        limit: Some(1),
    };
    let stopped = ledger.query(&stopped_query);
    assert_eq!(stopped.len(), 1, "Should have exactly 1 DaemonStopped");

    // Verify events were observed
    let event_query = LedgerQuery {
        entry_type: Some(LedgerEntryType::EventObserved),
        since: None,
        limit: Some(100),
    };
    let events = ledger.query(&event_query);
    assert!(
        events.len() >= 2,
        "Should observe at least 2 events, got {}",
        events.len()
    );

    // Verify consequences were scheduled and executed
    let scheduled_query = LedgerQuery {
        entry_type: Some(LedgerEntryType::ConsequenceScheduled),
        since: None,
        limit: Some(100),
    };
    let scheduled = ledger.query(&scheduled_query);
    assert!(
        !scheduled.is_empty(),
        "Should have ConsequenceScheduled entries"
    );

    let executed_query = LedgerQuery {
        entry_type: Some(LedgerEntryType::ConsequenceExecuted),
        since: None,
        limit: Some(100),
    };
    let executed = ledger.query(&executed_query);
    assert!(
        !executed.is_empty(),
        "Should have ConsequenceExecuted entries"
    );

    // Cleanup
    drop(ledger);
    cleanup_wal(&wal_path);
}

// ============================================================================
// Test 2: WAL recovery — write ledger → stop → recover → verify chain
// ============================================================================

#[tokio::test]
async fn e2e_wal_recovery() {
    let config = e2e_config("recovery");
    let wal_path = config.wal_path.clone();
    cleanup_wal(&wal_path);

    // Phase 1: Run daemon briefly
    {
        let mut daemon = VigilDaemon::new(config.clone());
        daemon.add_source(Box::new(TimerSource::new(
            "heartbeat",
            Duration::from_millis(10),
        )));
        daemon.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Observe)));

        let handle = daemon.shutdown_handle();
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(200)).await;
            handle.shutdown();
        });

        let result = daemon.run().await;
        assert!(result.is_ok());

        let original_len = daemon.ledger().lock().expect("lock").len();
        assert!(original_len >= 3, "Should have entries before recovery");
    }

    // Phase 2: Recover from WAL and verify
    assert!(wal_path.exists(), "WAL file should exist after daemon stop");

    let recovered = VigilanceLedger::recover_from_wal(&wal_path);
    assert!(recovered.is_ok(), "WAL recovery should succeed");

    let recovered = recovered.expect("recovery");
    assert!(recovered.len() >= 3, "Recovered ledger should have entries");

    let chain_ok = recovered.verify_chain().expect("verify");
    assert!(chain_ok, "Recovered chain should be intact");

    cleanup_wal(&wal_path);
}

// ============================================================================
// Test 3: Severity boundary — only High+ events trigger consequences
// ============================================================================

#[tokio::test]
async fn e2e_severity_filtering() {
    let config = e2e_config("severity");
    let wal_path = config.wal_path.clone();
    cleanup_wal(&wal_path);

    let mut daemon = VigilDaemon::new(config);

    // Timer events are severity Medium by default
    daemon.add_source(Box::new(TimerSource::new(
        "heartbeat",
        Duration::from_millis(10),
    )));

    // Boundary requires High severity — should NOT trigger on Medium timer events
    daemon.add_boundary(BoundarySpec {
        name: "high-only".to_string(),
        source_filter: None,
        kind_filter: None,
        threshold: ThresholdCheck::SeverityAtLeast(EventSeverity::High),
        cooldown: Duration::from_millis(0),
    });

    daemon.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Alert)));

    let handle = daemon.shutdown_handle();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        handle.shutdown();
    });

    let result = daemon.run().await;
    assert!(result.is_ok());

    let stats = daemon.stats();
    // Timer events are Info severity → should NOT cross High threshold
    assert_eq!(
        stats.total_violations, 0,
        "No violations expected for Info events against High threshold"
    );
    assert_eq!(
        stats.total_consequences, 0,
        "No consequences expected when no violations"
    );

    cleanup_wal(&wal_path);
}

// ============================================================================
// Test 4: Multiple boundaries — independent evaluation
// ============================================================================

#[tokio::test]
async fn e2e_multiple_boundaries() {
    let config = e2e_config("multi");
    let wal_path = config.wal_path.clone();
    cleanup_wal(&wal_path);

    let mut daemon = VigilDaemon::new(config);

    daemon.add_source(Box::new(TimerSource::new(
        "heartbeat",
        Duration::from_millis(10),
    )));

    // Boundary 1: catch-all (will fire)
    daemon.add_boundary(BoundarySpec {
        name: "catch-all".to_string(),
        source_filter: None,
        kind_filter: None,
        threshold: ThresholdCheck::Always,
        cooldown: Duration::from_millis(0),
    });

    // Boundary 2: only file changes (won't fire — timer events aren't file changes)
    daemon.add_boundary(BoundarySpec {
        name: "files-only".to_string(),
        source_filter: None,
        kind_filter: Some(EventKind::FileChange),
        threshold: ThresholdCheck::Always,
        cooldown: Duration::from_millis(0),
    });

    daemon.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Observe)));

    let handle = daemon.shutdown_handle();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        handle.shutdown();
    });

    let result = daemon.run().await;
    assert!(result.is_ok());

    let stats = daemon.stats();
    // Only catch-all should fire (timer events, not file changes)
    assert!(
        stats.total_violations >= 1,
        "Catch-all should produce violations"
    );

    // Verify both boundaries are registered
    let names = daemon.boundary_names();
    assert_eq!(names.len(), 2);
    assert!(names.contains(&"catch-all"));
    assert!(names.contains(&"files-only"));

    cleanup_wal(&wal_path);
}

// ============================================================================
// Test 5: Notify consequence writes file — verifiable side effect
// ============================================================================

#[tokio::test]
async fn e2e_notify_consequence_writes_file() {
    let config = e2e_config("notify");
    let wal_path = config.wal_path.clone();
    cleanup_wal(&wal_path);

    let notify_dir = PathBuf::from("/tmp/vigil-e2e-notify");
    let _ = std::fs::create_dir_all(&notify_dir);
    // Clean prior notification files
    if let Ok(entries) = std::fs::read_dir(&notify_dir) {
        for entry in entries.flatten() {
            let _ = std::fs::remove_file(entry.path());
        }
    }

    let mut daemon = VigilDaemon::new(config);

    daemon.add_source(Box::new(TimerSource::new(
        "heartbeat",
        Duration::from_millis(10),
    )));

    daemon.add_boundary(BoundarySpec {
        name: "catch-all".to_string(),
        source_filter: None,
        kind_filter: None,
        threshold: ThresholdCheck::Always,
        cooldown: Duration::from_millis(0),
    });

    // Use NotifyConsequence — writes JSON files to disk
    daemon.add_consequence(Box::new(NotifyConsequence::new(notify_dir.clone())));

    let handle = daemon.shutdown_handle();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(300)).await;
        handle.shutdown();
    });

    let result = daemon.run().await;
    assert!(result.is_ok());

    // Verify notification files were written
    let files: Vec<_> = std::fs::read_dir(&notify_dir)
        .expect("notify dir should exist")
        .flatten()
        .filter(|e| {
            e.path()
                .extension()
                .map(|ext| ext == "json")
                .unwrap_or(false)
        })
        .collect();

    assert!(
        !files.is_empty(),
        "NotifyConsequence should write at least 1 JSON file"
    );

    // Verify file content is valid JSON
    let first_file = &files[0].path();
    let content = std::fs::read_to_string(first_file).expect("read notification");
    let parsed: Result<serde_json::Value, _> = serde_json::from_str(&content);
    assert!(parsed.is_ok(), "Notification file should be valid JSON");

    let json = parsed.expect("parse");
    assert!(json.get("title").is_some(), "Should have title field");
    assert!(json.get("body").is_some(), "Should have body field");
    assert!(json.get("urgency").is_some(), "Should have urgency field");
    assert!(
        json.get("timestamp").is_some(),
        "Should have timestamp field"
    );

    // Cleanup
    let _ = std::fs::remove_dir_all(&notify_dir);
    cleanup_wal(&wal_path);
}

// ============================================================================
// Test 6: FRIDAY bridge mapping — event type classification
// ============================================================================

#[test]
fn e2e_friday_bridge_mapping() {
    use nexcore_vigil::models::{Event as FridayEvent, Urgency};
    use nexcore_vigil::vigilance::sources::{friday_to_watch_event, map_event_type, map_priority};

    // Priority mapping
    assert_eq!(map_priority(Urgency::Critical), EventSeverity::Critical);
    assert_eq!(map_priority(Urgency::High), EventSeverity::High);
    assert_eq!(map_priority(Urgency::Normal), EventSeverity::Medium);
    assert_eq!(map_priority(Urgency::Low), EventSeverity::Low);

    // Event type mapping
    assert_eq!(map_event_type("heartbeat_tick"), EventKind::Timer);
    assert_eq!(map_event_type("file_changed"), EventKind::FileChange);
    assert_eq!(map_event_type("signal_detected"), EventKind::Signal);
    assert_eq!(map_event_type("channel_message"), EventKind::Channel);
    assert_eq!(
        map_event_type("custom_thing"),
        EventKind::Custom("custom_thing".to_string())
    );

    // Full conversion
    let friday = FridayEvent {
        source: "webhook".to_string(),
        event_type: "signal_alert".to_string(),
        priority: Urgency::Critical,
        payload: serde_json::json!({"action": "deploy"}),
        ..Default::default()
    };

    let watch = friday_to_watch_event(&friday, 99);
    assert_eq!(watch.source, "webhook");
    assert_eq!(watch.kind, EventKind::Signal);
    assert_eq!(watch.severity, EventSeverity::Critical);
    assert_eq!(watch.payload["action"], "deploy");
}

// ============================================================================
// Test 7: Health and stats consistency
// ============================================================================

#[tokio::test]
async fn e2e_health_stats_consistency() {
    let config = e2e_config("health");
    let wal_path = config.wal_path.clone();
    cleanup_wal(&wal_path);

    let mut daemon = VigilDaemon::new(config);

    daemon.add_source(Box::new(TimerSource::new(
        "alpha",
        Duration::from_millis(10),
    )));
    daemon.add_source(Box::new(TimerSource::new(
        "beta",
        Duration::from_millis(10),
    )));

    daemon.add_boundary(BoundarySpec {
        name: "gate-one".to_string(),
        source_filter: None,
        kind_filter: None,
        threshold: ThresholdCheck::Always,
        cooldown: Duration::from_millis(0),
    });

    daemon.add_consequence(Box::new(LogConsequence::new(EscalationLevel::Observe)));

    // Before start
    let pre_health = daemon.health();
    assert!(!pre_health.running);
    assert_eq!(pre_health.sources, 2);
    assert_eq!(pre_health.boundaries, 1);
    assert_eq!(pre_health.consequences, 1);

    let handle = daemon.shutdown_handle();
    tokio::spawn(async move {
        tokio::time::sleep(Duration::from_millis(200)).await;
        handle.shutdown();
    });

    let result = daemon.run().await;
    assert!(result.is_ok());

    // After stop
    let post_health = daemon.health();
    assert!(!post_health.running, "Should be stopped after run()");
    assert!(
        post_health.ledger_entries >= 3,
        "Ledger should have entries: {}",
        post_health.ledger_entries
    );

    let stats = daemon.stats();
    assert!(
        stats.total_events >= 2,
        "2 sources should produce events: {}",
        stats.total_events
    );
    assert!(
        !stats.ledger_head_hash.is_empty(),
        "Head hash should be non-empty"
    );
    assert_ne!(
        stats.ledger_head_hash, "error",
        "Head hash should not be error"
    );

    cleanup_wal(&wal_path);
}
