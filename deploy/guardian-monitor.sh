#!/usr/bin/env zsh
# guardian-monitor.sh — Health + resource monitor for Guardian API
# Runs as a cron job or systemd timer every 60 seconds.
# Writes alerts to syslog (journald picks them up via logger).
#
# Install (on GCE instance):
#   sudo cp guardian-monitor.sh /usr/local/bin/
#   sudo chmod 755 /usr/local/bin/guardian-monitor.sh
#
# Systemd timer (preferred over cron):
#   sudo cp guardian-monitor.timer /etc/systemd/system/
#   sudo cp guardian-monitor.service /etc/systemd/system/
#   sudo systemctl enable --now guardian-monitor.timer

set -euo pipefail

# ── Config ────────────────────────────────────────────────────────────────────
HEALTH_URL="http://127.0.0.1:3030/health"
SYSTEMD_SERVICE="guardian-api"
LOG_DIR="/var/log/guardian"
SYSLOG_TAG="guardian-monitor"
DISK_WARN_PCT=80      # Warn when /var/lib/guardian is over this % full
DISK_CRIT_PCT=90      # Critical threshold

# ── Helpers ──────────────────────────────────────────────────────────────────
log()  { logger -t "${SYSLOG_TAG}" "[INFO]  $*" }
warn() { logger -t "${SYSLOG_TAG}" "[WARN]  $*" }
crit() { logger -t "${SYSLOG_TAG}" "[CRIT]  $*" }

# ── Health check ──────────────────────────────────────────────────────────────
check_health() {
    local response
    response=$(curl -sf --max-time 5 "${HEALTH_URL}" 2>/dev/null) || {
        crit "Health check FAILED — ${HEALTH_URL} unreachable"
        # Attempt auto-restart if service is dead
        if ! systemctl is-active --quiet "${SYSTEMD_SERVICE}"; then
            warn "Service ${SYSTEMD_SERVICE} is not active — attempting restart"
            systemctl restart "${SYSTEMD_SERVICE}" && log "Service restarted" || crit "Restart failed — manual intervention required"
        fi
        return 1
    }

    local status
    status=$(print "${response}" | grep -o '"status":"[^"]*"' | cut -d'"' -f4 || true)
    if [[ "${status}" != "healthy" ]]; then
        warn "Health endpoint returned unexpected status: ${status:-unknown}"
        return 1
    fi

    log "Health OK (status=${status})"
}

# ── Disk space check ──────────────────────────────────────────────────────────
check_disk() {
    local pct
    pct=$(df /var/lib/guardian 2>/dev/null | awk 'NR==2 {gsub(/%/,"",$5); print $5}') || return 0

    if (( pct >= DISK_CRIT_PCT )); then
        crit "Disk usage CRITICAL: /var/lib/guardian at ${pct}% — audit log growth risk"
    elif (( pct >= DISK_WARN_PCT )); then
        warn "Disk usage HIGH: /var/lib/guardian at ${pct}%"
    else
        log "Disk OK: /var/lib/guardian at ${pct}%"
    fi
}

# ── Service status ────────────────────────────────────────────────────────────
check_service() {
    if systemctl is-active --quiet "${SYSTEMD_SERVICE}"; then
        log "Service ${SYSTEMD_SERVICE} is active"
    else
        crit "Service ${SYSTEMD_SERVICE} is NOT active"
    fi
}

# ── Run all checks ────────────────────────────────────────────────────────────
check_service
check_health
check_disk
