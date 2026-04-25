#!/usr/bin/env bash
# stark-suit BMS backend matrix runner.
#
# Closes the v0.4 D-2 carry-over: forces the BmsSource trait to stay
# backend-agnostic by exercising every backend on demand. Wired as a
# local script (no .github/workflows) per the no-github-workflows
# directive — invoke from a Vigil-monitored runner, a justfile recipe,
# or a pre-push hook.
#
# Usage:
#   bms-matrix.sh                     # all 3 backends
#   bms-matrix.sh mock                # single backend
#   bms-matrix.sh replay serial-pty   # subset
#
# Exit code: non-zero on any failure. Each backend runs independently;
# failures are reported but do not short-circuit the rest.

set -uo pipefail

REPO_ROOT="$(git -C "$(dirname "$0")" rev-parse --show-toplevel)"
cd "$REPO_ROOT"

BACKENDS=("$@")
if [[ ${#BACKENDS[@]} -eq 0 ]]; then
    BACKENDS=(mock replay serial-pty perception-mock perception-replay perception-serial-pty)
fi

declare -A RESULTS
overall=0

run_backend() {
    local name="$1"
    echo
    echo "===== bms-matrix: $name ====="
    case "$name" in
        mock)
            cargo test -p stark-suit-station --bins -- bms::tests::mock_
            ;;
        replay)
            cargo test -p stark-suit-station --bins -- bms::tests::replay_
            ;;
        serial-pty)
            cargo test -p stark-suit-station --test serial_pty_roundtrip
            ;;
        perception-mock)
            cargo test -p stark-suit-station --lib -- perception::tests::mock_
            ;;
        perception-replay)
            cargo test -p stark-suit-station --lib -- perception::tests::replay_
            ;;
        perception-serial-pty)
            cargo test -p stark-suit-station --test perception_pty_roundtrip
            ;;
        *)
            echo "unknown backend: $name (expected mock|replay|serial-pty)" >&2
            return 2
            ;;
    esac
}

for b in "${BACKENDS[@]}"; do
    if run_backend "$b"; then
        RESULTS[$b]="PASS"
    else
        RESULTS[$b]="FAIL($?)"
        overall=1
    fi
done

# Smoke contract on default backend after the matrix.
echo
echo "===== bms-matrix: smoke contract ====="
cargo build --release -p stark-suit-station --bins >/dev/null 2>&1 || {
    echo "release build failed" >&2
    overall=1
}
BIN="${CARGO_TARGET_DIR:-$REPO_ROOT/target}/release/stark-suit-station"
if [[ -x "$BIN" ]]; then
    SNAP="$("$BIN" tick 2>/dev/null)"
    if echo "$SNAP" | jq -e '.compound_count == 4 and (.total_ticks // 0) > 0' >/dev/null 2>&1; then
        RESULTS[smoke]="PASS"
    else
        RESULTS[smoke]="FAIL"
        overall=1
        echo "snapshot contract failed:"
        echo "$SNAP"
    fi
else
    RESULTS[smoke]="MISSING_BIN"
    overall=1
fi

echo
echo "===== bms-matrix: summary ====="
for k in "${!RESULTS[@]}"; do
    printf "  %-12s %s\n" "$k" "${RESULTS[$k]}"
done

exit "$overall"
