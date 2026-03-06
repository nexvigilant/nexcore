#!/usr/bin/env bash
set -euo pipefail

###############################################################################
# qa-sentinel-cron.sh — Cron wrapper for QA Sentinel
#
# Designed to run via cron every 6 hours:
#   0 */6 * * * /home/matthew/Projects/Active/nexcore/scripts/qa-sentinel-cron.sh
#
# Actions:
#   1. Rotates logs (keeps last 30 days, rotates at 10MB)
#   2. Runs qa-sentinel.sh --json
#   3. Archives the report with timestamp
#   4. If any RED signals, writes alert to qa-sentinel-alerts.json
#   5. Escalates persistent AMBER (3+ consecutive runs) to alert
#
# All output goes to stdout for cron mail capture.
###############################################################################

readonly SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
readonly SENTINEL="${SCRIPT_DIR}/qa-sentinel.sh"
readonly LOG_FILE="${HOME}/.claude/logs/qa-sentinel.log"
readonly LOG_DIR="${HOME}/.claude/logs"
readonly ALERTS_FILE="${HOME}/.claude/data/qa-sentinel-alerts.json"
readonly REPORTS_DIR="${HOME}/.claude/data/qa-sentinel-reports"

# ---------------------------------------------------------------------------
# Log rotation — keep last 30 days, rotate at 10MB
# ---------------------------------------------------------------------------

rotate_logs() {
    if [[ -f "${LOG_FILE}" ]]; then
        local log_size
        log_size=$(stat -c%s "${LOG_FILE}" 2>/dev/null || echo "0")

        # Rotate if log exceeds 10MB
        if [[ ${log_size} -gt 10485760 ]]; then
            local ts
            ts="$(date -u '+%Y%m%dT%H%M%SZ')"
            cp "${LOG_FILE}" "${LOG_FILE}.${ts}"
            : > "${LOG_FILE}"
            echo "$(date -u '+%Y-%m-%dT%H:%M:%SZ') Log rotated (was ${log_size} bytes)" >> "${LOG_FILE}"
        fi
    fi

    # Delete rotated logs older than 30 days
    find "${LOG_DIR}" -name 'qa-sentinel.log.*' -mtime +30 -delete 2>/dev/null || true

    # Delete archived reports older than 30 days
    if [[ -d "${REPORTS_DIR}" ]]; then
        find "${REPORTS_DIR}" -name '*.json' -mtime +30 -delete 2>/dev/null || true
    fi
}

# ---------------------------------------------------------------------------
# Alert management
# ---------------------------------------------------------------------------

ensure_alerts_file() {
    mkdir -p "$(dirname "${ALERTS_FILE}")"
    if [[ ! -f "${ALERTS_FILE}" ]] || [[ ! -s "${ALERTS_FILE}" ]]; then
        echo '{"alerts": [], "last_updated": null}' > "${ALERTS_FILE}"
    fi
    # Validate JSON
    if ! jq empty "${ALERTS_FILE}" 2>/dev/null; then
        echo '{"alerts": [], "last_updated": null}' > "${ALERTS_FILE}"
    fi
}

write_alerts() {
    local report="$1"
    local min_severity="${2:-RED}"
    local ts
    ts="$(date -u '+%Y-%m-%dT%H:%M:%SZ')"

    ensure_alerts_file

    # Extract signals matching the minimum severity
    local target_signals
    if [[ "${min_severity}" == "RED" ]]; then
        target_signals=$(echo "${report}" | jq '[.qa_sentinel_report.signals[] | select(.severity == "RED")]' 2>/dev/null || echo "[]")
    else
        target_signals=$(echo "${report}" | jq '[.qa_sentinel_report.signals[] | select(.severity == "RED" or .severity == "AMBER")]' 2>/dev/null || echo "[]")
    fi

    local signal_count
    signal_count=$(echo "${target_signals}" | jq 'length' 2>/dev/null || echo "0")

    if [[ ${signal_count} -eq 0 ]]; then
        return
    fi

    # Build alert entries
    local new_alerts="[]"
    local i=0
    while [[ ${i} -lt ${signal_count} ]]; do
        local signal_num
        signal_num=$(echo "${target_signals}" | jq ".[${i}].signal" 2>/dev/null || echo "0")
        local signal_name
        signal_name=$(echo "${target_signals}" | jq -r ".[${i}].name" 2>/dev/null || echo "unknown")
        local signal_sev
        signal_sev=$(echo "${target_signals}" | jq -r ".[${i}].severity" 2>/dev/null || echo "UNKNOWN")
        local signal_detail
        signal_detail=$(echo "${target_signals}" | jq -r ".[${i}].detail" 2>/dev/null || echo "")

        new_alerts=$(echo "${new_alerts}" | jq \
            --arg ts "${ts}" \
            --argjson signal "${signal_num}" \
            --arg name "${signal_name}" \
            --arg sev "${signal_sev}" \
            --arg detail "${signal_detail}" \
            '. + [{"timestamp": $ts, "signal": $signal, "name": $name, "severity": $sev, "detail": $detail, "acknowledged": false}]')

        i=$((i + 1))
    done

    # Append new alerts, keep last 100, update timestamp atomically
    local updated
    updated=$(jq \
        --argjson new "${new_alerts}" \
        --arg ts "${ts}" \
        '.alerts = (.alerts + $new) | .alerts = .alerts[-100:] | .last_updated = $ts' \
        "${ALERTS_FILE}")

    local tmp
    tmp=$(mktemp)
    echo "${updated}" | jq '.' > "${tmp}" 2>/dev/null && cp "${tmp}" "${ALERTS_FILE}"
    rm -f "${tmp}"

    echo "[ALERT] ${signal_count} signal(s) at ${min_severity}+ severity written to ${ALERTS_FILE}"
}

# ---------------------------------------------------------------------------
# Archive report
# ---------------------------------------------------------------------------

archive_report() {
    local report="$1"
    mkdir -p "${REPORTS_DIR}"

    local ts
    ts="$(date -u '+%Y%m%dT%H%M%SZ')"
    local archive_file="${REPORTS_DIR}/sentinel-${ts}.json"

    local tmp
    tmp=$(mktemp)
    echo "${report}" | jq '.' > "${tmp}" 2>/dev/null && cp "${tmp}" "${archive_file}"
    rm -f "${tmp}"
}

# ---------------------------------------------------------------------------
# Persistent AMBER detection
# ---------------------------------------------------------------------------

count_consecutive_amber() {
    if [[ ! -d "${REPORTS_DIR}" ]]; then
        echo 0
        return
    fi

    local count=0
    local rpt
    while IFS= read -r rpt; do
        [[ -z "${rpt}" ]] && continue
        local sev
        sev=$(jq -r '.qa_sentinel_report.overall_severity' "${rpt}" 2>/dev/null || echo "")
        if [[ "${sev}" == "AMBER" ]]; then
            count=$((count + 1))
        else
            break
        fi
    done < <(ls -t "${REPORTS_DIR}"/sentinel-*.json 2>/dev/null | head -5)

    echo "${count}"
}

# ---------------------------------------------------------------------------
# Main
# ---------------------------------------------------------------------------

main() {
    rotate_logs

    # Verify sentinel script exists and is executable
    if [[ ! -x "${SENTINEL}" ]]; then
        echo "ERROR: qa-sentinel.sh not found or not executable at ${SENTINEL}" >&2
        exit 2
    fi

    # Run sentinel and capture both output and exit code
    local report=""
    local exit_code=0
    report=$("${SENTINEL}" --json 2>/dev/null) || exit_code=$?

    # Validate we got valid JSON back
    if [[ -z "${report}" ]] || ! echo "${report}" | jq empty 2>/dev/null; then
        echo "ERROR: qa-sentinel.sh produced invalid or empty output (exit code: ${exit_code})" >&2
        exit 2
    fi

    # Archive the report regardless of severity
    archive_report "${report}"

    # Write alerts for RED signals
    if [[ ${exit_code} -eq 2 ]]; then
        write_alerts "${report}" "RED"
    fi

    # Escalate persistent AMBER (3+ consecutive runs)
    if [[ ${exit_code} -eq 1 ]]; then
        local consecutive_amber
        consecutive_amber=$(count_consecutive_amber)

        if [[ ${consecutive_amber} -ge 3 ]]; then
            write_alerts "${report}" "AMBER"
            echo "[ESCALATION] Persistent AMBER across ${consecutive_amber} consecutive runs"
        fi
    fi

    # Print summary to stdout (captured by cron mail)
    local overall
    overall=$(echo "${report}" | jq -r '.qa_sentinel_report.overall_severity' 2>/dev/null || echo "UNKNOWN")
    local signal_count
    signal_count=$(echo "${report}" | jq '.qa_sentinel_report.signal_count // 0' 2>/dev/null || echo "0")
    local red_count
    red_count=$(echo "${report}" | jq '.qa_sentinel_report.red_count // 0' 2>/dev/null || echo "0")
    local amber_count
    amber_count=$(echo "${report}" | jq '.qa_sentinel_report.amber_count // 0' 2>/dev/null || echo "0")

    echo "$(date -u '+%Y-%m-%dT%H:%M:%SZ') QA Sentinel: ${overall} (${signal_count} signals: ${red_count} RED, ${amber_count} AMBER)"

    exit ${exit_code}
}

main
