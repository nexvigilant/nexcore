#!/usr/bin/env bash
# NVNC Station spawn helper.
#
# Sibling to nvnc-spawn-nucleus.sh. Reads nvnc-station.unit.toml and
# spawns the nexvigilant-station binary as a systemd --user unit.
#
# Bundle layout (matthew-owned, no /opt escalation):
#   ~/nvnc/station/bin/nexvigilant-station
#   ~/nvnc/station/configs/
#
# Override the default bundle by exporting NVNC_STATION_BUNDLE=/path.
set -euo pipefail

BUNDLE="${NVNC_STATION_BUNDLE:-$HOME/nvnc/station}"
PORT="${NVNC_STATION_PORT:-18082}"
HOST="${NVNC_STATION_HOST:-127.0.0.1}"
UNIT="nvnc-station.service"

if [[ ! -x "$BUNDLE/bin/nexvigilant-station" ]]; then
  echo "✗ binary missing: $BUNDLE/bin/nexvigilant-station" >&2
  exit 1
fi
if [[ ! -d "$BUNDLE/configs" ]]; then
  echo "✗ configs missing: $BUNDLE/configs" >&2
  exit 1
fi

# Stop any prior instance of the unit.
systemctl --user stop "$UNIT" 2>/dev/null || true
systemctl --user reset-failed "$UNIT" 2>/dev/null || true

echo "[spawn] unit:    $UNIT"
echo "[spawn] bundle:  $BUNDLE"
echo "[spawn] port:    $PORT"
echo "[spawn] host:    $HOST"
echo

systemd-run --user \
  --unit="$UNIT" \
  --setenv=RUST_LOG=info \
  --setenv=NODE_ENV=production \
  "$BUNDLE/bin/nexvigilant-station" \
  --config-dir "$BUNDLE/configs" \
  --transport combined \
  --host "$HOST" \
  --port "$PORT" \
  --exclude-private

echo
echo "[spawn] ✓ spawned $UNIT on $HOST:$PORT"
echo "[spawn] health:  curl -s http://$HOST:$PORT/health | jq ."
echo "[spawn] logs:    journalctl --user -u $UNIT --follow"
echo "[spawn] stop:    systemctl --user stop $UNIT"
