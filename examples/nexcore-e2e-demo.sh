#!/usr/bin/env bash
# =============================================================================
# NexCore CLI — End-to-End Demo
# =============================================================================
#
# Exercises every `nexcore` subcommand in a single run.
# Creates temp data files, runs all commands, cleans up.
#
# Usage:
#   chmod +x examples/nexcore-e2e-demo.sh
#   cargo build -p nexcore-cli                          # build first
#   ./examples/nexcore-e2e-demo.sh                      # run demo
#   ./examples/nexcore-e2e-demo.sh --section pv         # run one section
#   ./examples/nexcore-e2e-demo.sh --json               # raw JSON output (no banners)
#
# Requirements:
#   - Built nexcore-cli binary (debug or release)
#   - Skills directory at ~/nexcore/skills/
# =============================================================================

set -euo pipefail

# ---------------------------------------------------------------------------
# Configuration
# ---------------------------------------------------------------------------
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
NEXCORE_ROOT="${SCRIPT_DIR}/.."
TMPDIR_DEMO="$(mktemp -d /tmp/nexcore-demo.XXXXXX)"
SECTION_FILTER="${2:-}"
JSON_MODE=false
PASS=0
FAIL=0
SKIP=0

# Find binary (prefer release, fall back to debug, fall back to cargo run)
if [[ -x "${NEXCORE_ROOT}/target/release/nexcore-cli" ]]; then
    NEXCORE="${NEXCORE_ROOT}/target/release/nexcore-cli"
elif [[ -x "${NEXCORE_ROOT}/target/debug/nexcore-cli" ]]; then
    NEXCORE="${NEXCORE_ROOT}/target/debug/nexcore-cli"
else
    NEXCORE="cargo run -p nexcore-cli --quiet --"
fi

# ---------------------------------------------------------------------------
# Parse arguments
# ---------------------------------------------------------------------------
for arg in "$@"; do
    case "$arg" in
        --json) JSON_MODE=true ;;
        --section) ;; # value captured by $2
        --help|-h)
            echo "Usage: $0 [--section <name>] [--json]"
            echo ""
            echo "Sections: version foundation pv vigilance skill hooks security"
            echo "          orchestrator hud sos vigil verify"
            echo ""
            echo "Options:"
            echo "  --section <name>  Run only the named section"
            echo "  --json            Suppress banners, print raw JSON"
            echo "  --help            Show this help"
            exit 0
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Helpers
# ---------------------------------------------------------------------------
section() {
    local name="$1"
    local name_lower="${name,,}"
    local filter_lower="${SECTION_FILTER,,}"
    if [[ -n "$SECTION_FILTER" && "$name_lower" != *"$filter_lower"* ]]; then
        return 1  # skip this section
    fi
    if [[ "$JSON_MODE" == false ]]; then
        echo ""
        echo "=================================================================="
        echo "  $name"
        echo "=================================================================="
        echo ""
    fi
    return 0
}

run_cmd() {
    local description="$1"
    shift
    if [[ "$JSON_MODE" == false ]]; then
        echo "--- $description ---"
        echo "\$ $*"
        echo ""
    fi
    if eval "$@" 2>&1; then
        PASS=$((PASS + 1))
    else
        echo "[FAILED] $description (exit $?)"
        FAIL=$((FAIL + 1))
    fi
    if [[ "$JSON_MODE" == false ]]; then
        echo ""
    fi
}

cleanup() {
    rm -rf "$TMPDIR_DEMO"
    if [[ "$JSON_MODE" == false ]]; then
        echo ""
        echo "=================================================================="
        echo "  SUMMARY"
        echo "=================================================================="
        echo "  Passed:  $PASS"
        echo "  Failed:  $FAIL"
        echo "  Temp:    $TMPDIR_DEMO (cleaned)"
        echo "=================================================================="
    fi
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
# Create test data files
# ---------------------------------------------------------------------------

# YAML file for foundation yaml command
cat > "${TMPDIR_DEMO}/sample.yaml" << 'YAML'
drug:
  name: Aspirin
  class: NSAID
  indications:
    - pain
    - inflammation
    - fever
  dosage:
    unit: mg
    typical: 325
    max_daily: 4000
adverse_events:
  - name: GI bleeding
    frequency: common
    serious: true
  - name: Tinnitus
    frequency: uncommon
    serious: false
YAML

# PV system state for verify/report (all 11 Conservation Laws)
cat > "${TMPDIR_DEMO}/system-state.json" << 'JSON'
{
  "mass_balance": {
    "initial_dose": 100.0,
    "current_amount_in_body": 60.0,
    "cumulative_eliminated": 40.0
  },
  "binding": {
    "association_constant_m_inv": 1e6,
    "temperature_k": 310.15
  },
  "receptor": {
    "total_constant": 1000.0,
    "current_free": 600.0,
    "current_bound": 350.0,
    "current_desensitized": 50.0
  },
  "pathway": {
    "fluxes_in": [5.0, 3.0, 2.0],
    "fluxes_out": [4.0, 3.5, 2.5]
  },
  "enzyme": {
    "k_syn": 0.5,
    "k_deg": 0.1,
    "k_inact": 0.05,
    "inhibitor_conc": 2.0,
    "total_enzyme": 5.0,
    "measured_rate_of_change": 0.2
  },
  "adme": {
    "rates_in": [10.0, 5.0],
    "rates_out": [8.0, 6.0],
    "measured_rate_of_change": 1.0
  },
  "steady_state": {
    "bioavailability": 0.8,
    "dose": 500.0,
    "clearance_l_h": 10.0,
    "dosing_interval_h": 8.0,
    "measured_concentration": 5.0
  },
  "ionization": {
    "pka": 3.5,
    "ph": 7.4,
    "is_acid": true,
    "measured_fraction_unionized": 0.001
  },
  "saturation": {
    "concentration": 50.0,
    "half_saturation": 25.0,
    "measured_fraction": 0.667
  },
  "entropy": {
    "delta_s_system": -15.0,
    "delta_s_surroundings": 20.0
  },
  "genetic": {
    "sequence_before": "ATCGATCG",
    "sequence_after": "ATCGATCG"
  }
}
JSON

# SOS machine spec (Marketing Authorization Process)
cat > "${TMPDIR_DEMO}/map-machine.json" << 'JSON'
{
  "name": "MAP",
  "states": [
    { "name": "drafted",     "kind": "initial"  },
    { "name": "submitted",   "kind": "normal"   },
    { "name": "under_review","kind": "normal"   },
    { "name": "questions",   "kind": "normal"   },
    { "name": "approved",    "kind": "terminal" },
    { "name": "rejected",    "kind": "error"    }
  ],
  "transitions": [
    { "from": "drafted",      "to": "submitted",    "event": "submit"   },
    { "from": "submitted",    "to": "under_review",  "event": "accept"   },
    { "from": "under_review", "to": "approved",      "event": "approve"  },
    { "from": "under_review", "to": "questions",     "event": "query"    },
    { "from": "under_review", "to": "rejected",      "event": "reject"   },
    { "from": "questions",    "to": "under_review",  "event": "respond"  },
    { "from": "rejected",     "to": "drafted",       "event": "revise"   }
  ]
}
JSON

# SOS machine with a validation error (dangling transition)
cat > "${TMPDIR_DEMO}/bad-machine.json" << 'JSON'
{
  "name": "BAD",
  "states": [
    { "name": "start", "kind": "initial" },
    { "name": "end",   "kind": "terminal" }
  ],
  "transitions": [
    { "from": "start",   "to": "end",     "event": "go"    },
    { "from": "start",   "to": "missing", "event": "oops"  }
  ]
}
JSON

# SOS machine for interactive piped demo
cat > "${TMPDIR_DEMO}/linear-machine.json" << 'JSON'
{
  "name": "Pipeline",
  "states": [
    { "name": "init",  "kind": "initial"  },
    { "name": "run",   "kind": "normal"   },
    { "name": "done",  "kind": "terminal" },
    { "name": "error", "kind": "error"    }
  ],
  "transitions": [
    { "from": "init", "to": "run",   "event": "begin"    },
    { "from": "run",  "to": "done",  "event": "complete" },
    { "from": "run",  "to": "error", "event": "fail"     }
  ]
}
JSON

# =====================================================================
#  SECTION: VERSION
# =====================================================================
if section "VERSION"; then
    run_cmd "Show version info" \
        "$NEXCORE version"
fi

# =====================================================================
#  SECTION: FOUNDATION
# =====================================================================
if section "FOUNDATION — Algorithms & Utilities"; then

    run_cmd "Levenshtein distance: kitten -> sitting" \
        "$NEXCORE foundation levenshtein kitten sitting"

    run_cmd "Levenshtein distance: aspirin -> ibuprofen" \
        "$NEXCORE foundation levenshtein aspirin ibuprofen"

    run_cmd "SHA-256 hash" \
        "$NEXCORE foundation sha256 'pharmacovigilance signal detection'"

    run_cmd "Parse YAML to JSON" \
        "$NEXCORE foundation yaml '${TMPDIR_DEMO}/sample.yaml'"
fi

# =====================================================================
#  SECTION: PV — Pharmacovigilance Signal Detection
# =====================================================================
if section "PV — Pharmacovigilance Signal Detection"; then

    run_cmd "Full signal detection: Aspirin + GI Bleed (a=15,b=100,c=20,d=10000)" \
        "$NEXCORE pv signal --drug Aspirin --event 'GI Bleed' -a 15 -b 100 -c 20 -d 10000"

    run_cmd "PRR only (strong signal: a=50,b=200,c=10,d=15000)" \
        "$NEXCORE pv prr -a 50 -b 200 -c 10 -d 15000"

    run_cmd "ROR only (weak signal: a=3,b=500,c=100,d=20000)" \
        "$NEXCORE pv ror -a 3 -b 500 -c 100 -d 20000"

    run_cmd "Naranjo causality (probable ADR)" \
        "$NEXCORE pv naranjo --temporal=1 --dechallenge=1 --rechallenge=0 --alternatives=-1 --previous=1"

    run_cmd "Naranjo causality (unlikely ADR)" \
        "$NEXCORE pv naranjo --temporal=-1 --dechallenge=0 --rechallenge=-1 --alternatives=1 --previous=0"

    run_cmd "Verify system state against 11 Conservation Laws" \
        "$NEXCORE pv verify '${TMPDIR_DEMO}/system-state.json'"

    run_cmd "Generate regulatory safety report (EMA)" \
        "$NEXCORE pv report '${TMPDIR_DEMO}/system-state.json' --regulator EMA"

    run_cmd "Generate regulatory safety report (FDA)" \
        "$NEXCORE pv report '${TMPDIR_DEMO}/system-state.json' --regulator FDA"
fi

# =====================================================================
#  SECTION: VIGILANCE — Theory of Vigilance
# =====================================================================
if section "VIGILANCE — Theory of Vigilance / Guardian-AV"; then

    run_cmd "Safety margin calculation (strong signal)" \
        "$NEXCORE vigilance safety-margin --prr 3.5 --ror-lower 2.1 --ic025 1.2 --eb05 2.5 -n 25"

    run_cmd "Safety margin calculation (borderline signal)" \
        "$NEXCORE vigilance safety-margin --prr 1.8 --ror-lower 0.9 --ic025 -0.1 --eb05 1.3 -n 4"

    run_cmd "Risk score: Vioxx + MI (high risk)" \
        "$NEXCORE vigilance risk --drug Vioxx --event 'Myocardial Infarction' --prr 4.2 --ror-lower 3.1 --ic025 2.0 --eb05 3.5 -n 150"

    run_cmd "Risk score: Metformin + Lactic Acidosis (low case count)" \
        "$NEXCORE vigilance risk --drug Metformin --event 'Lactic Acidosis' --prr 1.5 --ror-lower 0.8 --ic025 0.2 --eb05 1.1 -n 3"
fi

# =====================================================================
#  SECTION: SKILL — Skill Management
# =====================================================================
if section "SKILL — Skill Management"; then

    run_cmd "Scan skills directory" \
        "$NEXCORE skill scan '${NEXCORE_ROOT}/skills'"

    run_cmd "List skill names (JSON)" \
        "$NEXCORE skill list '${NEXCORE_ROOT}/skills'"

    # Validate a known skill
    SAMPLE_SKILL=$(find "${NEXCORE_ROOT}/skills" -name "SKILL.md" -maxdepth 2 | head -1 || true)
    if [[ -n "$SAMPLE_SKILL" ]]; then
        SKILL_DIR=$(dirname "$SAMPLE_SKILL")
        run_cmd "Validate skill: $(basename "$SKILL_DIR")" \
            "$NEXCORE skill validate '${SKILL_DIR}'"
    else
        echo "[SKIP] No SKILL.md found for validation"
        SKIP=$((SKIP + 1))
    fi
fi

# =====================================================================
#  SECTION: HOOKS — Claude Code Hook Management
# =====================================================================
if section "HOOKS — Claude Code Hook Management"; then

    run_cmd "Show session state" \
        "$NEXCORE hooks state"

    run_cmd "List available hooks" \
        "$NEXCORE hooks list"

    run_cmd "Validate hook configuration" \
        "$NEXCORE hooks validate"
fi

# =====================================================================
#  SECTION: SECURITY — Security Scanning
# =====================================================================
if section "SECURITY — Security Scanning"; then

    # Scan a small known directory
    run_cmd "Security scan (nexcore-cli, JSON output)" \
        "$NEXCORE security scan '${NEXCORE_ROOT}/nexcore-cli' --json --min-severity low"

    run_cmd "Security scan (human-readable, medium+ severity)" \
        "$NEXCORE security scan '${NEXCORE_ROOT}/nexcore-cli' --min-severity medium"
fi

# =====================================================================
#  SECTION: ORCHESTRATOR — Agent Orchestration
# =====================================================================
if section "ORCHESTRATOR — Agent Orchestration & Skill Chaining"; then

    run_cmd "Orchestrate a request" \
        "$NEXCORE orchestrator run 'Analyze signal for drug X' --metrics-dir '${TMPDIR_DEMO}/metrics'"

    run_cmd "Trace execution (with failed node)" \
        "$NEXCORE orchestrator trace latest --failed-node analyze --metrics-dir '${TMPDIR_DEMO}/metrics'"

    run_cmd "Show orchestration stats" \
        "$NEXCORE orchestrator stats --metrics-dir '${TMPDIR_DEMO}/metrics'"
fi

# =====================================================================
#  SECTION: HUD — Head-Up Display Governance
# =====================================================================
if section "HUD — Head-Up Display (24 CAP Acts)"; then

    run_cmd "List implemented HUD capabilities" \
        "$NEXCORE hud list"

    echo ""
    echo "--- CAP-014: Public Health Act ---"
    echo ""

    run_cmd "Validate signal efficacy" \
        "$NEXCORE hud public-health validate -s SIG-001 -a 0.92"

    run_cmd "Measure public health impact" \
        "$NEXCORE hud public-health measure -t 100 -v 87"

    echo ""
    echo "--- CAP-018: Treasury Act ---"
    echo ""

    run_cmd "Convert signal asymmetry to resources" \
        "$NEXCORE hud treasury convert -s SIG-001 -a 0.75 -o 0.3"

    run_cmd "Audit treasury status" \
        "$NEXCORE hud treasury audit -c 10000 -m 4096"

    echo ""
    echo "--- CAP-025: Small Business Act ---"
    echo ""

    run_cmd "Allocate agent for task" \
        "$NEXCORE hud small-business allocate -t 'Analyze adverse event reports for drug X'"

    run_cmd "List registered agent types" \
        "$NEXCORE hud small-business list-agents"

    echo ""
    echo "--- CAP-027: Federal Reserve Act ---"
    echo ""

    run_cmd "Get budget report" \
        "$NEXCORE hud federal-reserve report"

    run_cmd "Record token usage (standard tier)" \
        "$NEXCORE hud federal-reserve record -s demo-session-1 -i 5000 -o 2000 -t standard"

    run_cmd "Record token usage (premium tier)" \
        "$NEXCORE hud federal-reserve record -s demo-session-2 -i 15000 -o 8000 -t premium"

    run_cmd "Recommend model tier based on budget" \
        "$NEXCORE hud federal-reserve recommend"

    echo ""
    echo "--- CAP-029: Communications Act ---"
    echo ""

    run_cmd "Recommend protocol (guaranteed + low latency)" \
        "$NEXCORE hud communications recommend -g -l"

    run_cmd "Recommend protocol (broadcast, no guarantee)" \
        "$NEXCORE hud communications recommend -b"

    run_cmd "Get protocol for tool_call" \
        "$NEXCORE hud communications get-protocol -t tool_call"

    run_cmd "Get protocol for notification" \
        "$NEXCORE hud communications get-protocol -t notification"

    echo ""
    echo "--- CAP-030: Exploration Act ---"
    echo ""

    run_cmd "Launch exploration mission" \
        "$NEXCORE hud exploration launch -i MISSION-001 -t '${NEXCORE_ROOT}/crates' -o 'Map crate dependency structure' -s thorough"

    run_cmd "Record a discovery" \
        "$NEXCORE hud exploration record-discovery -i DISC-001 -f 'Found 90+ crate directories' -l '${NEXCORE_ROOT}/crates/' -s 0.85"

    run_cmd "Get exploration frontier" \
        "$NEXCORE hud exploration frontier"
fi

# =====================================================================
#  SECTION: SOS — State Operating System
# =====================================================================
if section "SOS — State Operating System"; then

    run_cmd "Generate new machine spec (stdout)" \
        "$NEXCORE sos new DemoMachine"

    run_cmd "Generate new machine spec (file)" \
        "$NEXCORE sos new ReviewProcess -o '${TMPDIR_DEMO}/review.json'"

    run_cmd "Validate good machine (MAP)" \
        "$NEXCORE sos validate '${TMPDIR_DEMO}/map-machine.json'"

    run_cmd "Validate bad machine (dangling transition)" \
        "$NEXCORE sos validate '${TMPDIR_DEMO}/bad-machine.json'"

    run_cmd "Run machine (piped: begin -> complete)" \
        "printf 'begin\ncomplete\n' | $NEXCORE sos run '${TMPDIR_DEMO}/linear-machine.json'"

    run_cmd "Run machine (piped: begin -> fail, hits error terminal)" \
        "printf 'begin\nfail\n' | $NEXCORE sos run '${TMPDIR_DEMO}/linear-machine.json'"

    run_cmd "Run machine (piped: invalid event, then valid)" \
        "printf 'oops\nbegin\ncomplete\n' | $NEXCORE sos run '${TMPDIR_DEMO}/linear-machine.json'"

    run_cmd "Machine status (placeholder)" \
        "$NEXCORE sos status 42"
fi

# =====================================================================
#  SECTION: VIGIL — Vigilance Daemon
# =====================================================================
if section "VIGIL — Persistent Boundary-Watching Daemon"; then

    run_cmd "Vigil status (cold — no WAL yet)" \
        "$NEXCORE vigil status"

    run_cmd "Vigil start (one-shot cycle)" \
        "$NEXCORE vigil start"

    run_cmd "Vigil status (warm — after start)" \
        "$NEXCORE vigil status"

    run_cmd "Query vigil ledger (last 10 entries)" \
        "$NEXCORE vigil ledger --last 10"

    run_cmd "Query vigil ledger (violations only)" \
        "$NEXCORE vigil ledger --entry-type violation"

    run_cmd "Verify ledger hash chain integrity" \
        "$NEXCORE vigil verify"
fi

# =====================================================================
#  SECTION: VERIFY — Diamond Compliance
# =====================================================================
if section "VERIFY — Diamond Compliance Validation"; then

    SAMPLE_SKILL=$(find "${NEXCORE_ROOT}/skills" -name "SKILL.md" -maxdepth 2 | head -1 || true)
    if [[ -n "$SAMPLE_SKILL" ]]; then
        SKILL_DIR=$(dirname "$SAMPLE_SKILL")
        run_cmd "Diamond compliance: $(basename "$SKILL_DIR")" \
            "$NEXCORE verify '${SKILL_DIR}'"
    else
        echo "[SKIP] No SKILL.md found for Diamond verification"
        SKIP=$((SKIP + 1))
    fi
fi

# =====================================================================
#  COMBINED SCENARIO: Drug Signal Investigation Workflow
# =====================================================================
if section "SCENARIO — Full Drug Signal Investigation"; then

    echo "Scenario: Investigating a potential safety signal for Drug X + Hepatotoxicity"
    echo ""

    run_cmd "Step 1: Detect signal from contingency table" \
        "$NEXCORE pv signal --drug 'Drug-X' --event 'Hepatotoxicity' -a 25 -b 300 -c 15 -d 50000"

    run_cmd "Step 2: Calculate safety margin" \
        "$NEXCORE vigilance safety-margin --prr 3.8 --ror-lower 2.5 --ic025 1.5 --eb05 2.8 -n 25"

    run_cmd "Step 3: Assess risk level" \
        "$NEXCORE vigilance risk --drug 'Drug-X' --event 'Hepatotoxicity' --prr 3.8 --ror-lower 2.5 --ic025 1.5 --eb05 2.8 -n 25"

    run_cmd "Step 4: Naranjo causality for index case" \
        "$NEXCORE pv naranjo --temporal=1 --dechallenge=1 --rechallenge=1 --alternatives=-1 --previous=1"

    run_cmd "Step 5: Verify system state conservation" \
        "$NEXCORE pv verify '${TMPDIR_DEMO}/system-state.json'"

    run_cmd "Step 6: Generate regulatory report" \
        "$NEXCORE pv report '${TMPDIR_DEMO}/system-state.json' --regulator FDA"

    run_cmd "Step 7: Security scan on analysis codebase" \
        "$NEXCORE security scan '${NEXCORE_ROOT}/nexcore-cli' --json --min-severity high"

    echo ""
    echo "Investigation complete. All artifacts generated."
fi
