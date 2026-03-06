#!/usr/bin/env bash
set -euo pipefail

# Bulk publish all nexcore workspace crates to local Kellnr registry
# Generated from cargo metadata - publish in topological order
#
# Usage:
#   bulk-publish.sh              # Publish all
#   bulk-publish.sh --dry-run    # Preview only
#   bulk-publish.sh --tier N     # Publish only tier N

NEXCORE_ROOT="${NEXCORE_ROOT:-$HOME/Projects/Active/nexcore}"
REGISTRY="nexcore"
TOKEN="${CARGO_REGISTRIES_NEXCORE_TOKEN:-dummy}"
DRY_RUN=false
TARGET_TIER=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --dry-run) DRY_RUN=true; shift ;;
    --tier)    TARGET_TIER="$2"; shift 2 ;;
    *)         echo "Unknown arg: $1"; exit 1 ;;
  esac
done

info()  { echo "  → $*"; }
ok()    { echo "  ✓ $*"; }
fail()  { echo "  ✗ $*" >&2; }

patch_version() {
  local toml="$1" dep_name="$2" dep_version="$3"
  # Add version = "X.Y.Z" to dep lines that have path but no version
  # Match: dep_name = { path = "..." } or dep_name.workspace = true patterns
  # For workspace deps, we need to add version to the workspace definition
  if grep -q "^${dep_name}" "$toml" 2>/dev/null; then
    # Check if version already specified
    if ! grep "^${dep_name}" "$toml" | grep -q "version"; then
      sed -i "s/^\(${dep_name}.*path = \"[^\"]*\"\)/\1, version = \"${dep_version}\"/" "$toml"
    fi
  fi
}

publish_crate() {
  local name="$1" tier="$2"
  local crate_dir
  crate_dir=$(find "$NEXCORE_ROOT/crates" -maxdepth 1 -name "$name" -type d | head -1)
  
  if [[ -z "$crate_dir" ]]; then
    fail "$name: crate directory not found"
    return 1
  fi
  
  local toml="$crate_dir/Cargo.toml"
  
  if $DRY_RUN; then
    info "[DRY-RUN] Would publish $name from $crate_dir"
    return 0
  fi
  
  # Publish with allow-dirty since we may have patched Cargo.toml
  if CARGO_REGISTRIES_NEXCORE_TOKEN="$TOKEN" cargo publish \
      -p "$name" \
      --registry "$REGISTRY" \
      --allow-dirty \
      2>&1 | tail -3; then
    ok "$name published"
  else
    fail "$name publish failed"
    return 1
  fi
}


# ─── Tier 0: 18 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "0" ]]; then
  echo "\n═══ Tier 0 (18 crates) ═══"
  publish_crate "compendious-machine" 0
  publish_crate "integrity-calc" 0
  publish_crate "mcp-relay" 0
  publish_crate "nexcore-anatomy" 0
  publish_crate "nexcore-audio" 0
  publish_crate "nexcore-codec" 0
  publish_crate "nexcore-downloads-scanner" 0
  publish_crate "nexcore-error-derive" 0
  publish_crate "nexcore-fs" 0
  publish_crate "nexcore-hash" 0
  publish_crate "nexcore-id" 0
  publish_crate "nexcore-lex-primitiva" 0
  publish_crate "nexcore-primitive-scanner" 0
  publish_crate "nexcore-pty" 0
  publish_crate "nexcore-softrender" 0
  publish_crate "primitive-innovation" 0
  publish_crate "skill-macros-core" 0
  publish_crate "stem-derive-core" 0
fi

# ─── Tier 1: 31 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "1" ]]; then
  echo "\n═══ Tier 1 (31 crates) ═══"
  publish_crate "borrow-miner" 1
  publish_crate "core-true" 1
  publish_crate "nexcore-caesura" 1
  publish_crate "nexcore-cardiovascular" 1
  publish_crate "nexcore-ccp" 1
  publish_crate "nexcore-constants" 1
  publish_crate "nexcore-declension" 1
  publish_crate "nexcore-domain-primitives" 1
  publish_crate "nexcore-education-machine" 1
  publish_crate "nexcore-error" 1
  publish_crate "nexcore-fhir" 1
  publish_crate "nexcore-firebase" 1
  publish_crate "nexcore-forge-strategy" 1
  publish_crate "nexcore-harm-taxonomy" 1
  publish_crate "nexcore-knowledge" 1
  publish_crate "nexcore-laboratory" 1
  publish_crate "nexcore-lymphatic" 1
  publish_crate "nexcore-pal" 1
  publish_crate "nexcore-pharma-rd" 1
  publish_crate "nexcore-pharmacovigilance" 1
  publish_crate "nexcore-preemptive-pv" 1
  publish_crate "nexcore-respiratory" 1
  publish_crate "nexcore-signal-types" 1
  publish_crate "nexcore-sop-anatomy" 1
  publish_crate "nexcore-state-theory" 1
  publish_crate "nexcore-statemind" 1
  publish_crate "nexcore-topology" 1
  publish_crate "nexcore-urinary" 1
  publish_crate "nucli" 1
  publish_crate "skill-macros" 1
  publish_crate "stem-derive" 1
fi

# ─── Tier 2: 48 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "2" ]]; then
  echo "\n═══ Tier 2 (48 crates) ═══"
  publish_crate "academy-forge" 2
  publish_crate "antitransformer" 2
  publish_crate "claude-knowledge-mcp" 2
  publish_crate "claude-mcp-config" 2
  publish_crate "claude-repl-mcp" 2
  publish_crate "counter-awareness" 2
  publish_crate "crate-converter" 2
  publish_crate "dag-publish" 2
  publish_crate "mcp-stdio-client" 2
  publish_crate "nexcore-aggregate" 2
  publish_crate "nexcore-asm" 2
  publish_crate "nexcore-bicone" 2
  publish_crate "nexcore-ccim" 2
  publish_crate "nexcore-chrono" 2
  publish_crate "nexcore-cognition" 2
  publish_crate "nexcore-compositor" 2
  publish_crate "nexcore-config" 2
  publish_crate "nexcore-cortex" 2
  publish_crate "nexcore-dataframe" 2
  publish_crate "nexcore-docs-mcp" 2
  publish_crate "nexcore-energy" 2
  publish_crate "nexcore-fda-guidance" 2
  publish_crate "nexcore-integrity" 2
  publish_crate "nexcore-jeopardy" 2
  publish_crate "nexcore-model-checker" 2
  publish_crate "nexcore-pal-linux" 2
  publish_crate "nexcore-perplexity" 2
  publish_crate "nexcore-pvos" 2
  publish_crate "nexcore-signal-theory" 2
  publish_crate "nexcore-skill-exec" 2
  publish_crate "nexcore-skill-verify" 2
  publish_crate "nexcore-state-os" 2
  publish_crate "nexcore-stdio-proxy" 2
  publish_crate "nexcore-tov" 2
  publish_crate "nexcore-tov-grounded" 2
  publish_crate "nexcore-tov-proofs" 2
  publish_crate "nexcore-transcriptase" 2
  publish_crate "nexcore-transform" 2
  publish_crate "nexcore-viz" 2
  publish_crate "nexcore-word" 2
  publish_crate "nexcore-word-spectroscopy" 2
  publish_crate "prima" 2
  publish_crate "prima-academy" 2
  publish_crate "prima-chem" 2
  publish_crate "skill-core" 2
  publish_crate "skill-hunter" 2
  publish_crate "skills" 2
  publish_crate "stem-core" 2
fi

# ─── Tier 3: 70 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "3" ]]; then
  echo "\n═══ Tier 3 (70 crates) ═══"
  publish_crate "claude-fs-mcp" 3
  publish_crate "disney-loop" 3
  publish_crate "grounded" 3
  publish_crate "gsheets-mcp" 3
  publish_crate "gvids-mcp" 3
  publish_crate "lessons-mcp" 3
  publish_crate "nexcloud" 3
  publish_crate "nexcore-antibodies" 3
  publish_crate "nexcore-browser" 3
  publish_crate "nexcore-build-gate" 3
  publish_crate "nexcore-circulatory" 3
  publish_crate "nexcore-clearance" 3
  publish_crate "nexcore-compliance" 3
  publish_crate "nexcore-compound-registry" 3
  publish_crate "nexcore-ctvp" 3
  publish_crate "nexcore-cytokine" 3
  publish_crate "nexcore-db" 3
  publish_crate "nexcore-digestive" 3
  publish_crate "nexcore-engram" 3
  publish_crate "nexcore-flywheel" 3
  publish_crate "nexcore-foundry" 3
  publish_crate "nexcore-ghost" 3
  publish_crate "nexcore-homeostasis-primitives" 3
  publish_crate "nexcore-hormone-types" 3
  publish_crate "nexcore-insight" 3
  publish_crate "nexcore-integumentary" 3
  publish_crate "nexcore-knowledge-engine" 3
  publish_crate "nexcore-labs" 3
  publish_crate "nexcore-mcp-hot" 3
  publish_crate "nexcore-measure" 3
  publish_crate "nexcore-molcore" 3
  publish_crate "nexcore-muscular" 3
  publish_crate "nexcore-nervous" 3
  publish_crate "nexcore-network" 3
  publish_crate "nexcore-notebooklm" 3
  publish_crate "nexcore-openfda" 3
  publish_crate "nexcore-oracle" 3
  publish_crate "nexcore-organize" 3
  publish_crate "nexcore-prima" 3
  publish_crate "nexcore-proof-of-meaning" 3
  publish_crate "nexcore-registry" 3
  publish_crate "nexcore-renderer" 3
  publish_crate "nexcore-reproductive" 3
  publish_crate "nexcore-ribosome" 3
  publish_crate "nexcore-sentinel" 3
  publish_crate "nexcore-signal-fence" 3
  publish_crate "nexcore-skeletal" 3
  publish_crate "nexcore-skill-compiler" 3
  publish_crate "nexcore-social-types" 3
  publish_crate "nexcore-spliceosome" 3
  publish_crate "nexcore-synapse" 3
  publish_crate "nexcore-synth" 3
  publish_crate "nexcore-telemetry-core" 3
  publish_crate "nexcore-vault" 3
  publish_crate "nexcore-watch-core" 3
  publish_crate "perplexity-mcp" 3
  publish_crate "prima-codegen" 3
  publish_crate "prima-mcp" 3
  publish_crate "prima-pipeline" 3
  publish_crate "pvos-primitive-expansion" 3
  publish_crate "skill-loader" 3
  publish_crate "skill-primitive-extractor" 3
  publish_crate "skill-transfer-confidence" 3
  publish_crate "skills-mcp" 3
  publish_crate "stem-complex" 3
  publish_crate "stem-math" 3
  publish_crate "stem-number-theory" 3
  publish_crate "transformer-primitives" 3
  publish_crate "vr-core" 3
  publish_crate "wksp-types" 3
fi

# ─── Tier 4: 33 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "4" ]]; then
  echo "\n═══ Tier 4 (33 crates) ═══"
  publish_crate "nexcore-build-orchestrator" 4
  publish_crate "nexcore-combinatorics" 4
  publish_crate "nexcore-compilation-space" 4
  publish_crate "nexcore-dtree" 4
  publish_crate "nexcore-edit-distance" 4
  publish_crate "nexcore-homeostasis-memory" 4
  publish_crate "nexcore-homeostasis-storm" 4
  publish_crate "nexcore-hook-lib" 4
  publish_crate "nexcore-hormones" 4
  publish_crate "nexcore-immunity" 4
  publish_crate "nexcore-metabolite" 4
  publish_crate "nexcore-phenotype" 4
  publish_crate "nexcore-primitives" 4
  publish_crate "nexcore-qsar" 4
  publish_crate "nexcore-reason" 4
  publish_crate "nexcore-retrocasting" 4
  publish_crate "nexcore-social" 4
  publish_crate "nexcore-structural-alerts" 4
  publish_crate "nexcore-terminal" 4
  publish_crate "nexcore-trust" 4
  publish_crate "nexcore-value-mining" 4
  publish_crate "nexcore-watch-app" 4
  publish_crate "nexcore-zeta" 4
  publish_crate "prima-mcp-server" 4
  publish_crate "stem" 4
  publish_crate "stem-bio" 4
  publish_crate "stem-phys" 4
  publish_crate "stem-topology" 4
  publish_crate "vr-billing" 4
  publish_crate "vr-compliance" 4
  publish_crate "vr-marketplace" 4
  publish_crate "vr-platform-ml" 4
  publish_crate "vr-tenant" 4
fi

# ─── Tier 5: 13 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "5" ]]; then
  echo "\n═══ Tier 5 (13 crates) ═══"
  publish_crate "nexcore-algovigilance" 5
  publish_crate "nexcore-brain" 5
  publish_crate "nexcore-chemivigilance" 5
  publish_crate "nexcore-dna" 5
  publish_crate "nexcore-grammar-lab" 5
  publish_crate "nexcore-homeostasis" 5
  publish_crate "nexcore-mesh" 5
  publish_crate "nexcore-rh-proofs" 5
  publish_crate "nexcore-signal-pipeline" 5
  publish_crate "nexcore-skills-engine" 5
  publish_crate "nexcore-stoichiometry" 5
  publish_crate "nexcore-trial" 5
  publish_crate "reddit-mcp" 5
fi

# ─── Tier 6: 2 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "6" ]]; then
  echo "\n═══ Tier 6 (2 crates) ═══"
  publish_crate "bert-burn" 6
  publish_crate "signal" 6
fi

# ─── Tier 7: 1 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "7" ]]; then
  echo "\n═══ Tier 7 (1 crates) ═══"
  publish_crate "nexcore-pv-core" 7
fi

# ─── Tier 8: 4 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "8" ]]; then
  echo "\n═══ Tier 8 (4 crates) ═══"
  publish_crate "nexcore-guardian-engine" 8
  publish_crate "nexcore-os" 8
  publish_crate "nexcore-pvdsl" 8
  publish_crate "nexcore-qbr" 8
fi

# ─── Tier 9: 2 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "9" ]]; then
  echo "\n═══ Tier 9 (2 crates) ═══"
  publish_crate "nexcore-shell" 9
  publish_crate "nexcore-vigilance" 9
fi

# ─── Tier 10: 6 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "10" ]]; then
  echo "\n═══ Tier 10 (6 crates) ═══"
  publish_crate "nexcore-core" 10
  publish_crate "nexcore-faers-etl" 10
  publish_crate "nexcore-init" 10
  publish_crate "nexcore-nvrepl" 10
  publish_crate "nexcore-orchestration" 10
  publish_crate "nexcore-vigil" 10
fi

# ─── Tier 11: 1 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "11" ]]; then
  echo "\n═══ Tier 11 (1 crates) ═══"
  publish_crate "nexcore-pharos" 11
fi

# ─── Tier 12: 1 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "12" ]]; then
  echo "\n═══ Tier 12 (1 crates) ═══"
  publish_crate "nexcore-mcp" 12
fi

# ─── Tier 13: 1 crates ───
if [[ -z "$TARGET_TIER" || "$TARGET_TIER" == "13" ]]; then
  echo "\n═══ Tier 13 (1 crates) ═══"
  publish_crate "nexcore-api" 13
fi

echo "\n✓ Bulk publish complete"
