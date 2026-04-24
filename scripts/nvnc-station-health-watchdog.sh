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

# ── Self-test mode ─────────────────────────────────────────────────
# Fire all three alert channels (cytokine + Vigil wake + email)
# WITHOUT stopping the station. Confirms the alert path is intact
# before a real failure depends on it.
#
#   bash nvnc-station-health-watchdog.sh --self-test
#
# Exit 0 if all channels returned success, 1 if any failed.
if [[ "${1:-}" == "--self-test" ]]; then
  echo "nvnc-station-health: SELF-TEST — firing all 3 channels"
  ts=$(date -Is); host=$(hostname)
  fail=0

  # 1. Cytokine
  CYTO_HELPER="${NVNC_CYTOKINE_EMIT:-/home/matthew/nvnc/station/nvnc-cytokine-emit.sh}"
  if [[ -x "$CYTO_HELPER" ]]; then
    resp=$("$CYTO_HELPER" il6 nvnc_station_self_test low autocrine \
      "{\"self_test\":true,\"host\":\"$host\",\"ts\":\"$ts\"}" 2>&1 || true)
    sid=$(grep -oE 'IL-[0-9]+-[0-9a-f]{16}' <<<"$resp" | head -1)
    if [[ -n "$sid" ]]; then echo "  ✓ cytokine emit: $sid"; else echo "  ✗ cytokine emit: $resp"; fail=1; fi
  else
    echo "  ✗ cytokine helper not executable: $CYTO_HELPER"; fail=1
  fi

  # 2. Vigil wake — check /health first (cheap), then fire-and-forget
  #    the wake itself. Full turn takes 15-180s; self-test only verifies
  #    the endpoint is reachable + the wake POST is accepted.
  VIGIL_HEALTH="${NVNC_VIGIL_HEALTH:-http://127.0.0.1:7823/health}"
  VIGIL_URL="${NVNC_VIGIL_URL:-http://127.0.0.1:7823/wake}"
  health_code=$(curl -s -o /dev/null -w '%{http_code}' --max-time 3 "$VIGIL_HEALTH" || echo "000")
  if [[ "$health_code" == "200" ]]; then
    (curl -s -o /dev/null --max-time 180 \
      -X POST -H "Content-Type: application/json" \
      -d "{\"source\":\"nvnc-station-watchdog-selftest\",\"prompt\":\"self-test ping at $ts\"}" \
      "$VIGIL_URL" &)
    echo "  ✓ vigil wake: endpoint healthy, wake dispatched"
  else
    echo "  ✗ vigil wake: /health returned $health_code"; fail=1
  fi

  # 3. Email
  ALERT_TO="${NVNC_STATION_ALERT_TO:-matthew@camp-corp.com}"
  if command -v mail >/dev/null 2>&1; then
    echo "NVNC watchdog self-test on $host at $ts" | \
      mail -s "[NVNC self-test] watchdog alert channels $ts" "$ALERT_TO" \
      && echo "  ✓ mail: queued to $ALERT_TO" || { echo "  ✗ mail: send failed"; fail=1; }
  else
    echo "  ✗ mail: /usr/bin/mail not available"; fail=1
  fi

  echo "nvnc-station-health: self-test $([ $fail -eq 0 ] && echo PASS || echo FAIL)"
  exit $fail
fi

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

  # Guaranteed cytokine emit — direct to nexcore-mcp stdio so the
  # signal lands on the bus even if Vigil's model can't tool-call.
  # Vigil wakes in parallel for narrative + orchestration; this is the
  # contract-enforcing belt, Vigil is the smart suspender.
  CYTO_HELPER="${NVNC_CYTOKINE_EMIT:-/home/matthew/nvnc/station/nvnc-cytokine-emit.sh}"
  signal_id=""
  if [[ -x "$CYTO_HELPER" ]]; then
    payload_json=$(printf '{"unit":"%s","host":"%s","url":"%s","consecutive_failures":%s,"ts":"%s"}' \
      "$UNIT" "$host" "$URL" "$MAX_FAILS" "$ts")
    cyto_resp=$("$CYTO_HELPER" il1 nvnc_station_down critical systemic "$payload_json" 2>&1 || true)
    signal_id=$(grep -oE 'IL-1-[0-9a-f]{16}' <<<"$cyto_resp" | head -1)
    echo "nvnc-station-health: cytokine emitted signal_id=${signal_id:-UNKNOWN}"
  fi

  # Vigil wake — agent gets the context for narrative/escalation
  # decisions (email body, next-step suggestions, incident artifact).
  # Fire-and-forget; ~60-180s turn.
  VIGIL_URL="${NVNC_VIGIL_URL:-http://127.0.0.1:7823/wake}"
  payload=$(python3 -c "import json,sys;print(json.dumps({'source':'nvnc-station-watchdog','prompt':sys.stdin.read()}))" <<<"$prompt")
  (curl -s --max-time 180 -X POST -H "Content-Type: application/json" -d "$payload" "$VIGIL_URL" \
     > "/tmp/nvnc-vigil-wake-$(date +%s).log" 2>&1 &)
  echo "nvnc-station-health: woke Vigil (fire-and-forget, log in /tmp/nvnc-vigil-wake-*.log)"

  # Direct email — the third channel, never depends on Vigil or bus.
  ALERT_TO="${NVNC_STATION_ALERT_TO:-matthew@camp-corp.com}"
  if command -v mail >/dev/null 2>&1; then
    subject="[NVNC] $UNIT stopped on $host"
    [[ -n "$signal_id" ]] && subject="$subject (cytokine $signal_id)"
    echo "$prompt" | mail -s "$subject" "$ALERT_TO" || true
  fi

  rm -f "$FAIL_FILE"
  exit 1
fi
exit 0
