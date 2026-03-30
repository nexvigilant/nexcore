#!/usr/bin/env bash
set -uo pipefail

# publish-all.sh — Resilient crate publisher with rate limit handling
# Reads publish order from dag-publish, skips already-published, retries on 429

NEXCORE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
LOG="/tmp/dag-publish-full.log"
STATE="/tmp/dag-publish-published.txt"
touch "$STATE"

cd "$NEXCORE_ROOT"

published=0
skipped=0
failed=0
total=0

while IFS= read -r crate; do
    total=$((total + 1))
    
    # Skip if already published in this run
    if grep -qx "$crate" "$STATE" 2>/dev/null; then
        skipped=$((skipped + 1))
        continue
    fi
    
    # Find crate directory
    crate_dir=""
    if [ -d "crates/$crate" ]; then
        crate_dir="crates/$crate"
    elif [ -d "tools/$crate" ]; then
        crate_dir="tools/$crate"
    else
        echo "[$total] $crate: NOT FOUND" | tee -a "$LOG"
        failed=$((failed + 1))
        continue
    fi
    
    # Check publish field
    if grep -q 'publish = false' "$crate_dir/Cargo.toml" 2>/dev/null; then
        echo "[$total] $crate: SKIP (publish=false)" | tee -a "$LOG"
        echo "$crate" >> "$STATE"
        skipped=$((skipped + 1))
        continue
    fi
    
    # Attempt publish with retries
    max_retries=3
    retry=0
    while [ $retry -lt $max_retries ]; do
        output=$(cargo publish --manifest-path "$crate_dir/Cargo.toml" --allow-dirty 2>&1)
        exit_code=$?
        
        if [ $exit_code -eq 0 ]; then
            echo "[$total] $crate: PUBLISHED" | tee -a "$LOG"
            echo "$crate" >> "$STATE"
            published=$((published + 1))
            sleep 5
            break
        elif echo "$output" | grep -q "already exists"; then
            echo "[$total] $crate: ALREADY EXISTS" | tee -a "$LOG"
            echo "$crate" >> "$STATE"
            skipped=$((skipped + 1))
            break
        elif echo "$output" | grep -q "429"; then
            retry=$((retry + 1))
            wait_time=$((60 * retry))
            echo "[$total] $crate: RATE LIMITED, waiting ${wait_time}s (retry $retry/$max_retries)" | tee -a "$LOG"
            sleep "$wait_time"
        else
            echo "[$total] $crate: FAILED" | tee -a "$LOG"
            echo "  $output" | head -3 >> "$LOG"
            failed=$((failed + 1))
            break
        fi
    done
done < /tmp/publish-order.txt

echo "" | tee -a "$LOG"
echo "=== FINAL SUMMARY ===" | tee -a "$LOG"
echo "Total: $total" | tee -a "$LOG"
echo "Published: $published" | tee -a "$LOG"
echo "Skipped: $skipped" | tee -a "$LOG"
echo "Failed: $failed" | tee -a "$LOG"
