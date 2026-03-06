# Station Loop Closure — 2026-03-06

## Scope Closed

1. Codex MCP auth wiring for `moltbrowser-mcp`
2. Station telemetry wiring audit + legacy path check
3. Persistent live stream runner for telemetry
4. Verification pass (tests + live stream burn-in)

## Results

- Codex MCP server shows env binding for `HUB_API_KEY` (masked by CLI output).
- Active path telemetry is wired in `client.rs` + `feed.rs` + `telemetry.rs`.
- Legacy `resolution.rs` now emits resolve telemetry start/finish.
- Both notebooks are wired to `NEXCORE_STATION_EVENT_LOG` stream.
- Live runner script added and validated.

## Commands

- Wiring audit:
  - `scripts/station-wiring-audit.sh`
- Live loop runner:
  - `NEXCORE_STATION_EVENT_LOG=/tmp/nexcore-station-events.ndjson scripts/station-live-stream.sh`
- Tests:
  - `cargo test -p nexcore-station`
  - `cargo test -p nexcore-station --features live-feed`

## Deferred

- `observatory.rs` remains a legacy module without telemetry hooks.
  - Current impact: none on active resolution/feed path.
  - Recommendation: either deprecate/remove legacy module or wire full event surface if it becomes active.
