#!/usr/bin/env bash
# Release all nexcore crates in DAG phase order using release-plz
#
# Usage:
#   ./tools/release.sh              # Release all changed crates
#   ./tools/release.sh --dry-run    # Show what would be released
#   ./tools/release.sh nexcore-error # Release a single crate

set -euo pipefail

NEXCORE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CRATES_DIR="$NEXCORE_ROOT/crates"
DRY_RUN=""
SINGLE_CRATE=""

export CARGO_REGISTRIES_NEXCORE_INDEX="sparse+http://localhost:8000/api/v1/crates/"
export CARGO_REGISTRIES_NEXCORE_TOKEN="${CARGO_REGISTRIES_NEXCORE_TOKEN:-WUq8ItiDTqiWtcYnKD29FgCrVw3YoAPH}"

for arg in "$@"; do
    case "$arg" in
        --dry-run) DRY_RUN="--dry-run" ;;
        *) SINGLE_CRATE="$arg" ;;
    esac
done

# Single crate mode
if [ -n "$SINGLE_CRATE" ]; then
    echo "Releasing $SINGLE_CRATE..."
    release-plz release \
        --registry nexcore \
        --manifest-path "$CRATES_DIR/$SINGLE_CRATE/Cargo.toml" \
        --config "$NEXCORE_ROOT/release-plz.toml" \
        $DRY_RUN
    exit 0
fi

# Get DAG phase ordering
echo "Getting DAG phase ordering..."
PHASES=$(cargo run --manifest-path "$NEXCORE_ROOT/tools/dag-publish/Cargo.toml" \
    -- --crates-dir "$CRATES_DIR" --show-phases --dry-run 2>/dev/null | \
    grep -E '^\s+\S' | sed 's/^ *//')

TOTAL=$(echo "$PHASES" | wc -l)
echo "Found $TOTAL crates"
echo ""

RELEASED=0
SKIPPED=0
FAILED=0
COUNT=0

while IFS= read -r crate_name; do
    [ -z "$crate_name" ] && continue
    COUNT=$((COUNT + 1))

    crate_dir="$CRATES_DIR/$crate_name"
    if [ ! -f "$crate_dir/Cargo.toml" ]; then
        echo "  [$COUNT/$TOTAL] [SKIP] $crate_name: not found"
        SKIPPED=$((SKIPPED + 1))
        continue
    fi

    echo "  [$COUNT/$TOTAL] Releasing $crate_name..."
    if release-plz release \
        --registry nexcore \
        --manifest-path "$crate_dir/Cargo.toml" \
        --config "$NEXCORE_ROOT/release-plz.toml" \
        $DRY_RUN 2>&1 | tail -3; then
        RELEASED=$((RELEASED + 1))
    else
        echo "    -> FAILED"
        FAILED=$((FAILED + 1))
    fi
done <<< "$PHASES"

echo ""
echo "============================================================"
echo "RELEASE SUMMARY"
echo "============================================================"
echo "  Released: $RELEASED"
echo "  Skipped:  $SKIPPED"
echo "  Failed:   $FAILED"
