#!/usr/bin/env bash
set -euo pipefail

ROOT="/home/matthew/Projects/Active/nexcore"
STATION="$ROOT/crates/nexcore-station/src"

printf '== Station Wiring Audit ==\n'
printf 'root: %s\n' "$ROOT"

printf '\n== Telemetry Producers (active path) ==\n'
rg -n "emit_resolve_start|emit_resolve_finish|emit_feed_http|with_trace_id|current_trace_id" \
  "$STATION/client.rs" "$STATION/feed.rs" "$STATION/telemetry.rs" || true

printf '\n== Legacy Path Check ==\n'
for f in "$STATION/resolution.rs" "$STATION/observatory.rs"; do
  if [ -f "$f" ]; then
    echo "present: $f"
    rg -n "emit_resolve_start|emit_resolve_finish|emit_feed_http" "$f" || echo "  telemetry hooks: none"
  fi
done

printf '\n== Notebooks Wired ==\n'
python3 - <<'PY'
import json
from pathlib import Path
for p in [
    Path('/home/matthew/station-train-live-observatory.ipynb'),
    Path('/home/matthew/station-health.ipynb'),
]:
    ok=False
    if p.exists():
        nb=json.loads(p.read_text())
        text='\n'.join(''.join(c.get('source',[])) for c in nb.get('cells',[]))
        ok='NEXCORE_STATION_EVENT_LOG' in text
    print(f'{p}: {"wired" if ok else "missing_live_stream_wiring"}')
PY

printf '\n== MCP moltbrowser config ==\n'
codex mcp get moltbrowser-mcp || true
