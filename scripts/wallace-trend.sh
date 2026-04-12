#!/usr/bin/env bash
set -euo pipefail

# Wallace Trend Tracker — appends quality snapshot to JSONL file per run.
# Usage: ./scripts/wallace-trend.sh [crates_dir]
# Output: ~/.claude/wallace-trend.jsonl

CRATES_DIR="${1:-crates}"
TREND_FILE="${HOME}/.claude/wallace-trend.jsonl"
COMMIT=$(git rev-parse --short HEAD 2>/dev/null || echo "unknown")
DATE=$(date -u +"%Y-%m-%dT%H:%M:%SZ")

# Run wallace pipeline and capture JSON
OUTPUT=$(cargo run -p nexcore-wallace --bin wallace -- scan "$CRATES_DIR" --json 2>/dev/null)

if [ -z "$OUTPUT" ]; then
    echo "Error: wallace scan produced no output" >&2
    exit 1
fi

# Extract totals from crate-level data
TOTALS=$(echo "$OUTPUT" | python3 -c "
import sys, json
r = json.load(sys.stdin)
total = sum(len(c['violations']) for c in r['crates'])
actionable = sum(1 for c in r['crates'] for v in c['violations'] if v['classification'] in ('Mechanical','SignatureLift','Movable','Unnecessary'))
print(f'{total} {actionable} {len(r[\"crates\"])}')
")
TOTAL=$(echo "$TOTALS" | cut -d' ' -f1)
ACTIONABLE=$(echo "$TOTALS" | cut -d' ' -f2)
CRATE_COUNT=$(echo "$TOTALS" | cut -d' ' -f3)

# Append to trend file
echo "{\"date\":\"$DATE\",\"commit\":\"$COMMIT\",\"total\":$TOTAL,\"actionable\":$ACTIONABLE,\"crates\":$CRATE_COUNT}" >> "$TREND_FILE"

echo "Recorded: commit=$COMMIT total=$TOTAL actionable=$ACTIONABLE crates=$CRATE_COUNT"
echo "Trend file: $TREND_FILE ($(wc -l < "$TREND_FILE") entries)"
