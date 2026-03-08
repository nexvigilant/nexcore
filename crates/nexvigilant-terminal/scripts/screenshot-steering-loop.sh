#!/usr/bin/env bash
set -euo pipefail

# Screenshot Steering Loop for NexVigilant Terminal
#
# Captures the terminal UI at configurable intervals, producing
# a visual signal stream that Claude can Read to assess and steer
# development. Each capture is timestamped and annotated with the
# last remote controller action.
#
# Usage:
#   screenshot-steering-loop.sh [interval_secs] [max_captures] [mode]
#     interval_secs  — seconds between captures (default: 10)
#     max_captures   — stop after N captures, 0=unlimited (default: 20)
#     mode           — screenshot mode: full|efficient|thumb (default: thumb)
#
# Output:
#   /tmp/nv-steering/              — capture directory
#   /tmp/nv-steering/manifest.json — ordered list of captures with metadata
#   /tmp/nv-steering/latest.png    — symlink to most recent capture

INTERVAL="${1:-10}"
MAX_CAPTURES="${2:-20}"
MODE="${3:-thumb}"
CAPTURE_DIR="/tmp/nv-steering"
MANIFEST="${CAPTURE_DIR}/manifest.json"
CAPTURE_SCRIPT="${HOME}/.claude/hooks/bash/screenshot-capture.sh"

# Validate screenshot tool exists
if [[ ! -x "${CAPTURE_SCRIPT}" ]]; then
    echo "ERROR: Screenshot capture script not found at ${CAPTURE_SCRIPT}" >&2
    exit 1
fi

# Create capture directory
mkdir -p "${CAPTURE_DIR}"

# Initialize manifest
echo '{"captures":[],"started":"'"$(date -Iseconds)"'","interval":'"${INTERVAL}"',"mode":"'"${MODE}"'"}' > "${MANIFEST}"

echo "NexVigilant Steering Loop"
echo "  Interval: ${INTERVAL}s"
echo "  Max captures: ${MAX_CAPTURES}"
echo "  Mode: ${MODE}"
echo "  Output: ${CAPTURE_DIR}/"
echo ""

capture_count=0

while true; do
    capture_count=$((capture_count + 1))

    # Check max captures (0 = unlimited)
    if [[ "${MAX_CAPTURES}" -gt 0 ]] && [[ "${capture_count}" -gt "${MAX_CAPTURES}" ]]; then
        echo "Reached max captures (${MAX_CAPTURES}). Stopping."
        break
    fi

    timestamp=$(date -Iseconds)
    seq_id=$(printf "%04d" "${capture_count}")

    # Capture screenshot
    capture_path=$("${CAPTURE_SCRIPT}" "${CAPTURE_DIR}" "${MODE}" 2>/dev/null) || {
        echo "[${seq_id}] Capture failed at ${timestamp}" >&2
        sleep "${INTERVAL}"
        continue
    }

    # Rename to sequential
    ext="${capture_path##*.}"
    target="${CAPTURE_DIR}/capture-${seq_id}.${ext}"
    mv "${capture_path}" "${target}"

    # Update latest symlink
    ln -sf "${target}" "${CAPTURE_DIR}/latest.png"

    # Append to manifest (using python for JSON safety)
    python3 -c "
import json, sys
with open('${MANIFEST}', 'r') as f:
    m = json.load(f)
m['captures'].append({
    'seq': ${capture_count},
    'timestamp': '${timestamp}',
    'path': '${target}',
    'mode': '${MODE}'
})
m['latest'] = '${target}'
m['capture_count'] = ${capture_count}
with open('${MANIFEST}', 'w') as f:
    json.dump(m, f, indent=2)
" 2>/dev/null

    echo "[${seq_id}] Captured: ${target} (${timestamp})"

    # Sleep until next capture
    sleep "${INTERVAL}"
done

echo ""
echo "Steering loop complete. ${capture_count} captures in ${CAPTURE_DIR}/"
echo "Manifest: ${MANIFEST}"
