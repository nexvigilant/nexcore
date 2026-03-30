#!/usr/bin/env bash
set -euo pipefail

# publish-kellnr.sh — Publish all crates to Kellnr (crates.nexvigilant.com)
# No rate limits. Runs in DAG order. Skips already-published.
#
# Usage:
#   ./scripts/publish-kellnr.sh              # Publish all
#   ./scripts/publish-kellnr.sh --dry-run    # Show order only
#   ./scripts/publish-kellnr.sh --filter stem-  # Only stem-* crates

NEXCORE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$NEXCORE_ROOT"

REGISTRY="nexcore"
DELAY=2  # 2s between publishes (Kellnr handles fast)
DRY_RUN=false
FILTER=""
LIMIT=0

# Parse args
while [ $# -gt 0 ]; do
    case "$1" in
        --dry-run) DRY_RUN=true; shift ;;
        --filter) FILTER="$2"; shift 2 ;;
        --limit) LIMIT="$2"; shift 2 ;;
        *) echo "Unknown arg: $1"; exit 1 ;;
    esac
done

# Build dag-publish if needed
if [ ! -f target/release/dag-publish ]; then
    echo "Building dag-publish..."
    cargo build -p dag-publish --release
fi

# Construct args
ARGS="--crates-dir crates --include-tools --delay $DELAY --registry $REGISTRY"

if [ -n "$FILTER" ]; then
    ARGS="$ARGS --filter $FILTER"
fi

if [ "$LIMIT" -gt 0 ]; then
    ARGS="$ARGS --limit $LIMIT"
fi

if [ "$DRY_RUN" = true ]; then
    ARGS="$ARGS --show-phases"
    echo "=== DRY RUN: Kellnr publish order ==="
    ./target/release/dag-publish $ARGS
else
    ARGS="$ARGS --skip-published --allow-dirty"
    echo "=== Publishing to Kellnr ($REGISTRY) ==="
    echo "Registry: crates.nexvigilant.com"
    echo "Delay: ${DELAY}s between crates"
    echo ""
    ./target/release/dag-publish $ARGS
fi
