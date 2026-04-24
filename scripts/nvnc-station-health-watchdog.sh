#!/usr/bin/env bash
# NVNC Station /health watchdog.
# Runs on a systemd-user timer. Curls /health; after 3 consecutive
# failures, stops the Station unit so journald + systemctl surface
# the bad state. A human or higher-level supervisor restarts it.
#
# State: $XDG_RUNTIME_DIR/nvnc-station-health.fail (failure counter).
set -euo pipefail

URL="${NVNC_STATION_HEALTH_URL:-http://127.0.0.1:8084/health}"
UNIT="${NVNC_STATION_UNIT:-nvnc-station.service}"
MAX_FAILS="${NVNC_STATION_MAX_FAILS:-3}"
STATE_DIR="${XDG_RUNTIME_DIR:-/tmp}"
FAIL_FILE="$STATE_DIR/nvnc-station-health.fail"

code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 5 "$URL" || echo "000")

if [[ "$code" == "200" ]]; then
  rm -f "$FAIL_FILE"
  echo "nvnc-station-health: ok code=$code"
  exit 0
fi

n=$(cat "$FAIL_FILE" 2>/dev/null || echo 0)
n=$((n + 1))
echo "$n" > "$FAIL_FILE"

echo "nvnc-station-health: fail code=$code count=$n/$MAX_FAILS" >&2

if (( n >= MAX_FAILS )); then
  echo "nvnc-station-health: reached $MAX_FAILS consecutive failures — stopping $UNIT" >&2
  systemctl --user stop "$UNIT" || true
  ts=$(date -Is)
  host=$(hostname)
  last_log=$(journalctl --user -u "$UNIT" --no-pager -n 20 2>/dev/null | tail -c 2000 || echo "(journalctl unavailable)")

  # Wake Vigil — the agent owns response orchestration (email, cytokine
  # emit, restart, page Matthew). Watchdog is a dumb sensor: it reports
  # state change, Vigil decides the action. Design-per-mandate: Vigil
  # is the agent that uses all the tools.
  VIGIL_URL="${NVNC_VIGIL_URL:-http://127.0.0.1:7823/wake}"
  prompt=$(cat <<EOT
NVNC Station watchdog stopped $UNIT on $host at $ts after $MAX_FAILS consecutive /health failures ($URL).

Investigate:
1. journalctl --user -u $UNIT --no-pager -n 50 (last tail below)
2. Check upstream: LB 136.110.227.199, firewall allow-nvnc-station, port 8084
3. Attempt one restart: bash /home/matthew/Projects/Active/nucleus/workspaces/nexcore/scripts/nvnc-spawn-station.sh
4. If restart fails, emit il1 critical cytokine nvnc_station_down and email matthew@camp-corp.com with root cause.

Last 2KB of journal:
$last_log
EOT
)

  # POST to /wake — Vigil runs a full Claude turn synchronously. The
  # watchdog fire-and-forgets (runs in background); Vigil owns all
  # downstream response (email, cytokine, restart, page). Design-per-
  # mandate: Vigil is the agent that uses all the tools.
  payload=$(python3 -c "import json,sys;print(json.dumps({'source':'nvnc-station-watchdog','prompt':sys.stdin.read()}))" <<<"$prompt")
  (curl -s --max-time 180 -X POST -H "Content-Type: application/json" -d "$payload" "$VIGIL_URL" \
     > "/tmp/nvnc-vigil-wake-$(date +%s).log" 2>&1 &)
  echo "nvnc-station-health: woke Vigil (fire-and-forget, log in /tmp/nvnc-vigil-wake-*.log)"

  # Belt + suspenders: direct email to matthew as a redundant channel
  # in case Vigil itself is down when the station fails.
  ALERT_TO="${NVNC_STATION_ALERT_TO:-matthew@camp-corp.com}"
  if command -v mail >/dev/null 2>&1; then
    echo "$prompt" | mail -s "[NVNC] $UNIT stopped on $host (Vigil woken)" "$ALERT_TO" || true
  fi

  rm -f "$FAIL_FILE"
  exit 1
fi
exit 0
