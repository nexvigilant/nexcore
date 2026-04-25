# BMS Frame Wire Format — v1

**Status:** stable. Adopted 2026-04-25 with `stark-suit-station` v0.3.

The BMS telemetry stream is **NDJSON** (newline-delimited JSON, UTF-8): one JSON object per line, no array wrapper, `\n` between frames. NDJSON was chosen over length-prefixed framing because it is record/replay/serial compatible with zero adaptation and debuggable with `cat`.

Every frame carries an explicit `version: u32` field. A reader that encounters a `version` it does not recognize **must** raise `BmsError::UnsupportedVersion` rather than silently parsing — without this, schema evolution corrupts every old recording into a quiet test failure.

## Fields (v1, 8 total)

| Field | Type | Required | Notes |
|---|---|:-:|---|
| `version` | u32 | yes | Always `1` for this schema. |
| `ts_ms` | u64 | yes | Epoch milliseconds (UTC). `Instant` is **not** serializable — replay determinism requires a wall-clock anchor. |
| `pack_voltage_v` | f32 | yes | Pack voltage, volts. |
| `pack_current_a` | f32 | yes | Pack current, amps. Positive = draw, negative = charge. |
| `cell_temp_c` | f32 | yes | Hottest cell temperature, °C. |
| `soc_pct` | f32 | yes | State of charge, 0..=100. |
| `soh_pct` | f32 | yes | State of health, 0..=100. |
| `tier` | string enum | yes | One of `"comms"`, `"compute"`, `"actuation"`, `"critical"` (snake_case). Source-of-truth load classification from BMS. |

## Example line

```
{"version":1,"ts_ms":1777130962205,"pack_voltage_v":400.0,"pack_current_a":10.0,"cell_temp_c":25.0,"soc_pct":100.0,"soh_pct":100.0,"tier":"comms"}
```

## Lifecycle

- **Record**: `stark-suit-station record --out trace.ndjson --seconds N` writes one frame per 100 ms tick.
- **Replay**: `stark-suit-station run --bms-source replay --trace trace.ndjson [--speedup N]` reconstructs inter-frame delay from `ts_ms` deltas (clamped non-negative), divided by `speedup`.
- **Round-trip contract**: `record → replay` is frame-for-frame identical for the persisted fields (`Instant` is a wall-clock decoration and is NOT preserved).

## Versioning rule

Bump `FRAME_VERSION` **only** when an existing field changes type, semantics, or required-ness. Adding a new optional field with a `#[serde(default)]` does not require a bump — old readers ignore unknown fields and new readers fall back. Removing a field requires a bump and a parser branch for the older version.

Source: `crates/stark-suit-station/src/bms.rs`, `FRAME_VERSION` constant.

## Hardware fidelity

The `SerialBmsSource` backend is validated against `stark-suit-test-pty`, which spawns a kernel pseudoterminal pair via `nix::pty::openpty`. The slave path (`/dev/pts/N`) is opened by `tokio-serial::open_native_async` exactly as `/dev/ttyUSB0` would be — the kernel does not distinguish.

What the pty round-trip proves:
- **Protocol fidelity** — NDJSON framing, line-delimited reads, version handshake, schema parse compatibility across the byte stream boundary.
- **Trait abstraction** — three backends (Mock, Replay, Serial) pass the same frame-for-frame equality contract test on the persisted fields.

What the pty round-trip does NOT prove:
- **Physical-layer fidelity** — baud-rate sync, parity drift, USB hot-unplug, electrical noise, CAN-vs-UART framing. These require silicon.
- **Driver-level edge cases** — `tcdrain`, kernel buffer overruns at sustained 115200 baud, FTDI vs CH340 chip-specific quirks.

When a USB-serial loopback adapter (~$8) or a microcontroller emitting test frames is on the bench, re-run `serial_pty_roundtrip.rs` against the real device path (`--port /dev/ttyUSB0`). The test code itself does not change — only the path argument. Until then, the Serial backend is **loopback-validated; hardware fidelity pending**.

## Backend matrix

| Backend | Wire path | Validation today | Pending |
|---|---|---|---|
| Mock | in-memory baked trace | unit (3 tests) | — |
| Replay | NDJSON file | unit + round-trip equality | — |
| Serial | `/dev/pts/*` (pty) or `/dev/tty*` (hardware) | pty round-trip | physical layer |
| CAN | TBD | deferred to v0.5+ | codec, transport |

