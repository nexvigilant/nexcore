#!/usr/bin/env bash
set -euo pipefail

###############################################################################
# qa-sentinel.sh — QA Sentinel Monitoring for NexVigilant WebMCP Hub Presence
#
# Monitors 6 threat signals at the marketplace boundary:
#   1. License circumvention (corporate domain leakage past Firebase auth)
#   2. Config integrity (local checksum drift — ungated modifications)
#   3. Marketplace clone detection (IP theft on WebMCP Hub)
#   4. Supply chain integrity (TLS, remote tampering, dependency drift)
#   5. Version drift (published vs local capability gap)
#   6. Reputation monitoring (vote/comment deltas)
#
# Usage:
#   qa-sentinel.sh                       # Run all signals, human-readable + JSON
#   qa-sentinel.sh --json                # Machine-readable JSON only
#   qa-sentinel.sh --signal 3            # Run only signal 3
#   qa-sentinel.sh --signal 1 --json     # Single signal, JSON only
#   qa-sentinel.sh --update-checksums    # Reset checksum baseline
#   qa-sentinel.sh --dry-run             # Validate config, no external calls
#
# Exit codes: 0=GREEN, 1=AMBER, 2=RED
#
# Dependencies: bash 4+, curl, jq, sha256sum, openssl (optional for TLS)
###############################################################################

# ---------------------------------------------------------------------------
# Constants
# ---------------------------------------------------------------------------

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly NEXCORE_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"
readonly WEBMCP_CONFIGS="${NEXCORE_ROOT}/webmcp-configs"
readonly CHECKSUMS_FILE="${WEBMCP_CONFIGS}/.checksums"
readonly ALLOWED_DOMAINS="${NEXCORE_ROOT}/licenses/ALLOWED-DOMAINS.md"
readonly STATE_FILE="${HOME}/.claude/data/qa-sentinel-state.json"
readonly LOG_FILE="${HOME}/.claude/logs/qa-sentinel.log"
readonly LOCK_FILE="/tmp/qa-sentinel.lock"
readonly FIREBASE_PROJECT="nexvigilant-digital-clubhouse"
readonly WEBMCP_HUB_BASE="https://www.webmcp-hub.com"
readonly CURL_TIMEOUT=15

# Our known published config identifiers (update as configs are added)
readonly -a OUR_CONFIG_NAMES=(
    "nexvigilant-signal-detection"
    "nexvigilant-causality-assessment"
    "nexvigilant-case-management"
    "nexvigilant-faers-pipeline"
    "nexvigilant-benefit-risk"
    "nexvigilant-regulatory-intelligence"
    "nexvigilant-safety-reporting"
    "nexvigilant-aggregate-analysis"
    "nexvigilant-literature-monitoring"
    "nexvigilant-risk-management"
)

# PV keywords to monitor for clone detection
readonly -a PV_KEYWORDS=(
    "pharmacovigilance"
    "signal+detection"
    "adverse+event"
    "FAERS"
    "causality"
    "naranjo"
    "PRR"
    "drug+safety"
)

# Severity constants
readonly SEV_GREEN="GREEN"
readonly SEV_AMBER="AMBER"
readonly SEV_RED="RED"

# ---------------------------------------------------------------------------
# Globals (mutable)
# ---------------------------------------------------------------------------

JSON_ONLY=false
SINGLE_SIGNAL=""
UPDATE_CHECKSUMS=false
DRY_RUN=false
WORST_SEVERITY=0  # 0=GREEN, 1=AMBER, 2=RED

# Temp files for cleanup
declare -a TEMP_FILES=()

# ---------------------------------------------------------------------------
# Argument parsing
# ---------------------------------------------------------------------------

while [[ $# -gt 0 ]]; do
    case "$1" in
        --json)
            JSON_ONLY=true
            shift
            ;;
        --signal)
            if [[ $# -lt 2 ]]; then
                echo "Error: --signal requires a number (1-6)" >&2
                exit 1
            fi
            SINGLE_SIGNAL="$2"
            if ! [[ "${SINGLE_SIGNAL}" =~ ^[1-6]$ ]]; then
                echo "Error: --signal must be 1-6, got '${SINGLE_SIGNAL}'" >&2
                exit 1
            fi
            shift 2
            ;;
        --update-checksums)
            UPDATE_CHECKSUMS=true
            shift
            ;;
        --dry-run)
            DRY_RUN=true
            shift
            ;;
        --help|-h)
            echo "Usage: qa-sentinel.sh [--json] [--signal N] [--update-checksums] [--dry-run]"
            echo ""
            echo "  --json              Machine-readable JSON output only"
            echo "  --signal N          Run only signal N (1-6)"
            echo "  --update-checksums  Reset config checksum baseline"
            echo "  --dry-run           Validate config without external calls"
            echo ""
            echo "Signals:"
            echo "  1  License Circumvention Detection"
            echo "  2  Config Integrity Verification"
            echo "  3  Marketplace Clone Detection"
            echo "  4  Supply Chain Integrity"
            echo "  5  Version Drift Detection"
            echo "  6  Reputation Monitoring"
            echo ""
            echo "Exit codes: 0=GREEN, 1=AMBER, 2=RED"
            exit 0
            ;;
        *)
            echo "Unknown option: $1 (try --help)" >&2
            exit 1
            ;;
    esac
done

# ---------------------------------------------------------------------------
# Cleanup
# ---------------------------------------------------------------------------

cleanup() {
    local f
    for f in "${TEMP_FILES[@]}"; do
        rm -f "${f}" 2>/dev/null || true
    done
    rm -f "${LOCK_FILE}" 2>/dev/null || true
}
trap cleanup EXIT

# ---------------------------------------------------------------------------
# Locking — prevent concurrent runs
# ---------------------------------------------------------------------------

acquire_lock() {
    if [[ -f "${LOCK_FILE}" ]]; then
        local lock_pid
        lock_pid=$(cat "${LOCK_FILE}" 2>/dev/null || echo "")
        if [[ -n "${lock_pid}" ]] && kill -0 "${lock_pid}" 2>/dev/null; then
            echo "Another qa-sentinel instance is running (PID ${lock_pid})" >&2
            exit 1
        fi
        # Stale lock — remove it
        rm -f "${LOCK_FILE}"
    fi
    echo $$ > "${LOCK_FILE}"
}

acquire_lock

# ---------------------------------------------------------------------------
# Logging
# ---------------------------------------------------------------------------

mkdir -p "$(dirname "${LOG_FILE}")"
mkdir -p "$(dirname "${STATE_FILE}")"

log() {
    local level="$1"
    shift
    local ts
    ts="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"
    echo "${ts} [${level}] $*" >> "${LOG_FILE}"
    if [[ "${JSON_ONLY}" == "false" ]]; then
        echo "[${level}] $*" >&2
    fi
}

# ---------------------------------------------------------------------------
# Utility functions
# ---------------------------------------------------------------------------

severity_to_int() {
    case "$1" in
        GREEN) echo 0 ;;
        AMBER) echo 1 ;;
        RED)   echo 2 ;;
        *)     echo 0 ;;
    esac
}

update_worst() {
    local sev_int
    sev_int=$(severity_to_int "$1")
    if [[ ${sev_int} -gt ${WORST_SEVERITY} ]]; then
        WORST_SEVERITY=${sev_int}
    fi
}

make_temp() {
    local tmp
    tmp=$(mktemp)
    TEMP_FILES+=("${tmp}")
    echo "${tmp}"
}

# Safe JSON string escaping for embedding in heredocs
json_escape() {
    local s="$1"
    s="${s//\\/\\\\}"
    s="${s//\"/\\\"}"
    s="${s//$'\n'/\\n}"
    s="${s//$'\r'/}"
    s="${s//$'\t'/\\t}"
    echo "${s}"
}

ensure_state_file() {
    if [[ ! -f "${STATE_FILE}" ]] || [[ ! -s "${STATE_FILE}" ]]; then
        cat > "${STATE_FILE}" << 'INIT'
{
  "last_run": null,
  "config_checksums": {},
  "marketplace_votes": {},
  "known_clones": [],
  "flagged_accounts": [],
  "version_drift": { "published": 0, "local": 0 },
  "mcp_remote_version": null,
  "alert_history": []
}
INIT
    fi
    # Validate it is parseable JSON
    if ! jq empty "${STATE_FILE}" 2>/dev/null; then
        log "WARN" "State file corrupt — reinitializing"
        cat > "${STATE_FILE}" << 'INIT'
{
  "last_run": null,
  "config_checksums": {},
  "marketplace_votes": {},
  "known_clones": [],
  "flagged_accounts": [],
  "version_drift": { "published": 0, "local": 0 },
  "mcp_remote_version": null,
  "alert_history": []
}
INIT
    fi
}

# Atomic state write — write to temp then move to prevent corruption
state_write() {
    local content="$1"
    local tmp
    tmp=$(make_temp)
    echo "${content}" | jq '.' > "${tmp}" 2>/dev/null || {
        log "ERROR" "state_write: invalid JSON — aborting state update"
        return 1
    }
    cp "${tmp}" "${STATE_FILE}"
}

# Read the full state, apply a jq update, write back atomically
state_update() {
    local jq_filter="$1"
    shift
    local updated
    updated=$(jq "${jq_filter}" "$@" "${STATE_FILE}" 2>/dev/null) || {
        log "ERROR" "state_update: jq filter failed"
        return 1
    }
    state_write "${updated}"
}

# Extract blocked domains from ALLOWED-DOMAINS.md Section 2
# Returns one domain per line, lowercase, deduplicated
extract_blocked_domains() {
    if [[ ! -f "${ALLOWED_DOMAINS}" ]]; then
        log "WARN" "ALLOWED-DOMAINS.md not found at ${ALLOWED_DOMAINS}"
        return
    fi
    local in_section2=false
    while IFS= read -r line; do
        if [[ "${line}" == *"## Section 2"* ]]; then
            in_section2=true
            continue
        fi
        if [[ "${in_section2}" == "true" ]] && [[ "${line}" == *"## Section 3"* ]]; then
            break
        fi
        if [[ "${in_section2}" == "true" ]]; then
            # Extract domain patterns from markdown table cells
            # Match word.word patterns (domain-like), excluding numbers-first
            echo "${line}" | grep -oE '[a-zA-Z][a-zA-Z0-9-]*\.[a-zA-Z][a-zA-Z0-9.-]+' | \
                grep -v '^\.' | \
                tr '[:upper:]' '[:lower:]' || true
        fi
    done < "${ALLOWED_DOMAINS}"
}

# ---------------------------------------------------------------------------
# Signal 1: License Circumvention Detection
# ---------------------------------------------------------------------------

signal_1_license_circumvention() {
    log "INFO" "Signal 1: License Circumvention Detection"

    local severity="${SEV_GREEN}"
    local flagged_json="[]"
    local detail=""
    local checked_count=0
    local flagged_count=0

    # Build blocked domain set
    local -a blocked_domains
    mapfile -t blocked_domains < <(extract_blocked_domains | sort -u)
    local blocked_count=${#blocked_domains[@]}

    if [[ ${blocked_count} -eq 0 ]]; then
        log "WARN" "Signal 1: Could not extract blocked domains from policy"
        severity="${SEV_AMBER}"
        detail="Unable to parse blocked domains from ALLOWED-DOMAINS.md"
        update_worst "${severity}"
        jq -n \
            --arg sev "${severity}" \
            --arg detail "${detail}" \
            --argjson checked "${checked_count}" \
            --argjson flagged_n "${flagged_count}" \
            --argjson blocked_n "${blocked_count}" \
            --argjson flagged_list "${flagged_json}" \
            '{
                signal: 1,
                name: "License Circumvention Detection",
                severity: $sev,
                detail: $detail,
                checked_accounts: $checked,
                flagged_count: $flagged_n,
                blocked_domains_in_policy: $blocked_n,
                flagged_accounts: $flagged_list
            }'
        return
    fi

    if [[ "${DRY_RUN}" == "true" ]]; then
        detail="DRY RUN: ${blocked_count} blocked domains parsed, skipping Firebase query"
        update_worst "${severity}"
        jq -n \
            --arg sev "${severity}" \
            --arg detail "${detail}" \
            --argjson blocked_n "${blocked_count}" \
            '{
                signal: 1,
                name: "License Circumvention Detection",
                severity: $sev,
                detail: $detail,
                checked_accounts: 0,
                flagged_count: 0,
                blocked_domains_in_policy: $blocked_n,
                flagged_accounts: []
            }'
        return
    fi

    # Attempt to query Firebase Auth for recent users
    if command -v firebase >/dev/null 2>&1; then
        local tmp_users
        tmp_users=$(make_temp)

        if firebase auth:export "${tmp_users}" --format=json \
            --project="${FIREBASE_PROJECT}" 2>/dev/null; then

            local user_emails
            user_emails=$(jq -r '.users[]?.email // empty' "${tmp_users}" 2>/dev/null || echo "")

            while IFS= read -r email; do
                [[ -z "${email}" ]] && continue
                checked_count=$((checked_count + 1))
                local domain
                domain=$(echo "${email}" | sed 's/.*@//' | tr '[:upper:]' '[:lower:]')

                local blocked
                for blocked in "${blocked_domains[@]}"; do
                    if [[ "${domain}" == "${blocked}" ]] || [[ "${domain}" == *".${blocked}" ]]; then
                        flagged_count=$((flagged_count + 1))
                        flagged_json=$(echo "${flagged_json}" | jq \
                            --arg email "${email}" \
                            --arg domain "${domain}" \
                            --arg matched "${blocked}" \
                            '. + [{"email": $email, "domain": $domain, "matched_rule": $matched}]')
                        log "WARN" "Signal 1: Flagged corporate signup: ${email} (matches ${blocked})"
                        break
                    fi
                done
            done <<< "${user_emails}"

            if [[ ${flagged_count} -gt 0 ]]; then
                severity="${SEV_RED}"
                detail="${flagged_count} corporate domain(s) found in ${checked_count} accounts"
            else
                detail="Checked ${checked_count} accounts, 0 corporate domains found (${blocked_count} in blocklist)"
            fi

            # Persist flagged accounts in state
            state_update --argjson accts "${flagged_json}" '.flagged_accounts = $accts'
        else
            log "WARN" "Signal 1: Firebase auth export failed — check credentials"
            severity="${SEV_AMBER}"
            detail="Firebase auth export failed; manual check required"
        fi
    else
        log "INFO" "Signal 1: Firebase CLI not available — skipping live auth check"
        severity="${SEV_AMBER}"
        detail="Firebase CLI not installed; cannot query auth signups"
    fi

    update_worst "${severity}"

    jq -n \
        --arg sev "${severity}" \
        --arg detail "${detail}" \
        --argjson checked "${checked_count}" \
        --argjson flagged_n "${flagged_count}" \
        --argjson blocked_n "${blocked_count}" \
        --argjson flagged_list "${flagged_json}" \
        '{
            signal: 1,
            name: "License Circumvention Detection",
            severity: $sev,
            detail: $detail,
            checked_accounts: $checked,
            flagged_count: $flagged_n,
            blocked_domains_in_policy: $blocked_n,
            flagged_accounts: $flagged_list
        }'
}

# ---------------------------------------------------------------------------
# Signal 2: Config Integrity Verification
# ---------------------------------------------------------------------------

signal_2_config_integrity() {
    log "INFO" "Signal 2: Config Integrity Verification"

    local severity="${SEV_GREEN}"
    local detail=""
    local modified_files="[]"
    local missing_files="[]"
    local new_files="[]"

    if [[ ! -d "${WEBMCP_CONFIGS}" ]]; then
        severity="${SEV_AMBER}"
        detail="Config directory ${WEBMCP_CONFIGS} does not exist"
        update_worst "${severity}"
        jq -n \
            --arg sev "${severity}" \
            --arg detail "${detail}" \
            '{
                signal: 2,
                name: "Config Integrity Verification",
                severity: $sev,
                detail: $detail,
                modified: [],
                missing: [],
                new: []
            }'
        return
    fi

    # Compute current checksums for all config files
    local current_checksums="{}"
    while IFS= read -r config_file; do
        [[ -z "${config_file}" ]] && continue
        local rel_path="${config_file#"${WEBMCP_CONFIGS}/"}"
        local checksum
        checksum=$(sha256sum "${config_file}" | cut -d' ' -f1)
        current_checksums=$(echo "${current_checksums}" | jq \
            --arg file "${rel_path}" \
            --arg hash "${checksum}" \
            '. + {($file): $hash}')
    done < <(find "${WEBMCP_CONFIGS}" -type f \( -name '*.json' -o -name '*.yaml' -o -name '*.yml' -o -name '*.toml' \) ! -name '.checksums' 2>/dev/null | sort)

    # --update-checksums: force baseline reset
    if [[ "${UPDATE_CHECKSUMS}" == "true" ]]; then
        echo "${current_checksums}" | jq '.' > "${CHECKSUMS_FILE}"
        local file_count
        file_count=$(echo "${current_checksums}" | jq 'keys | length')
        detail="Checksum baseline reset (${file_count} files)"
        log "INFO" "Signal 2: ${detail}"
        update_worst "${severity}"
        jq -n \
            --arg sev "${severity}" \
            --arg detail "${detail}" \
            '{
                signal: 2,
                name: "Config Integrity Verification",
                severity: $sev,
                detail: $detail,
                modified: [],
                missing: [],
                new: []
            }'
        return
    fi

    if [[ ! -f "${CHECKSUMS_FILE}" ]]; then
        # First run — establish baseline
        echo "${current_checksums}" | jq '.' > "${CHECKSUMS_FILE}"
        local file_count
        file_count=$(echo "${current_checksums}" | jq 'keys | length')
        detail="Baseline checksums established — ${file_count} files (first run)"
        log "INFO" "Signal 2: ${detail}"
    else
        # Compare against stored checksums
        local stored_checksums
        stored_checksums=$(cat "${CHECKSUMS_FILE}")
        local diff_count=0
        local missing_count=0
        local new_count=0

        # Check for modified or missing files (present in stored, compare to current)
        local key
        for key in $(echo "${stored_checksums}" | jq -r 'keys[]' 2>/dev/null); do
            local stored_hash
            stored_hash=$(echo "${stored_checksums}" | jq -r --arg k "${key}" '.[$k]')
            local current_hash
            current_hash=$(echo "${current_checksums}" | jq -r --arg k "${key}" '.[$k] // "MISSING"')

            if [[ "${current_hash}" == "MISSING" ]]; then
                missing_count=$((missing_count + 1))
                missing_files=$(echo "${missing_files}" | jq --arg f "${key}" '. + [$f]')
                log "WARN" "Signal 2: Config file missing: ${key}"
            elif [[ "${current_hash}" != "${stored_hash}" ]]; then
                diff_count=$((diff_count + 1))
                modified_files=$(echo "${modified_files}" | jq \
                    --arg f "${key}" \
                    --arg old "${stored_hash}" \
                    --arg new_h "${current_hash}" \
                    '. + [{"file": $f, "expected": $old, "actual": $new_h}]')
                log "WARN" "Signal 2: Config modified without gate: ${key}"
            fi
        done

        # Check for new files not in baseline
        for key in $(echo "${current_checksums}" | jq -r 'keys[]' 2>/dev/null); do
            local in_stored
            in_stored=$(echo "${stored_checksums}" | jq -r --arg k "${key}" '.[$k] // "NEW"')
            if [[ "${in_stored}" == "NEW" ]]; then
                new_count=$((new_count + 1))
                new_files=$(echo "${new_files}" | jq --arg f "${key}" '. + [$f]')
            fi
        done

        if [[ ${diff_count} -gt 0 ]] || [[ ${missing_count} -gt 0 ]]; then
            severity="${SEV_RED}"
            detail="FAIL: ${diff_count} modified, ${missing_count} missing, ${new_count} new configs"
        elif [[ ${new_count} -gt 0 ]]; then
            severity="${SEV_AMBER}"
            detail="New configs detected (${new_count}) — run with --update-checksums to accept"
        else
            detail="PASS: All configs match stored checksums"
        fi

        # Persist current checksums as new baseline
        echo "${current_checksums}" | jq '.' > "${CHECKSUMS_FILE}"
    fi

    update_worst "${severity}"

    jq -n \
        --arg sev "${severity}" \
        --arg detail "${detail}" \
        --argjson modified "${modified_files}" \
        --argjson missing "${missing_files}" \
        --argjson new_f "${new_files}" \
        '{
            signal: 2,
            name: "Config Integrity Verification",
            severity: $sev,
            detail: $detail,
            modified: $modified,
            missing: $missing,
            new: $new_f
        }'
}

# ---------------------------------------------------------------------------
# Signal 3: Marketplace Clone Detection
# ---------------------------------------------------------------------------

signal_3_clone_detection() {
    log "INFO" "Signal 3: Marketplace Clone Detection"

    local severity="${SEV_GREEN}"
    local detail=""
    local potential_clones="[]"
    local searched_keywords=0
    local total_results=0
    local clone_count=0
    local new_clone_count=0

    if [[ "${DRY_RUN}" == "true" ]]; then
        detail="DRY RUN: Would search ${#PV_KEYWORDS[@]} keywords on WebMCP Hub"
        update_worst "${severity}"
        jq -n \
            --arg sev "${severity}" \
            --arg detail "${detail}" \
            '{
                signal: 3,
                name: "Marketplace Clone Detection",
                severity: $sev,
                detail: $detail,
                searched_keywords: 0,
                total_results: 0,
                clone_count: 0,
                new_clone_count: 0,
                potential_clones: []
            }'
        return
    fi

    local api_failures=0

    for keyword in "${PV_KEYWORDS[@]}"; do
        searched_keywords=$((searched_keywords + 1))
        local search_url="${WEBMCP_HUB_BASE}/api/search?q=${keyword}&type=config"

        local response
        response=$(curl -sS --max-time "${CURL_TIMEOUT}" --fail "${search_url}" 2>/dev/null) || {
            log "WARN" "Signal 3: Search failed for keyword '${keyword}'"
            api_failures=$((api_failures + 1))
            continue
        }

        # Parse results — handle multiple possible API response shapes
        local results
        results=$(echo "${response}" | jq '.results // .data // .items // []' 2>/dev/null) || {
            log "WARN" "Signal 3: Unparseable response for keyword '${keyword}'"
            continue
        }

        local result_count
        result_count=$(echo "${results}" | jq 'length' 2>/dev/null || echo "0")
        total_results=$((total_results + result_count))

        local i=0
        while [[ ${i} -lt ${result_count} ]]; do
            local name
            name=$(echo "${results}" | jq -r ".[${i}].name // .[${i}].title // \"unknown\"")
            local author
            author=$(echo "${results}" | jq -r ".[${i}].author // .[${i}].owner // \"unknown\"")
            local url
            url=$(echo "${results}" | jq -r ".[${i}].url // .[${i}].link // \"\"")
            local description
            description=$(echo "${results}" | jq -r ".[${i}].description // \"\"" | head -c 200)

            # Skip our own configs
            local is_ours=false
            local author_lower
            author_lower=$(echo "${author}" | tr '[:upper:]' '[:lower:]')
            if [[ "${author_lower}" == *"nexvigilant"* ]]; then
                is_ours=true
            fi
            local our_name
            for our_name in "${OUR_CONFIG_NAMES[@]}"; do
                if [[ "${name}" == "${our_name}" ]]; then
                    is_ours=true
                    break
                fi
            done

            if [[ "${is_ours}" == "false" ]]; then
                # Compute similarity score
                local similarity_score=0
                local similarity_reasons="[]"

                local name_lower
                name_lower=$(echo "${name}" | tr '[:upper:]' '[:lower:]')
                local desc_lower
                desc_lower=$(echo "${description}" | tr '[:upper:]' '[:lower:]')

                # Check if name contains our distinctive suffixes
                for our_name in "${OUR_CONFIG_NAMES[@]}"; do
                    local our_suffix="${our_name#nexvigilant-}"
                    if [[ "${name_lower}" == *"${our_suffix}"* ]]; then
                        similarity_score=$((similarity_score + 30))
                        similarity_reasons=$(echo "${similarity_reasons}" | jq \
                            --arg r "Name contains '${our_suffix}'" '. + [$r]')
                    fi
                done

                # Description keyword overlap scoring
                local pv_term
                for pv_term in "pharmacovigilance" "adverse event" "signal detection" "faers" "naranjo" "prr" "drug safety" "icsr" "meddra"; do
                    if [[ "${desc_lower}" == *"${pv_term}"* ]]; then
                        similarity_score=$((similarity_score + 10))
                    fi
                done

                # Flag if similarity exceeds threshold
                if [[ ${similarity_score} -ge 20 ]]; then
                    clone_count=$((clone_count + 1))
                    potential_clones=$(echo "${potential_clones}" | jq \
                        --arg name "${name}" \
                        --arg author "${author}" \
                        --arg url "${url}" \
                        --arg keyword "${keyword}" \
                        --argjson score "${similarity_score}" \
                        --argjson reasons "${similarity_reasons}" \
                        '. + [{"name": $name, "author": $author, "url": $url, "matched_keyword": $keyword, "similarity_score": $score, "reasons": $reasons}]')
                    log "WARN" "Signal 3: Potential clone: '${name}' by ${author} (score: ${similarity_score})"
                fi
            fi

            i=$((i + 1))
        done
    done

    # Compare against previously known clones to detect new ones
    local known_clones
    known_clones=$(jq '.known_clones // []' "${STATE_FILE}" 2>/dev/null || echo "[]")
    local clone_url
    for clone_url in $(echo "${potential_clones}" | jq -r '.[].url // empty' 2>/dev/null); do
        [[ -z "${clone_url}" ]] && continue
        local is_known
        is_known=$(echo "${known_clones}" | jq --arg u "${clone_url}" '[.[] | select(. == $u)] | length')
        if [[ "${is_known}" == "0" ]]; then
            new_clone_count=$((new_clone_count + 1))
        fi
    done

    # Determine severity
    if [[ ${new_clone_count} -gt 0 ]]; then
        severity="${SEV_RED}"
        detail="${new_clone_count} NEW potential clone(s) detected from ${total_results} marketplace results"
    elif [[ ${clone_count} -gt 0 ]]; then
        severity="${SEV_AMBER}"
        detail="${clone_count} known potential clone(s) still present (no new ones)"
    elif [[ ${api_failures} -eq ${searched_keywords} ]] && [[ ${searched_keywords} -gt 0 ]]; then
        severity="${SEV_AMBER}"
        detail="All ${searched_keywords} API searches failed — WebMCP Hub may be unavailable"
    elif [[ ${total_results} -eq 0 ]] && [[ ${searched_keywords} -gt 0 ]]; then
        detail="Searched ${searched_keywords} keywords, 0 results, 0 clones detected"
    else
        detail="Searched ${searched_keywords} keywords, ${total_results} results, 0 clones"
    fi

    # Update known clones in state
    local all_clone_urls
    all_clone_urls=$(echo "${potential_clones}" | jq '[.[].url]')
    state_update --argjson clones "${all_clone_urls}" '.known_clones = $clones'

    update_worst "${severity}"

    jq -n \
        --arg sev "${severity}" \
        --arg detail "${detail}" \
        --argjson searched "${searched_keywords}" \
        --argjson total "${total_results}" \
        --argjson clones_n "${clone_count}" \
        --argjson new_n "${new_clone_count}" \
        --argjson clones "${potential_clones}" \
        '{
            signal: 3,
            name: "Marketplace Clone Detection",
            severity: $sev,
            detail: $detail,
            searched_keywords: $searched,
            total_results: $total,
            clone_count: $clones_n,
            new_clone_count: $new_n,
            potential_clones: $clones
        }'
}

# ---------------------------------------------------------------------------
# Signal 4: Supply Chain Integrity
# ---------------------------------------------------------------------------

signal_4_supply_chain() {
    log "INFO" "Signal 4: Supply Chain Integrity"

    local severity="${SEV_GREEN}"
    local detail=""
    local tls_status="UNKNOWN"
    local tls_expiry="unknown"
    local tls_issuer="unknown"
    local endpoint_reachable=false
    local configs_verified=false
    local mcp_remote_version="unknown"
    local mcp_remote_drift=false
    local tamper_detected="[]"
    local checks_passed=0
    local checks_total=4

    if [[ "${DRY_RUN}" == "true" ]]; then
        detail="DRY RUN: Would check TLS, config tampering, mcp-remote version"
        update_worst "${severity}"
        jq -n \
            --arg sev "${severity}" \
            --arg detail "${detail}" \
            '{
                signal: 4,
                name: "Supply Chain Integrity",
                severity: $sev,
                detail: $detail,
                checks_passed: 0,
                checks_total: 4,
                tls: {"status": "SKIPPED", "expiry": "N/A", "issuer": "N/A"},
                endpoint_reachable: false,
                configs_verified: false,
                mcp_remote: {"version": "unknown", "drift_detected": false},
                tamper_detected: []
            }'
        return
    fi

    # ---- Check 1: TLS endpoint health ----
    local http_code=""
    local ssl_verify=""
    local curl_output
    curl_output=$(make_temp)

    if curl -sS --max-time "${CURL_TIMEOUT}" \
        -w '%{ssl_verify_result}\n%{http_code}' \
        -o /dev/null "${WEBMCP_HUB_BASE}" > "${curl_output}" 2>/dev/null; then
        ssl_verify=$(head -1 "${curl_output}")
        http_code=$(tail -1 "${curl_output}")
    else
        # curl itself may write partial output even on failure
        ssl_verify=$(head -1 "${curl_output}" 2>/dev/null || echo "FAIL")
        http_code=$(tail -1 "${curl_output}" 2>/dev/null || echo "000")
    fi

    if [[ "${ssl_verify}" == "0" ]] && [[ "${http_code}" != "000" ]]; then
        tls_status="VALID"
        endpoint_reachable=true
        checks_passed=$((checks_passed + 1))
    elif [[ "${http_code}" == "000" ]]; then
        tls_status="UNREACHABLE"
        severity="${SEV_RED}"
        log "WARN" "Signal 4: WebMCP Hub endpoint unreachable"
    else
        tls_status="INVALID"
        severity="${SEV_RED}"
        log "WARN" "Signal 4: TLS verification failed (ssl_verify_result=${ssl_verify})"
    fi

    # Extract TLS certificate details if openssl available
    if command -v openssl >/dev/null 2>&1 && [[ "${endpoint_reachable}" == "true" ]]; then
        local cert_info
        cert_info=$(echo | openssl s_client -connect "www.webmcp-hub.com:443" \
            -servername "www.webmcp-hub.com" 2>/dev/null | \
            openssl x509 -noout -dates -issuer 2>/dev/null) || true
        tls_expiry=$(echo "${cert_info}" | grep 'notAfter' | sed 's/notAfter=//' || echo "unknown")
        tls_issuer=$(echo "${cert_info}" | grep 'issuer' | sed 's/issuer= *//' | head -c 100 || echo "unknown")
    fi

    # ---- Check 2: Remote config tampering detection ----
    if [[ "${endpoint_reachable}" == "true" ]] && [[ -f "${CHECKSUMS_FILE}" ]]; then
        local remote_match_count=0
        local remote_check_count=0
        local config_name

        for config_name in "${OUR_CONFIG_NAMES[@]}"; do
            local remote_url="${WEBMCP_HUB_BASE}/api/configs/${config_name}/raw"
            local remote_content
            remote_content=$(curl -sS --max-time "${CURL_TIMEOUT}" --fail "${remote_url}" 2>/dev/null) || continue
            remote_check_count=$((remote_check_count + 1))

            local remote_hash
            remote_hash=$(echo "${remote_content}" | sha256sum | cut -d' ' -f1)

            # Compare to local config
            local local_file
            for local_file in "${WEBMCP_CONFIGS}/${config_name}".{json,yaml,yml,toml}; do
                if [[ -f "${local_file}" ]]; then
                    local local_hash
                    local_hash=$(sha256sum "${local_file}" | cut -d' ' -f1)
                    if [[ "${remote_hash}" == "${local_hash}" ]]; then
                        remote_match_count=$((remote_match_count + 1))
                    else
                        tamper_detected=$(echo "${tamper_detected}" | jq \
                            --arg name "${config_name}" \
                            --arg local_h "${local_hash}" \
                            --arg remote_h "${remote_hash}" \
                            '. + [{"config": $name, "local_hash": $local_h, "remote_hash": $remote_h}]')
                        log "WARN" "Signal 4: Remote config mismatch: ${config_name}"
                    fi
                    break
                fi
            done
        done

        local tamper_count
        tamper_count=$(echo "${tamper_detected}" | jq 'length')
        if [[ ${tamper_count} -gt 0 ]]; then
            severity="${SEV_RED}"
        elif [[ ${remote_check_count} -gt 0 ]]; then
            configs_verified=true
            checks_passed=$((checks_passed + 1))
        fi
    else
        # Cannot verify — graceful skip
        checks_passed=$((checks_passed + 1))
    fi

    # ---- Check 3: mcp-remote package version ----
    if command -v npm >/dev/null 2>&1; then
        mcp_remote_version=$(npm view mcp-remote version 2>/dev/null || echo "unknown")
    fi

    if [[ "${mcp_remote_version}" != "unknown" ]]; then
        local expected_version
        expected_version=$(jq -r '.mcp_remote_version // empty' "${STATE_FILE}" 2>/dev/null || echo "")

        if [[ -z "${expected_version}" ]] || [[ "${expected_version}" == "null" ]]; then
            # First run — record baseline
            state_update --arg v "${mcp_remote_version}" '.mcp_remote_version = $v'
            checks_passed=$((checks_passed + 1))
        elif [[ "${mcp_remote_version}" != "${expected_version}" ]]; then
            mcp_remote_drift=true
            log "WARN" "Signal 4: mcp-remote version changed: ${expected_version} -> ${mcp_remote_version}"
            state_update --arg v "${mcp_remote_version}" '.mcp_remote_version = $v'
            if [[ "${severity}" != "${SEV_RED}" ]]; then
                severity="${SEV_AMBER}"
            fi
        else
            checks_passed=$((checks_passed + 1))
        fi
    fi

    # ---- Check 4: HTTP response validity ----
    if [[ "${endpoint_reachable}" == "true" ]]; then
        checks_passed=$((checks_passed + 1))
    fi

    # ---- Final severity determination ----
    local tamper_count
    tamper_count=$(echo "${tamper_detected}" | jq 'length')
    if [[ "${tls_status}" == "INVALID" ]]; then
        detail="FAIL: TLS verification failed for ${WEBMCP_HUB_BASE}"
    elif [[ "${tls_status}" == "UNREACHABLE" ]]; then
        detail="FAIL: WebMCP Hub endpoint unreachable"
    elif [[ ${tamper_count} -gt 0 ]]; then
        detail="FAIL: ${tamper_count} remote config(s) do not match local checksums"
    elif [[ "${mcp_remote_drift}" == "true" ]]; then
        detail="mcp-remote version drift detected (review recommended)"
    else
        detail="PASS: ${checks_passed}/${checks_total} supply chain checks passed"
    fi

    update_worst "${severity}"

    jq -n \
        --arg sev "${severity}" \
        --arg detail "${detail}" \
        --argjson passed "${checks_passed}" \
        --argjson total "${checks_total}" \
        --arg tls_s "${tls_status}" \
        --arg tls_e "${tls_expiry}" \
        --arg tls_i "${tls_issuer}" \
        --argjson reachable "${endpoint_reachable}" \
        --argjson verified "${configs_verified}" \
        --arg mcpv "${mcp_remote_version}" \
        --argjson drift "${mcp_remote_drift}" \
        --argjson tamper "${tamper_detected}" \
        '{
            signal: 4,
            name: "Supply Chain Integrity",
            severity: $sev,
            detail: $detail,
            checks_passed: $passed,
            checks_total: $total,
            tls: {status: $tls_s, expiry: $tls_e, issuer: $tls_i},
            endpoint_reachable: $reachable,
            configs_verified: $verified,
            mcp_remote: {version: $mcpv, drift_detected: $drift},
            tamper_detected: $tamper
        }'
}

# ---------------------------------------------------------------------------
# Signal 5: Version Drift Detection
# ---------------------------------------------------------------------------

signal_5_version_drift() {
    log "INFO" "Signal 5: Version Drift Detection"

    local severity="${SEV_GREEN}"
    local detail=""
    local local_tool_count=0
    local published_tool_count=0
    local local_microgram_count=0
    local published_microgram_refs=0
    local stale_refs="[]"
    local opportunity_gaps="[]"

    # Count local MCP tools — try API health endpoint first
    local health_response
    health_response=$(curl -s --max-time 5 http://localhost:3030/health 2>/dev/null || echo "{}")
    local_tool_count=$(echo "${health_response}" | jq '.tool_count // 0' 2>/dev/null || echo "0")

    # Fallback: count #[tool] annotations in Rust source files only
    # NOTE: --include='*.rs' is critical — without it, grep scans binaries and takes 60s+
    if [[ ${local_tool_count} -eq 0 ]]; then
        local_tool_count=$(grep -r --include='*.rs' -c '#\[tool\]' "${NEXCORE_ROOT}/crates/" 2>/dev/null | \
            awk -F: '{s+=$NF} END {print s+0}')
    fi

    # Count local micrograms
    local microgram_dir="${HOME}/Projects/rsk-core/rsk/micrograms"
    if [[ -d "${microgram_dir}" ]]; then
        local mc_files
        mc_files=$(find "${microgram_dir}" -name '*.yaml' -o -name '*.yml' 2>/dev/null || true)
        if [[ -n "${mc_files}" ]]; then
            local_microgram_count=$(echo "${mc_files}" | wc -l)
            local_microgram_count="${local_microgram_count// /}"
        fi
    fi

    # Count published capabilities from config files
    if [[ -d "${WEBMCP_CONFIGS}" ]]; then
        local tool_grep_pub
        tool_grep_pub=$(grep -rl '"tool"' "${WEBMCP_CONFIGS}/" 2>/dev/null || true)
        if [[ -n "${tool_grep_pub}" ]]; then
            published_tool_count=$(grep -r '"tool"' "${WEBMCP_CONFIGS}/" 2>/dev/null | wc -l)
            published_tool_count="${published_tool_count// /}"
        fi

        local mcg_grep
        mcg_grep=$(grep -rlE 'microgram|mcg|decision.tree' "${WEBMCP_CONFIGS}/" 2>/dev/null || true)
        if [[ -n "${mcg_grep}" ]]; then
            published_microgram_refs=$(grep -rE 'microgram|mcg|decision.tree' "${WEBMCP_CONFIGS}/" 2>/dev/null | wc -l)
            published_microgram_refs="${published_microgram_refs// /}"
        fi
    fi

    # Detect stale references: published configs referencing tools that no longer exist
    if [[ -d "${WEBMCP_CONFIGS}" ]]; then
        local config_file
        while IFS= read -r config_file; do
            [[ -z "${config_file}" ]] && continue
            local tool_names
            tool_names=$(jq -r '.. | .tool? // empty' "${config_file}" 2>/dev/null || true)
            local tool_name
            while IFS= read -r tool_name; do
                [[ -z "${tool_name}" ]] && continue
                local exists
                exists=$(grep -rl --include='*.rs' "\"${tool_name}\"" "${NEXCORE_ROOT}/crates/" 2>/dev/null | head -1 || echo "")
                if [[ -z "${exists}" ]]; then
                    stale_refs=$(echo "${stale_refs}" | jq \
                        --arg tool "${tool_name}" \
                        --arg file "$(basename "${config_file}")" \
                        '. + [{"type": "tool", "name": $tool, "config": $file}]')
                fi
            done <<< "${tool_names}"
        done < <(find "${WEBMCP_CONFIGS}" -name '*.json' 2>/dev/null)
    fi

    # Compute drift
    local tool_drift=0
    if [[ ${local_tool_count} -gt 0 ]] && [[ ${published_tool_count} -gt 0 ]]; then
        tool_drift=$((local_tool_count - published_tool_count))
    fi

    local stale_count
    stale_count=$(echo "${stale_refs}" | jq 'length')

    # Determine severity
    if [[ ${stale_count} -gt 0 ]]; then
        severity="${SEV_RED}"
        detail="${stale_count} stale reference(s) in published configs — remove or update"
    elif [[ ${tool_drift} -gt 50 ]]; then
        severity="${SEV_AMBER}"
        detail="Large capability gap: ${tool_drift} local tools not represented in published configs"
    else
        detail="Local: ${local_tool_count} tools, ${local_microgram_count} micrograms | Published: ${published_tool_count} tool refs, ${published_microgram_refs} microgram refs"
    fi

    # Persist drift metrics in state
    state_update \
        --argjson pub "${published_tool_count}" \
        --argjson loc "${local_tool_count}" \
        '.version_drift = {"published": $pub, "local": $loc}'

    update_worst "${severity}"

    jq -n \
        --arg sev "${severity}" \
        --arg detail "${detail}" \
        --argjson l_tools "${local_tool_count}" \
        --argjson l_mcg "${local_microgram_count}" \
        --argjson p_tools "${published_tool_count}" \
        --argjson p_mcg "${published_microgram_refs}" \
        --argjson drift "${tool_drift}" \
        --argjson stale "${stale_refs}" \
        --argjson gaps "${opportunity_gaps}" \
        '{
            signal: 5,
            name: "Version Drift Detection",
            severity: $sev,
            detail: $detail,
            local: {tool_count: $l_tools, microgram_count: $l_mcg},
            published: {tool_refs: $p_tools, microgram_refs: $p_mcg},
            tool_drift: $drift,
            stale_references: $stale,
            opportunity_gaps: $gaps
        }'
}

# ---------------------------------------------------------------------------
# Signal 6: Reputation Monitoring
# ---------------------------------------------------------------------------

signal_6_reputation() {
    log "INFO" "Signal 6: Reputation Monitoring"

    local severity="${SEV_GREEN}"
    local detail=""
    local current_votes="{}"
    local vote_deltas="{}"
    local total_vote_change=0
    local configs_checked=0

    if [[ "${DRY_RUN}" == "true" ]]; then
        detail="DRY RUN: Would check ${#OUR_CONFIG_NAMES[@]} config listings on WebMCP Hub"
        update_worst "${severity}"
        jq -n \
            --arg sev "${severity}" \
            --arg detail "${detail}" \
            '{
                signal: 6,
                name: "Reputation Monitoring",
                severity: $sev,
                detail: $detail,
                configs_checked: 0,
                total_vote_change: 0,
                current_votes: {},
                vote_deltas: {}
            }'
        return
    fi

    # Fetch current vote/comment counts for our configs
    local config_name
    for config_name in "${OUR_CONFIG_NAMES[@]}"; do
        local listing_url="${WEBMCP_HUB_BASE}/api/configs/${config_name}"
        local response
        response=$(curl -sS --max-time "${CURL_TIMEOUT}" --fail "${listing_url}" 2>/dev/null) || continue
        configs_checked=$((configs_checked + 1))

        local votes
        votes=$(echo "${response}" | jq '.votes // .stars // .likes // 0' 2>/dev/null || echo "0")
        local comments
        comments=$(echo "${response}" | jq '.comments // .reviews // 0' 2>/dev/null || echo "0")

        current_votes=$(echo "${current_votes}" | jq \
            --arg name "${config_name}" \
            --argjson votes "${votes}" \
            --argjson comments "${comments}" \
            '. + {($name): {"votes": $votes, "comments": $comments}}')
    done

    # Compare to previous state
    local previous_votes
    previous_votes=$(jq '.marketplace_votes // {}' "${STATE_FILE}" 2>/dev/null || echo "{}")

    for config_name in "${OUR_CONFIG_NAMES[@]}"; do
        local curr_v
        curr_v=$(echo "${current_votes}" | jq --arg n "${config_name}" '.[$n].votes // null')
        local prev_v
        prev_v=$(echo "${previous_votes}" | jq --arg n "${config_name}" '.[$n].votes // null')

        if [[ "${curr_v}" != "null" ]] && [[ "${prev_v}" != "null" ]]; then
            local delta=$((curr_v - prev_v))
            total_vote_change=$((total_vote_change + delta))

            if [[ ${delta} -ne 0 ]]; then
                vote_deltas=$(echo "${vote_deltas}" | jq \
                    --arg name "${config_name}" \
                    --argjson delta "${delta}" \
                    --argjson prev "${prev_v}" \
                    --argjson curr "${curr_v}" \
                    '. + {($name): {"delta": $delta, "previous": $prev, "current": $curr}}')
            fi
        fi
    done

    # Determine severity
    if [[ ${total_vote_change} -lt -5 ]]; then
        severity="${SEV_RED}"
        detail="Significant reputation drop: ${total_vote_change} net votes across ${configs_checked} configs"
    elif [[ ${total_vote_change} -lt 0 ]]; then
        severity="${SEV_AMBER}"
        detail="Minor reputation decline: ${total_vote_change} net votes across ${configs_checked} configs"
    elif [[ ${configs_checked} -eq 0 ]]; then
        severity="${SEV_AMBER}"
        detail="Could not fetch any config listings from WebMCP Hub"
    else
        detail="Reputation stable: ${total_vote_change} net vote change across ${configs_checked} configs"
    fi

    # Persist current votes in state
    state_update --argjson votes "${current_votes}" '.marketplace_votes = $votes'

    update_worst "${severity}"

    jq -n \
        --arg sev "${severity}" \
        --arg detail "${detail}" \
        --argjson checked "${configs_checked}" \
        --argjson total_change "${total_vote_change}" \
        --argjson votes "${current_votes}" \
        --argjson deltas "${vote_deltas}" \
        '{
            signal: 6,
            name: "Reputation Monitoring",
            severity: $sev,
            detail: $detail,
            configs_checked: $checked,
            total_vote_change: $total_change,
            current_votes: $votes,
            vote_deltas: $deltas
        }'
}

# ---------------------------------------------------------------------------
# Report assembly
# ---------------------------------------------------------------------------

assemble_report() {
    local -a signal_outputs=("$@")
    local ts
    ts="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

    # Combine signal outputs into a JSON array
    local signals_json="["
    local first=true
    local output
    for output in "${signal_outputs[@]}"; do
        if [[ "${first}" == "true" ]]; then
            first=false
        else
            signals_json="${signals_json},"
        fi
        signals_json="${signals_json}${output}"
    done
    signals_json="${signals_json}]"

    # Derive overall severity by scanning all signal outputs
    local overall_severity="${SEV_GREEN}"
    local has_red=false
    local has_amber=false
    for output in "${signal_outputs[@]}"; do
        local sig_sev
        sig_sev=$(echo "${output}" | jq -r '.severity // "GREEN"' 2>/dev/null || echo "GREEN")
        if [[ "${sig_sev}" == "RED" ]]; then
            has_red=true
        elif [[ "${sig_sev}" == "AMBER" ]]; then
            has_amber=true
        fi
    done
    if [[ "${has_red}" == "true" ]]; then
        overall_severity="${SEV_RED}"
        WORST_SEVERITY=2
    elif [[ "${has_amber}" == "true" ]]; then
        overall_severity="${SEV_AMBER}"
        WORST_SEVERITY=1
    fi

    # Update last_run timestamp in state
    state_update --arg ts "${ts}" '.last_run = $ts'

    # Produce final report using jq for safe JSON construction
    jq -n \
        --arg ts "${ts}" \
        --arg sev "${overall_severity}" \
        --argjson signals "${signals_json}" \
        '{
            qa_sentinel_report: {
                timestamp: $ts,
                overall_severity: $sev,
                signal_count: ($signals | length),
                red_count: ([$signals[] | select(.severity == "RED")] | length),
                amber_count: ([$signals[] | select(.severity == "AMBER")] | length),
                green_count: ([$signals[] | select(.severity == "GREEN")] | length),
                signals: $signals
            }
        }'
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

main() {
    log "INFO" "============================================"
    log "INFO" "QA Sentinel run started (PID $$)"
    log "INFO" "Mode: json_only=${JSON_ONLY} single_signal=${SINGLE_SIGNAL:-all} dry_run=${DRY_RUN}"

    ensure_state_file

    # Validate required tools
    local missing_tools=""
    local tool
    for tool in curl jq sha256sum; do
        if ! command -v "${tool}" >/dev/null 2>&1; then
            missing_tools="${missing_tools} ${tool}"
        fi
    done
    if [[ -n "${missing_tools}" ]]; then
        log "ERROR" "Missing required tools:${missing_tools}"
        echo "{\"error\": \"Missing required tools:${missing_tools}\"}"
        exit 2
    fi

    local -a signal_results=()

    if [[ -n "${SINGLE_SIGNAL}" ]]; then
        case "${SINGLE_SIGNAL}" in
            1) signal_results+=("$(signal_1_license_circumvention)") ;;
            2) signal_results+=("$(signal_2_config_integrity)") ;;
            3) signal_results+=("$(signal_3_clone_detection)") ;;
            4) signal_results+=("$(signal_4_supply_chain)") ;;
            5) signal_results+=("$(signal_5_version_drift)") ;;
            6) signal_results+=("$(signal_6_reputation)") ;;
        esac
    else
        signal_results+=("$(signal_1_license_circumvention)")
        signal_results+=("$(signal_2_config_integrity)")
        signal_results+=("$(signal_3_clone_detection)")
        signal_results+=("$(signal_4_supply_chain)")
        signal_results+=("$(signal_5_version_drift)")
        signal_results+=("$(signal_6_reputation)")
    fi

    local report
    report=$(assemble_report "${signal_results[@]}")

    # Output the report
    echo "${report}" | jq '.'

    local sev_label
    case ${WORST_SEVERITY} in
        0) sev_label="GREEN" ;;
        1) sev_label="AMBER" ;;
        2) sev_label="RED" ;;
        *) sev_label="UNKNOWN" ;;
    esac

    log "INFO" "QA Sentinel run completed — overall: ${sev_label}"
    log "INFO" "============================================"

    exit ${WORST_SEVERITY}
}

main
