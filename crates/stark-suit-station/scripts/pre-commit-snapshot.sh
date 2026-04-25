#!/usr/bin/env bash
# stark-suit-station snapshot contract gate.
#
# Wire from the workspace root via:
#   ln -s ../../crates/stark-suit-station/scripts/pre-commit-snapshot.sh .git/hooks/pre-commit
#
# Or chain alongside an existing hook in .githooks/. Fails the commit if:
#   - the binary is missing
#   - the snapshot JSON does not parse
#   - top-level keys drift from the contract
#   - compound_count != 4
#   - any compound's `tick` field is missing or null

set -euo pipefail

BIN="${CARGO_TARGET_DIR:-./target}/release/stark-suit-station"
if [[ ! -x "$BIN" ]]; then
    echo "pre-commit-snapshot: binary missing at $BIN — run: cargo build --release -p stark-suit-station --bins" >&2
    exit 1
fi

SNAP="$("$BIN" tick 2>/dev/null)"
if [[ -z "$SNAP" ]]; then
    echo "pre-commit-snapshot: tick produced no output" >&2
    exit 1
fi

EXPECTED='compound_count control human_interface perception power total_ticks'
ACTUAL="$(printf '%s' "$SNAP" | jq -r 'keys | join(" ")')"
if [[ "$ACTUAL" != "$EXPECTED" ]]; then
    echo "pre-commit-snapshot: top-level keys drifted" >&2
    echo "  expected: $EXPECTED" >&2
    echo "  actual:   $ACTUAL" >&2
    exit 1
fi

CC="$(printf '%s' "$SNAP" | jq -r '.compound_count')"
if [[ "$CC" != "4" ]]; then
    echo "pre-commit-snapshot: compound_count=$CC, expected 4" >&2
    exit 1
fi

NULL_TICKS="$(printf '%s' "$SNAP" \
    | jq -r '[.perception.tick, .power.tick, .control.tick, .human_interface.tick] | map(select(. == null)) | length')"
if [[ "$NULL_TICKS" != "0" ]]; then
    echo "pre-commit-snapshot: $NULL_TICKS compound(s) have null tick fields" >&2
    exit 1
fi

echo "pre-commit-snapshot: contract OK (compound_count=4, ticks live)"
