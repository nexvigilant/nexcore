//! Closes the third leg of the PerceptionSource ρ contract — Serial via pty
//! loopback. Mirrors crates/stark-suit-station/tests/serial_pty_roundtrip.rs
//! exactly, swapping BMS for Perception types — the trait template is one
//! contract, two compounds.

#![cfg(unix)]

use stark_suit_station::perception::{
    PerceptionFrame, PerceptionSource, SerialPerceptionSource, PERCEPTION_FRAME_VERSION,
};
use stark_suit_test_pty::TestPty;

fn frame(hr: u8, ts_ms: u64) -> PerceptionFrame {
    PerceptionFrame {
        version: PERCEPTION_FRAME_VERSION,
        ts_ms,
        accel_mps2: [0.0, 0.0, 9.81],
        gyro_radps: [0.0, 0.0, 0.0],
        mag_ut: [22.0, 0.0, -42.0],
        pressure_hpa: 1013.25,
        temp_c: 20.0,
        heart_rate_bpm: hr,
        spo2_pct: 98,
        ts: None,
    }
}

#[tokio::test]
async fn perception_pty_round_trip_preserves_frame_sequence() {
    let mut pty = TestPty::spawn().expect("openpty");
    let slave_path = pty
        .slave_path
        .to_str()
        .expect("slave path utf8")
        .to_string();
    let serial = SerialPerceptionSource::open(&slave_path, 115200).expect("open serial");

    let trace = vec![
        frame(60, 1_000),
        frame(61, 1_050),
        frame(62, 1_100),
        frame(63, 1_150),
        frame(64, 1_200),
    ];

    let trace_clone = trace.clone();
    let writer_handle = tokio::spawn(async move {
        for f in &trace_clone {
            let line = serde_json::to_string(f).expect("serialize");
            pty.write_line(&line).await.expect("write_line");
            tokio::time::sleep(std::time::Duration::from_millis(5)).await;
        }
    });

    let mut observed = Vec::new();
    for _ in 0..trace.len() {
        let polled = tokio::time::timeout(std::time::Duration::from_secs(2), serial.poll())
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
        assert_eq!(got.heart_rate_bpm, want.heart_rate_bpm);
        assert_eq!(got.accel_mps2, want.accel_mps2);
    }
}
