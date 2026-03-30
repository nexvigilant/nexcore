#!/usr/bin/env bash
set -euo pipefail

# enrich-crate-metadata.sh — Adds missing metadata fields to all crate Cargo.toml files
# Fields added: homepage, readme (via workspace inheritance)
# Fields added per-crate: keywords, categories (based on crate name/description classification)
#
# Usage:
#   ./scripts/enrich-crate-metadata.sh              # Dry run (default)
#   ./scripts/enrich-crate-metadata.sh --apply       # Apply changes
#   ./scripts/enrich-crate-metadata.sh --report      # Summary report only

NEXCORE_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
CRATES_DIR="$NEXCORE_ROOT/crates"
TOOLS_DIR="$NEXCORE_ROOT/tools"
MODE="${1:---dry-run}"

added=0
skipped=0
errors=0

# Classify a crate into keywords and categories based on name and description
classify_crate() {
    local name="$1"
    local desc="$2"
    local keywords=""
    local categories=""

    # Math / STEM (check BEFORE PV — stem-topology, stem-phys, etc.)
    if echo "$name" | grep -qiE '^stem-'; then
        keywords='"mathematics", "science"'
        categories='"science", "algorithms"'
    # Build / Publish tools
    elif echo "$name $desc" | grep -qiE 'dag-publish|crate-converter|build-gate|build-orc|justfile'; then
        keywords='"build-tools", "publishing"'
        categories='"development-tools"'
    # VR platform crates
    elif echo "$name" | grep -qiE '^vr-'; then
        keywords='"platform", "pharmacovigilance"'
        categories='"development-tools"'
    # PV / Drug Safety
    elif echo "$name $desc" | grep -qiE 'vigil|pv-core|faers|signal|causality|harm|drug|adverse|icsr|meddra|safety|naranjo|pharmacov'; then
        keywords='"pharmacovigilance", "drug-safety"'
        categories='"science"'
    # Biology / Medical
    elif echo "$name $desc" | grep -qiE 'cytokine|hormone|immunity|synapse|phenotype|ribosome|transcriptase|spliceosome|fhir|bio'; then
        keywords='"biology", "medical"'
        categories='"science"'
    # Math / STEM (non-stem-prefixed)
    elif echo "$name $desc" | grep -qiE 'math|topology|phys|number-theory|complex|graph|metric'; then
        keywords='"mathematics", "science"'
        categories='"science", "algorithms"'
    # Crypto / Security
    elif echo "$name $desc" | grep -qiE 'hash|codec|vault|crypto|encrypt|sign'; then
        keywords='"cryptography", "security"'
        categories='"cryptography"'
    # Database / Storage
    elif echo "$name $desc" | grep -qiE '\bdb\b|sqlite|storage|persist|dataframe'; then
        keywords='"database", "storage"'
        categories='"database"'
    # MCP / API / Network
    elif echo "$name $desc" | grep -qiE 'mcp|api|http|server|client|transport|proxy|stdio'; then
        keywords='"mcp", "api"'
        categories='"web-programming"'
    # Config / Infrastructure
    elif echo "$name $desc" | grep -qiE 'config|build|deploy|gate|orchestrat'; then
        keywords='"infrastructure", "configuration"'
        categories='"development-tools"'
    # OS / System
    elif echo "$name $desc" | grep -qiE '\bos\b|shell|terminal|compositor|\bpal\b|init\b|audio|render'; then
        keywords='"operating-system", "systems"'
        categories='"os"'
    # Compiler / Language
    elif echo "$name $desc" | grep -qiE 'pvdsl|dtree|prima|compiler|parser|interpret|transpil|grammar'; then
        keywords='"compiler", "language"'
        categories='"compilers"'
    # AI / ML
    elif echo "$name $desc" | grep -qiE '\bai\b|\bml\b|bert|transformer|model|inference|embed'; then
        keywords='"machine-learning", "ai"'
        categories='"science", "algorithms"'
    # Skills / Brain / Knowledge
    elif echo "$name $desc" | grep -qiE 'brain|skill|knowledge|memory|session|artifact|learn'; then
        keywords='"knowledge-management", "automation"'
        categories='"development-tools"'
    # Foundation / Primitives
    elif echo "$name $desc" | grep -qiE 'primitiv|lex|constant|macro|trait|error|derive|\bid\b|uuid|fs\b|chrono|time\b|date'; then
        keywords='"foundation", "primitives"'
        categories='"development-tools"'
    # Catch-all: NexCore ecosystem
    else
        keywords='"nexcore", "pharmacovigilance"'
        categories='"development-tools"'
    fi

    # Always prepend nexcore keyword
    if ! echo "$keywords" | grep -q '"nexcore"'; then
        keywords="\"nexcore\", $keywords"
    fi

    echo "$keywords|$categories"
}

# Process a single Cargo.toml
process_crate() {
    local toml="$1"
    local crate_dir
    crate_dir="$(dirname "$toml")"
    local crate_name
    crate_name="$(basename "$crate_dir")"

    # Extract current description
    local desc
    desc=$(grep '^description' "$toml" | head -1 | sed 's/description *= *"\(.*\)"/\1/' || echo "")

    local changes=""

    # 1. Add homepage.workspace = true if missing
    if ! grep -q '^homepage' "$toml"; then
        changes="$changes +homepage"
        if [ "$MODE" = "--apply" ]; then
            # Insert after repository line (or after description if no repository)
            if grep -q '^repository' "$toml"; then
                sed -i '/^repository/a homepage.workspace = true' "$toml"
            elif grep -q '^description' "$toml"; then
                sed -i '/^description/a homepage.workspace = true' "$toml"
            fi
        fi
    fi

    # 2. Add readme.workspace = true if missing
    if ! grep -q '^readme' "$toml"; then
        # Only add readme if the crate has a README.md
        if [ -f "$crate_dir/README.md" ]; then
            changes="$changes +readme(local)"
            if [ "$MODE" = "--apply" ]; then
                if grep -q '^homepage' "$toml"; then
                    sed -i '/^homepage/a readme = "README.md"' "$toml"
                elif grep -q '^repository' "$toml"; then
                    sed -i '/^repository/a readme = "README.md"' "$toml"
                fi
            fi
        fi
    fi

    # 3. Add keywords if missing
    if ! grep -q '^keywords' "$toml"; then
        local classified
        classified=$(classify_crate "$crate_name" "$desc")
        local kw
        kw=$(echo "$classified" | cut -d'|' -f1)
        local cat
        cat=$(echo "$classified" | cut -d'|' -f2)

        changes="$changes +keywords[$kw]"
        if [ "$MODE" = "--apply" ]; then
            # Insert before [dependencies] or at end of [package]
            if grep -q '^\[dependencies\]' "$toml"; then
                sed -i "/^\[dependencies\]/i keywords = [$kw]" "$toml"
            elif grep -q '^\[features\]' "$toml"; then
                sed -i "/^\[features\]/i keywords = [$kw]" "$toml"
            elif grep -q '^\[lints\]' "$toml"; then
                sed -i "/^\[lints\]/i keywords = [$kw]" "$toml"
            fi
        fi

        # 4. Add categories if missing (same condition as keywords)
        if ! grep -q '^categories' "$toml"; then
            changes="$changes +categories[$cat]"
            if [ "$MODE" = "--apply" ]; then
                if grep -q '^keywords' "$toml"; then
                    sed -i "/^keywords/a categories = [$cat]" "$toml"
                fi
            fi
        fi
    fi

    if [ -n "$changes" ]; then
        echo "  $crate_name:$changes"
        added=$((added + 1))
    else
        skipped=$((skipped + 1))
    fi
}

echo "=== NexCore Crate Metadata Enrichment ==="
echo "Mode: $MODE"
echo "Scanning: $CRATES_DIR and $TOOLS_DIR"
echo

# Process all crates
for toml in "$CRATES_DIR"/*/Cargo.toml "$TOOLS_DIR"/*/Cargo.toml; do
    if [ -f "$toml" ]; then
        process_crate "$toml"
    fi
done

echo
echo "=== Summary ==="
echo "Modified: $added"
echo "Already complete: $skipped"
echo "Errors: $errors"

if [ "$MODE" = "--dry-run" ]; then
    echo
    echo "This was a dry run. Use --apply to make changes."
fi
