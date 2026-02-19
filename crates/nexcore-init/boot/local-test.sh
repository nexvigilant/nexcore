#!/usr/bin/env bash
# NexCore OS — Local Boot Test (no Docker/QEMU required)
#
# Runs nexcore-init directly on the host in virtual mode.
# Tests all three form factors with bounded ticks.
#
# Usage:
#   ./boot/local-test.sh           # Test all form factors
#   ./boot/local-test.sh desktop   # Test one form factor

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/../../.." && pwd)"
FORM_FACTOR="${1:-all}"
BINARY="$WORKSPACE_DIR/target/release/nexcore-init"

echo "=== NexCore OS — Local Boot Test ==="
echo ""

# Build if needed
if [ ! -f "$BINARY" ]; then
    echo ">>> Building nexcore-init (release)..."
    cd "$WORKSPACE_DIR"
    cargo build --release -p nexcore-init 2>&1
    echo ""
fi

run_test() {
    local ff="$1"
    local ticks="${2:-10}"

    echo "--- Testing: $ff (${ticks} ticks) ---"

    local output
    output=$("$BINARY" --virtual --form-factor "$ff" --ticks "$ticks" 2>&1) || {
        echo "  FAIL: Binary exited with error"
        echo "$output" | tail -5
        return 1
    }

    # Verify expected output
    local ok=true

    if ! echo "$output" | grep -q "NexCore OS booted successfully"; then
        echo "  FAIL: Missing boot success"
        ok=false
    fi

    if ! echo "$output" | grep -q "Shell booted"; then
        echo "  FAIL: Missing shell boot"
        ok=false
    fi

    if ! echo "$output" | grep -q "NexCore OS halted"; then
        echo "  FAIL: Missing shutdown message"
        ok=false
    fi

    # Form-factor-specific checks
    case $ff in
        watch)
            echo "$output" | grep -q "2 regions" || { echo "  FAIL: Watch should have 2 regions"; ok=false; }
            ;;
        phone)
            echo "$output" | grep -q "3 regions" || { echo "  FAIL: Phone should have 3 regions"; ok=false; }
            ;;
        desktop)
            echo "$output" | grep -q "2 regions" || { echo "  FAIL: Desktop should have 2 regions"; ok=false; }
            ;;
    esac

    if $ok; then
        local frames
        frames=$(echo "$output" | grep -o '[0-9]* frames rendered' | grep -o '^[0-9]*')
        echo "  PASS: $ff — booted, $ticks ticks, $frames frames rendered"
    else
        echo "  Output:"
        echo "$output" | sed 's/^/    /'
        return 1
    fi
}

FAILURES=0

if [ "$FORM_FACTOR" = "all" ]; then
    for ff in desktop watch phone; do
        run_test "$ff" 10 || FAILURES=$((FAILURES + 1))
    done
else
    run_test "$FORM_FACTOR" 10 || FAILURES=$((FAILURES + 1))
fi

echo ""
if [ "$FAILURES" -eq 0 ]; then
    echo "=== All local boot tests PASSED ==="
else
    echo "=== $FAILURES test(s) FAILED ==="
    exit 1
fi
