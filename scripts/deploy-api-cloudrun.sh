#!/usr/bin/env bash
set -euo pipefail

# deploy-api-cloudrun.sh — Deploy nexcore-api to Cloud Run
#
# Supports WebSocket terminal sessions with PTY.
# Cloud Run WebSocket: max 3600s idle timeout, session affinity for sticky routing.
#
# Usage:
#   ./scripts/deploy-api-cloudrun.sh              # Canary deploy (10% → health → 100%)
#   ./scripts/deploy-api-cloudrun.sh --no-canary   # Full deploy immediately
#   ./scripts/deploy-api-cloudrun.sh --build-only   # Build image, don't deploy

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(dirname "$SCRIPT_DIR")"

# Config
SERVICE_NAME="nexcore-api"
REGION="us-central1"
PROJECT_ID="nexvigilant-digital-clubhouse"
IMAGE="us-central1-docker.pkg.dev/${PROJECT_ID}/nexcore/${SERVICE_NAME}"
DOCKERFILE="Dockerfile.api"

# Parse flags
CANARY=true
BUILD_ONLY=false
for arg in "$@"; do
    case "$arg" in
        --no-canary) CANARY=false ;;
        --build-only) BUILD_ONLY=true ;;
    esac
done

cd "$PROJECT_ROOT"

echo "=== nexcore-api Cloud Run Deploy ==="
echo "  Project:  ${PROJECT_ID}"
echo "  Service:  ${SERVICE_NAME}"
echo "  Region:   ${REGION}"
echo "  Canary:   ${CANARY}"
echo ""

# Step 1: Build with Cloud Build (uses cloudbuild-api.yaml → Dockerfile.api)
echo ">>> Building container image..."
gcloud builds submit \
    --project="${PROJECT_ID}" \
    --config=cloudbuild-api.yaml \
    .

if [ "$BUILD_ONLY" = true ]; then
    echo ">>> Build complete. Skipping deploy (--build-only)."
    exit 0
fi

# Step 2: Deploy to Cloud Run
echo ">>> Deploying to Cloud Run..."
deploy_flags=(
    --image="${IMAGE}:latest"
    --region="${REGION}"
    --project="${PROJECT_ID}"
    --platform=managed
    --allow-unauthenticated
    --port=3030
    --cpu=2
    --memory=2Gi
    --min-instances=0
    --max-instances=10
    --timeout=3600
    --session-affinity
    --concurrency=50
    --execution-environment=gen2
    --set-env-vars="PORT=3030,BIND_ADDR=0.0.0.0,RUST_LOG=nexcore_api=info,SHELL=/bin/bash,TERM=xterm-256color,FIREBASE_PROJECT_ID=nexvigilant-digital-clubhouse"
)

# GCS FUSE volume mount for persistent user home directories
# gs://nexvigilant-cloud-shell-homes/{user_id}/ → /home/cloud-shell/{user_id}/
FUSE_FLAGS=(
    --add-volume=name=cloud-shell-homes,type=cloud-storage,bucket=nexvigilant-cloud-shell-homes
    --add-volume-mount=volume=cloud-shell-homes,mount-path=/home/cloud-shell
)

if [ "$CANARY" = true ]; then
    # Canary: route 10% traffic to new revision
    echo ">>> Canary deploy: 10% traffic to new revision..."
    gcloud beta run deploy "${SERVICE_NAME}" "${deploy_flags[@]}" "${FUSE_FLAGS[@]}" --no-traffic

    # Get the latest revision name
    LATEST_REV=$(gcloud run revisions list \
        --service="${SERVICE_NAME}" \
        --region="${REGION}" \
        --project="${PROJECT_ID}" \
        --sort-by="~metadata.creationTimestamp" \
        --limit=1 \
        --format="value(metadata.name)")

    echo ">>> Routing 10% to ${LATEST_REV}..."
    gcloud run services update-traffic "${SERVICE_NAME}" \
        --region="${REGION}" \
        --project="${PROJECT_ID}" \
        --to-revisions="${LATEST_REV}=10"

    # Health check
    echo ">>> Health check..."
    SERVICE_URL=$(gcloud run services describe "${SERVICE_NAME}" \
        --region="${REGION}" \
        --project="${PROJECT_ID}" \
        --format="value(status.url)")

    if curl -sf "${SERVICE_URL}/health" > /dev/null 2>&1; then
        echo ">>> Health check passed. Promoting to 100%..."
        gcloud run services update-traffic "${SERVICE_NAME}" \
            --region="${REGION}" \
            --project="${PROJECT_ID}" \
            --to-latest
        echo ">>> Canary promoted to 100%."
    else
        echo ">>> Health check FAILED. Rolling back..."
        gcloud run services update-traffic "${SERVICE_NAME}" \
            --region="${REGION}" \
            --project="${PROJECT_ID}" \
            --to-latest
        echo ">>> Rollback complete. Check logs:"
        echo "    gcloud logging read 'resource.type=cloud_run_revision AND resource.labels.service_name=${SERVICE_NAME}' --project=${PROJECT_ID} --limit=50"
        exit 1
    fi
else
    # Full deploy: route all traffic immediately
    gcloud beta run deploy "${SERVICE_NAME}" "${deploy_flags[@]}" "${FUSE_FLAGS[@]}"
fi

# Step 3: Verify
SERVICE_URL=$(gcloud run services describe "${SERVICE_NAME}" \
    --region="${REGION}" \
    --project="${PROJECT_ID}" \
    --format="value(status.url)")

echo ""
echo "=== Deploy Complete ==="
echo "  URL:      ${SERVICE_URL}"
echo "  Health:   ${SERVICE_URL}/health"
echo "  Docs:     ${SERVICE_URL}/docs"
echo "  Terminal WS: wss://$(echo "${SERVICE_URL}" | sed 's|https://||')/api/v1/terminal/ws"
echo ""
echo "  Next: Set NEXT_PUBLIC_NEXCORE_API_URL=${SERVICE_URL} in Vercel"
echo "  Or:   Map api.nexvigilant.com → Cloud Run via domain mapping"
