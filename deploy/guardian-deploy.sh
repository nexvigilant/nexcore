#!/usr/bin/env zsh
# guardian-deploy.sh — Deploy nexcore-api to guardian.nexvigilant.com
# Idempotent: safe to run multiple times
# Usage: ./guardian-deploy.sh [--rollback]

set -euo pipefail

# ── Config ────────────────────────────────────────────────────────────────────
BINARY_NAME="nexcore-api"
CRATE_NAME="nexcore-api"
GCE_INSTANCE="kellnr-registry"
GCE_ZONE="us-central1-a"
GCE_USER="matthew"
REMOTE_BINARY_PATH="/usr/local/bin/nexcore-api"
REMOTE_BACKUP_PATH="/usr/local/bin/nexcore-api.prev"
SYSTEMD_SERVICE="guardian-api"
HEALTH_URL="https://guardian.nexvigilant.com/health"
HEALTH_RETRIES=10
HEALTH_WAIT_SEC=3
NEXCORE_ROOT="${0:A:h}/.."

# ── Helpers ──────────────────────────────────────────────────────────────────
log()  { print "[$(date -u +%H:%M:%SZ)] $*" }
err()  { print "[$(date -u +%H:%M:%SZ)] ERROR: $*" >&2 }
die()  { err "$*"; exit 1 }

gce_run() { gcloud compute ssh "${GCE_USER}@${GCE_INSTANCE}" --zone="${GCE_ZONE}" -- "$@" }
gce_scp() { gcloud compute scp "$1" "${GCE_USER}@${GCE_INSTANCE}:$2" --zone="${GCE_ZONE}" }

health_check() {
    local i=0
    while (( i < HEALTH_RETRIES )); do
        if curl -sf "${HEALTH_URL}" | grep -q '"status":"healthy"'; then
            log "Health check passed."
            return 0
        fi
        i=$(( i + 1 ))
        log "Health check attempt ${i}/${HEALTH_RETRIES} failed, waiting ${HEALTH_WAIT_SEC}s..."
        sleep "${HEALTH_WAIT_SEC}"
    done
    return 1
}

rollback() {
    err "Health check failed — initiating rollback"
    gce_run "sudo cp -f '${REMOTE_BACKUP_PATH}' '${REMOTE_BINARY_PATH}' && sudo systemctl restart ${SYSTEMD_SERVICE}" || true
    if health_check; then
        log "Rollback succeeded."
    else
        err "Rollback also failed. Manual intervention required."
        err "  gcloud compute ssh ${GCE_USER}@${GCE_INSTANCE} --zone=${GCE_ZONE}"
    fi
    exit 1
}

# ── Rollback-only mode ────────────────────────────────────────────────────────
if [[ "${1:-}" == "--rollback" ]]; then
    log "Manual rollback requested"
    rollback
fi

# ── Step 1: Build release binary ──────────────────────────────────────────────
log "Building ${CRATE_NAME} release binary..."
(cd "${NEXCORE_ROOT}" && cargo build --release -p "${CRATE_NAME}")
LOCAL_BINARY="${NEXCORE_ROOT}/target/release/${BINARY_NAME}"
[[ -f "${LOCAL_BINARY}" ]] || die "Build succeeded but binary not found at ${LOCAL_BINARY}"
log "Binary size: $(du -sh "${LOCAL_BINARY}" | cut -f1)"

# ── Step 2: Back up existing binary on remote ─────────────────────────────────
log "Backing up existing remote binary..."
gce_run "sudo cp -f '${REMOTE_BINARY_PATH}' '${REMOTE_BACKUP_PATH}' 2>/dev/null || true"

# ── Step 3: Upload new binary ─────────────────────────────────────────────────
log "Uploading binary to ${GCE_INSTANCE}..."
gce_scp "${LOCAL_BINARY}" "/tmp/${BINARY_NAME}"
gce_run "sudo mv /tmp/${BINARY_NAME} ${REMOTE_BINARY_PATH} && sudo chmod 755 ${REMOTE_BINARY_PATH}"

# ── Step 4: Restart service ───────────────────────────────────────────────────
log "Restarting ${SYSTEMD_SERVICE}..."
gce_run "sudo systemctl restart ${SYSTEMD_SERVICE}"

# ── Step 5: Health check ──────────────────────────────────────────────────────
log "Running health check against ${HEALTH_URL}..."
sleep 2  # give the process a moment to bind
if ! health_check; then
    rollback
fi

log "Deploy complete."
log "  Version: $(curl -sf ${HEALTH_URL} | grep -o '"version":"[^"]*"' || echo unknown)"
