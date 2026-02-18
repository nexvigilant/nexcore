# NexCore Development Justfile
# Usage: just <recipe> [args]
# Run `just --list` to see all recipes

# Default recipe: check + clippy
default: check clippy

# ── Build & Check ────────────────────────────────────────────────

# Fast type check (no codegen)
check:
    cargo check --workspace

# Full release build
build:
    cargo build --release --workspace

# Check a single crate
check-crate crate:
    cargo check -p {{crate}}

# ── Lint ─────────────────────────────────────────────────────────

# Clippy with deny warnings (matches CI)
clippy:
    cargo clippy --workspace -- -D warnings

# Format check (no modify)
fmt-check:
    cargo fmt --all -- --check

# Format fix
fmt:
    cargo fmt --all

# ── Test ─────────────────────────────────────────────────────────

# Core vigilance tests (fastest feedback, 3300+ tests)
test-core:
    cargo test -p nexcore-vigilance --lib

# Full workspace tests
test:
    cargo test --workspace

# Full workspace tests with nextest (parallel)
test-fast:
    cargo nextest run --workspace

# Test single crate
test-crate crate:
    cargo test -p {{crate}}

# Test with pattern match
test-match crate pattern:
    cargo test -p {{crate}} -- {{pattern}}

# Test nexcore-renderer (313 tests)
test-renderer:
    cargo test -p nexcore-renderer

# Test nexcloud (SHELVED — excluded from workspace)
# test-nexcloud:
#     cargo test -p nexcloud

# Test Prima language (868 tests)
test-prima:
    cd ~/prima && cargo test --workspace

# ── Validation Pipeline ──────────────────────────────────────────

# Full CI validation via DAG orchestrator: fmt → (clippy | test | docs) → build
validate:
    cargo run --release -p nexcore-build-orchestrator --bin orchestrator-cli -- run validate

# Quick validation via DAG orchestrator: check → (clippy | test-core)
validate-quick:
    cargo run --release -p nexcore-build-orchestrator --bin orchestrator-cli -- run validate-quick

# Sequential validation (fallback, no orchestrator)
validate-seq: fmt-check clippy test anatomy-check

# Sequential quick validation (fallback, no orchestrator)
validate-quick-seq: check clippy test-core anatomy-check

# ── Brain Milestones (Crate Workflow Gate) ──────────────────────

# Capture crate task milestone (save + resolve)
brain-crate-task crate *content:
    ./scripts/brain-crate-milestone {{crate}} task "{{content}}"

# Capture crate plan milestone (save + resolve)
brain-crate-plan crate *content:
    ./scripts/brain-crate-milestone {{crate}} plan "{{content}}"

# Capture crate handoff milestone (save + resolve)
brain-crate-handoff crate *content:
    ./scripts/brain-crate-milestone {{crate}} handoff "{{content}}"

# Gate crate workflow before finish/release
brain-crate-gate crate:
    ./scripts/brain-crate-gate {{crate}} --strict-markers

# Audit recent sessions for missing required resolved milestones
brain-crate-audit limit="20":
    ./scripts/brain-crate-audit {{limit}}

# Strict audit (non-zero when any milestone gaps are found)
brain-crate-audit-strict limit="20":
    ./scripts/brain-crate-audit {{limit}} --strict

# Crate completion gate: brain gate + compile + lint + tests
brain-crate-ready crate:
    ./scripts/brain-crate-gate {{crate}} --strict-markers
    cargo check -p {{crate}}
    cargo clippy -p {{crate}} -- -D warnings
    cargo test -p {{crate}}

# ── MCP & Services ───────────────────────────────────────────────

# Build and install MCP server
mcp-build:
    cargo build --release -p nexcore-mcp

# Build and install API server
api-build:
    cargo build --release -p nexcore-api

# Build and install CLI
cli-build:
    cargo build --release -p nexcore-cli

# Build orchestrator CLI
orc-build:
    cargo build --release -p nexcore-build-orchestrator --bin orchestrator-cli

# Build orchestrator SSR dashboard
orc-dashboard:
    cargo build --release -p nexcore-build-orchestrator --features ssr

# Rebuild all service binaries
services: mcp-build api-build cli-build orc-build

# ── Operator Toolkit ───────────────────────────────────────────

# Launch all NexCore services (build + tmux + API)
up *args:
    nexcore-up {{args}}

# Launch services (skip build, use existing binaries)
up-fast:
    nexcore-up --skip-build

# Graceful shutdown of all services
down:
    nexcore-down

# Show service health status
status:
    nexcore-status

# Query the running API
query *args:
    nexcore-query {{args}}

# Run 12-system physical exam
exam *args:
    nexcore-exam {{args}}

# Quick vitals check only
vitals:
    nexcore-exam vitals

# ── Build Orchestrator ──────────────────────────────────────────

# Run pipeline (default: validate-quick)
orc-run pipeline="validate-quick":
    cargo run --release -p nexcore-build-orchestrator --bin orchestrator-cli -- run {{pipeline}}

# Show most recent pipeline status
orc-status:
    cargo run --release -p nexcore-build-orchestrator --bin orchestrator-cli -- status

# Show build history
orc-history n="10":
    cargo run --release -p nexcore-build-orchestrator --bin orchestrator-cli -- history -n {{n}}

# Dry-run pipeline plan (DAG wave visualization)
orc-plan pipeline="validate":
    cargo run --release -p nexcore-build-orchestrator --bin orchestrator-cli -- plan {{pipeline}}

# Scan workspace crates and dirty status
orc-workspace:
    cargo run --release -p nexcore-build-orchestrator --bin orchestrator-cli -- workspace

# Prune old history entries
orc-prune keep="50":
    cargo run --release -p nexcore-build-orchestrator --bin orchestrator-cli -- prune {{keep}}

# Start orchestrator web dashboard (port 3100)
orc-serve port="3100":
    cargo run --release -p nexcore-build-orchestrator --features ssr --bin build-orchestrator -- --port {{port}}

# ── NexBrowser ───────────────────────────────────────────────────

# Build NexBrowser
browser-build:
    cargo build --release -p nexcore-renderer

# Run NexBrowser (requires DISPLAY)
browser-run:
    DISPLAY=:0 cargo run --release -p nexcore-renderer

# ── Hooks ────────────────────────────────────────────────────────

# Build hook binaries (separate workspace)
hooks-build:
    cd ~/.claude/hooks && cargo build --release

# ── Analysis ─────────────────────────────────────────────────────

# Count workspace crates
crate-count:
    @ls -d crates/*/ | wc -l

# Count total tests
test-count:
    @cargo test --workspace -- --list 2>/dev/null | grep -c "test$" || echo "run 'just test' first"

# Show workspace member tree
members:
    @cargo metadata --no-deps --format-version=1 2>/dev/null | jq -r '.packages[].name' 2>/dev/null || echo "requires cargo metadata"

# Measure crate health
measure crate_path:
    cargo run --release -p nexcore-measure -- crate {{crate_path}}

# Anatomy check (default: tolerate severity-1 violations)
anatomy-check:
    cargo run --release -p nexcore-anatomy --bin anatomy-check

# Anatomy check strict (exit 1 on ANY violation)
anatomy-check-strict:
    cargo run --release -p nexcore-anatomy --bin anatomy-check -- --strict

# Anatomy report as JSON
anatomy-json:
    cargo run --release -p nexcore-anatomy --bin anatomy-check -- --json

# ── Documentation ────────────────────────────────────────────────

# Build docs (all workspace)
docs:
    cargo doc --workspace --no-deps

# Build and open docs
docs-open:
    cargo doc --workspace --no-deps --open

# ── Cleanup ──────────────────────────────────────────────────────

# Clean build artifacts
clean:
    cargo clean

# Clean and rebuild
rebuild: clean build

# ── Waste Management (Lysosomal System) ────────────────────────

# Audit: show waste accumulation across all systems
waste-audit:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "═══════════════════════════════════════════════"
    echo "  WASTE AUDIT — NexCore Cell Lysosomal Report"
    echo "═══════════════════════════════════════════════"
    echo ""
    echo "── Target Directories (Build Artifacts) ──"
    du -sh ~/nexcore/target 2>/dev/null || echo "  (none)"
    du -sh ~/.claude/hooks/target 2>/dev/null || echo "  (none)"
    for d in ~/projects/*/target; do
        [ -d "$d" ] && du -sh "$d"
    done
    echo ""
    echo "── JSONL Telemetry (Unbounded Growth) ──"
    for f in ~/.claude/logs/hook_telemetry.jsonl \
             ~/.claude/debug/error_index.jsonl \
             ~/.claude/decision-audit/decisions.jsonl \
             ~/.claude/brain/telemetry/hook_executions.jsonl \
             ~/.claude/brain/telemetry/signals.jsonl \
             ~/.claude/brain/telemetry/skill_invocations.jsonl; do
        if [ -f "$f" ]; then
            lines=$(wc -l < "$f")
            size=$(du -sh "$f" | cut -f1)
            printf "  %-50s %6s lines  %s\n" "$(basename $f)" "$lines" "$size"
        fi
    done
    echo ""
    echo "── Session & Debug Accumulation ──"
    du -sh ~/.claude/debug/ 2>/dev/null || echo "  debug/: (none)"
    du -sh ~/.claude/file-history/ 2>/dev/null || echo "  file-history/: (none)"
    du -sh ~/.claude/brain/sessions/ 2>/dev/null || echo "  brain/sessions/: (none)"
    echo ""
    echo "── Docker Storage ──"
    du -sh ~/.local/share/docker 2>/dev/null || echo "  (docker not found)"
    echo ""
    echo "── Archive ──"
    du -sh ~/archive 2>/dev/null || echo "  (none)"
    echo ""
    echo "═══════════════════════════════════════════════"

# Rotate oversized JSONL files (run waste-collector binary)
waste-rotate:
    @echo "Running JSONL rotation..."
    @echo '{}' | ~/.claude/hooks/target/release/waste-collector

# Clean ALL target directories (workspace + hooks + projects)
clean-all-targets:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Cleaning all target directories..."
    total=0
    for d in ~/nexcore/target \
             ~/.claude/hooks/target \
             ~/projects/*/target \
             ~/prima/target; do
        if [ -d "$d" ]; then
            size=$(du -sb "$d" | cut -f1)
            total=$((total + size))
            echo "  rm -rf $d ($(du -sh "$d" | cut -f1))"
            rm -rf "$d"
        fi
    done
    echo "Freed: $((total / 1024 / 1024)) MB"

# Clean stale target dirs in projects/ (>30 days untouched)
clean-stale-targets:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Cleaning stale project targets (>30 days)..."
    find ~/projects -maxdepth 2 -name "target" -type d -mtime +30 2>/dev/null | while read d; do
        echo "  rm -rf $d ($(du -sh "$d" | cut -f1))"
        rm -rf "$d"
    done
    echo "Done."

# Prune Docker: remove dangling images, stopped containers, build cache
clean-docker:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Docker before: $(du -sh ~/.local/share/docker 2>/dev/null | cut -f1)"
    docker system prune -af --volumes 2>/dev/null || echo "Docker not running or not installed"
    echo "Docker after:  $(du -sh ~/.local/share/docker 2>/dev/null | cut -f1)"

# Purge debug session output older than 7 days
clean-debug:
    #!/usr/bin/env bash
    set -euo pipefail
    echo "Purging debug files older than 7 days..."
    count=0
    freed=0
    find ~/.claude/debug -maxdepth 1 -type f -name "*.txt" -mtime +7 2>/dev/null | while read f; do
        size=$(stat -c%s "$f" 2>/dev/null || echo 0)
        rm -f "$f"
        count=$((count + 1))
        freed=$((freed + size))
    done
    echo "Done."

# Run full waste collection: rotate + debug purge + report
waste-collect: waste-rotate clean-debug
    @echo ""
    @echo "Waste collection complete. Run 'just waste-audit' to verify."

# Nuclear option: clean everything recoverable
clean-everything: clean-all-targets clean-docker clean-debug waste-rotate
    @echo ""
    @echo "Full cleanup complete. Run 'just waste-audit' to see results."

# ── Security Audit ─────────────────────────────────────────────

# Audit workspace dependencies for known vulnerabilities
audit:
    cargo audit

# Audit with JSON output for CI integration
audit-json:
    cargo audit --json

# Audit hooks workspace dependencies
audit-hooks:
    cd ~/.claude/hooks && cargo audit

# Full security audit: workspace + hooks
audit-all: audit audit-hooks

# ── Cron Installation ──────────────────────────────────────────

# Install weekly stale-target cleanup cron (Sundays 3am)
cron-install-cleanup:
    #!/usr/bin/env bash
    set -euo pipefail
    CRON_CMD="0 3 * * 0 cd /home/matthew/nexcore && /home/matthew/.cargo/bin/just clean-stale-targets >> /tmp/nexcore-cleanup.log 2>&1"
    (crontab -l 2>/dev/null | grep -v "nexcore.*clean-stale"; echo "$CRON_CMD") | crontab -
    echo "Installed weekly stale-target cleanup (Sundays 3am)"
    echo "Verify with: crontab -l"

# Remove the weekly cleanup cron
cron-remove-cleanup:
    #!/usr/bin/env bash
    set -euo pipefail
    crontab -l 2>/dev/null | grep -v "nexcore.*clean-stale" | crontab -
    echo "Removed weekly stale-target cleanup"

# Show all NexCore-related cron entries
cron-list:
    @crontab -l 2>/dev/null | grep -E "(nexcore|claude|waste)" || echo "No NexCore cron entries found"

# ── NexCore OS ─────────────────────────────────────────────────

# Build nexcore-init (PID 1 binary)
os-build:
    cargo build --release -p nexcore-init

# Run NexCore OS in virtual mode (default: desktop, 10 ticks)
os-run form_factor="desktop" ticks="10":
    cargo run --release -p nexcore-init -- --virtual --form-factor {{form_factor}} --ticks {{ticks}}

# Test all OS crates (PAL, OS, compositor, shell, init)
os-test:
    cargo test -p nexcore-pal -p nexcore-pal-linux -p nexcore-os -p nexcore-compositor -p nexcore-shell -p nexcore-init

# Local boot test — all 3 form factors
os-boot-test:
    ./boot/local-test.sh

# Docker PID 1 boot test
os-docker-test:
    ./boot/docker-test.sh

# Build Docker image for NexCore OS
os-docker-build:
    docker build -t nexcore-os:test -f boot/Dockerfile .

# Build initramfs for QEMU boot (requires musl target)
os-initramfs form_factor="desktop" ticks="10":
    ./boot/build-initramfs.sh --form-factor {{form_factor}} --ticks {{ticks}}

# Boot NexCore OS in QEMU (requires kernel + initramfs)
os-qemu:
    ./boot/qemu-run.sh

# ── Kellnr Registry ──────────────────────────────────────────

# Start local Kellnr registry
kellnr-up:
    docker compose -f kellnr/docker-compose.yml up -d

# Stop local Kellnr registry
kellnr-down:
    docker compose -f kellnr/docker-compose.yml down

# Show Kellnr status
kellnr-status:
    @curl -s http://localhost:8000/api/v1/crates 2>/dev/null | jq . 2>/dev/null || echo "Kellnr not running"

# ── Workspace Migration ──────────────────────────────────────

# Convert a single crate to standalone (dry run)
convert-crate crate_path:
    cargo run --manifest-path tools/crate-converter/Cargo.toml -- --workspace . --crate-path {{crate_path}} --dry-run

# Convert a single crate to standalone (write)
convert-crate-write crate_path:
    cargo run --manifest-path tools/crate-converter/Cargo.toml -- --workspace . --crate-path {{crate_path}}

# Convert ALL crates to standalone (dry run)
convert-all-dry:
    cargo run --manifest-path tools/crate-converter/Cargo.toml -- --workspace . --all --dry-run

# Show publish order (DAG)
publish-order:
    cargo run --manifest-path tools/dag-publish/Cargo.toml -- --crates-dir crates --dry-run

# Show publish phases (parallelizable)
publish-phases:
    cargo run --manifest-path tools/dag-publish/Cargo.toml -- --crates-dir crates --show-phases

# Publish all crates in DAG order
publish-all:
    cargo run --manifest-path tools/dag-publish/Cargo.toml -- --crates-dir crates

# ── Post-Migration ────────────────────────────────────────────

# Check a standalone crate builds against registry
check-standalone crate_path:
    #!/usr/bin/env bash
    pushd {{crate_path}} > /dev/null && cargo check && popd > /dev/null

# Test a standalone crate against registry
test-standalone crate_path:
    #!/usr/bin/env bash
    pushd {{crate_path}} > /dev/null && cargo test && popd > /dev/null

# Build all standalone crates
build-all-standalone:
    #!/usr/bin/env bash
    set -euo pipefail
    for d in crates/*/; do
        if [ -f "$d/Cargo.toml" ]; then
            echo "Checking $d..."
            (cd "$d" && cargo check) || { echo "FAILED: $d"; exit 1; }
        fi
    done
    echo "All crates checked successfully."

# Test all standalone crates
test-all-standalone:
    #!/usr/bin/env bash
    set -euo pipefail
    failed=0
    for d in crates/*/; do
        if [ -f "$d/Cargo.toml" ]; then
            echo "Testing $d..."
            (cd "$d" && cargo test) || { echo "FAILED: $d"; failed=$((failed + 1)); }
        fi
    done
    echo "Done. $failed failures."
