#!/usr/bin/env bash
set -euo pipefail

ROOT="/home/matthew/Projects/Active/nexcore"
EVENT_LOG="${NEXCORE_STATION_EVENT_LOG:-/tmp/nexcore-station-events.ndjson}"
INTERVAL_SEC="${STATION_LIVE_INTERVAL_SEC:-5}"

export NEXCORE_STATION_EVENT_LOG="$EVENT_LOG"
mkdir -p "$(dirname "$EVENT_LOG")"
touch "$EVENT_LOG"

echo "[station-live] event log: $EVENT_LOG"
echo "[station-live] interval: ${INTERVAL_SEC}s"
echo "[station-live] press Ctrl+C to stop"

cd "$ROOT"

while true; do
  cargo test -p nexcore-station --features live-feed,integration \
    client::tests::live_resolve_dailymed_integration -- --nocapture >/tmp/station-live-loop.log 2>&1 || true
  sleep "$INTERVAL_SEC"
done
