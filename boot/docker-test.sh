#!/usr/bin/env bash
# NexCore OS — Docker PID 1 Test
#
# Builds and runs nexcore-init as PID 1 in a Docker container.
# Tests all three form factors and validates boot + shutdown.
#
# Usage:
#   ./boot/docker-test.sh              # Test all form factors
#   ./boot/docker-test.sh desktop      # Test one form factor
#   ./boot/docker-test.sh --build-only # Build image only

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(dirname "$SCRIPT_DIR")"
IMAGE="nexcore-os:test"
FORM_FACTOR="${1:-all}"

echo "=== NexCore OS — Docker PID 1 Test ==="
echo ""

# Step 1: Build the Docker image
if [ "$FORM_FACTOR" = "--build-only" ]; then
    echo ">>> Building Docker image..."
    cd "$WORKSPACE_DIR"
    docker build -t "$IMAGE" -f boot/Dockerfile . 2>&1
    echo ""
    echo "Image built: $IMAGE"
    docker images "$IMAGE"
    exit 0
fi

# Build if image doesn't exist
if ! docker image inspect "$IMAGE" &>/dev/null; then
    echo ">>> Building Docker image (first run)..."
    cd "$WORKSPACE_DIR"
    docker build -t "$IMAGE" -f boot/Dockerfile . 2>&1
    echo ""
fi

# Step 2: Run tests
run_test() {
    local ff="$1"
    local ticks="${2:-10}"

    echo "--- Testing: $ff (${ticks} ticks) ---"

    local output
    output=$(docker run --rm --init "$IMAGE" --form-factor "$ff" --ticks "$ticks" 2>&1) || {
        echo "  FAIL: Container exited with error"
        echo "$output" | tail -5
        return 1
    }

    # Verify expected output
    local ok=true

    if ! echo "$output" | grep -q "NexCore Init starting"; then
        echo "  FAIL: Missing init start message"
        ok=false
    fi

    if ! echo "$output" | grep -q "NexCore OS booted successfully"; then
        echo "  FAIL: Missing boot success"
        ok=false
    fi

    if ! echo "$output" | grep -q "Shell booted"; then
        echo "  FAIL: Missing shell boot"
        ok=false
    fi

    if ! echo "$output" | grep -q "NexCore OS halted after $ticks ticks"; then
        echo "  FAIL: Missing shutdown message"
        ok=false
    fi

    if $ok; then
        echo "  PASS: $ff — booted, ran $ticks ticks, shutdown clean"
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
    echo "=== All PID 1 boot tests PASSED ==="
else
    echo "=== $FAILURES test(s) FAILED ==="
    exit 1
fi
