//! Closes the third leg of the BmsSource ρ contract — Serial via pty loopback.
//!
//! Mock: in-memory. Replay: file-based. Serial: kernel character device.
//! Each backend reads the same NDJSON wire format. If round-trip equality
//! holds across a real `tokio-serial` open against a `/dev/pts/N` slave, the
//! trait is genuinely backend-agnostic to the protocol layer (the wire
//! format is the contract; the byte-stream source is interchangeable).

#![cfg(unix)]

use stark_suit_test_pty::TestPty;

// v0.5: stark-suit-station is now lib + bin, so the BMS surface imports
// through the public API instead of the v0.4 `#[path]` hack.

use stark_suit_station::bms::{BmsFrame, BmsSource, FRAME_VERSION, PowerTier, SerialBmsSource};

fn frame(soc: f32, ts_ms: u64, tier: PowerTier) -> BmsFrame {
    BmsFrame {
        version: FRAME_VERSION,
        ts_ms,
        pack_voltage_v: 400.0 - (100.0 - soc) * 0.5,
        pack_current_a: 10.0,
        cell_temp_c: 25.0 + (100.0 - soc) * 0.05,
        soc_pct: soc,
        soh_pct: 100.0,
        tier,
        ts: None,
    }
}

#[tokio::test]
async fn serial_pty_round_trip_preserves_frame_sequence() {
    let mut pty = TestPty::spawn().expect("openpty");
    let slave_path = pty
        .slave_path
        .to_str()
        .expect("slave path utf8")
        .to_string();
    let serial = SerialBmsSource::open(&slave_path, 115200).expect("open serial");

    let trace = vec![
        frame(100.0, 1_000, PowerTier::Comms),
        frame(99.0, 1_100, PowerTier::Comms),
        frame(98.0, 1_200, PowerTier::Compute),
        frame(97.0, 1_300, PowerTier::Compute),
        frame(96.0, 1_400, PowerTier::Actuation),
    ];

    // Spawn a writer task that pushes the trace through the pty master.
    let trace_clone = trace.clone();
    let writer_handle = tokio::spawn(async move {
        for f in &trace_clone {
            let line = serde_json::to_string(f).expect("serialize");
            pty.write_line(&line).await.expect("write_line");
            // Tiny gap so the reader's read_line doesn't fuse two frames.
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    });

    // Read N frames back through SerialBmsSource. Cap with a per-poll
    // timeout so a stuck read fails the test loudly.
    let mut observed = Vec::new();
    for _ in 0..trace.len() {
        let polled = tokio::time::timeout(
            std::time::Duration::from_secs(2),
            serial.poll(),
        )
        .await
        .expect("poll timed out — pty plumbing wedged")
        .expect("poll returned error");
        observed.push(polled);
    }
    writer_handle.await.expect("writer task");

    assert_eq!(observed.len(), trace.len());
    for (got, want) in observed.iter().zip(trace.iter()) {
        assert_eq!(got.version, want.version);
        assert_eq!(got.ts_ms, want.ts_ms);
        assert_eq!(got.soc_pct, want.soc_pct);
        assert_eq!(got.tier, want.tier);
    }
}
