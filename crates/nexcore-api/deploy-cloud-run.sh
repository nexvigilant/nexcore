#!/usr/bin/env zsh
# deploy-cloud-run.sh — Deploy NexCore API to Google Cloud Run
# Usage: ./deploy-cloud-run.sh [--build-local]
set -euo pipefail

PROJECT_ID="nexvigilant-digital-clubhouse"
REGION="us-central1"
SERVICE_NAME="nexcore-api"
IMAGE_NAME="gcr.io/${PROJECT_ID}/${SERVICE_NAME}"

echo "=== NexCore API → Cloud Run ==="
echo "Project: ${PROJECT_ID}"
echo "Region:  ${REGION}"
echo "Service: ${SERVICE_NAME}"
echo ""

# Check gcloud auth
if ! gcloud auth list --filter="status:ACTIVE" --format="value(account)" 2>/dev/null | head -1 | grep -q '@'; then
  echo "ERROR: No active gcloud auth. Run: gcloud auth login"
  exit 1
fi

# Set project
gcloud config set project "${PROJECT_ID}" 2>/dev/null

# Option 1: Build locally and push (faster if binary already exists)
if [[ "${1:-}" == "--build-local" ]]; then
  echo "Building locally and pushing to GCR..."

  # Use pre-built binary with minimal container
  local_bin="$(dirname "$0")/target/release/nexcore-api"
  if [[ ! -f "${local_bin}" ]]; then
    echo "Building release binary..."
    cd "$(dirname "$0")"
    cargo build --release
  fi

  # Create a minimal Dockerfile that just copies the binary
  tmp_dockerfile=$(mktemp)
  cat > "${tmp_dockerfile}" << 'DOCKERFILE'
FROM debian:bookworm-slim
RUN apt-get update && apt-get install -y --no-install-recommends ca-certificates libssl3 curl && rm -rf /var/lib/apt/lists/*
RUN groupadd --gid 1000 appuser && useradd --uid 1000 --gid 1000 --create-home appuser
WORKDIR /app
COPY nexcore-api /app/nexcore-api
RUN chmod +x /app/nexcore-api
USER appuser
ENV PORT=8080 BIND_ADDR=0.0.0.0 RUST_LOG=nexcore_api=info,tower_http=info
EXPOSE 8080
HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 CMD curl -f http://localhost:${PORT}/health || exit 1
ENTRYPOINT ["/app/nexcore-api"]
DOCKERFILE

  # Build and push
  cd "$(dirname "$0")"
  docker build -f "${tmp_dockerfile}" -t "${IMAGE_NAME}:latest" --context target/release/ .
  docker push "${IMAGE_NAME}:latest"
  rm -f "${tmp_dockerfile}"

# Option 2: Cloud Build (builds from source in GCP)
else
  echo "Building with Cloud Build and deploying..."
  cd "$(dirname "$0")"

  # Deploy directly from source (Cloud Build handles Dockerfile)
  gcloud run deploy "${SERVICE_NAME}" \
    --source . \
    --region "${REGION}" \
    --platform managed \
    --allow-unauthenticated \
    --port 8080 \
    --memory 512Mi \
    --cpu 1 \
    --min-instances 0 \
    --max-instances 10 \
    --timeout 300 \
    --set-env-vars "PORT=8080,BIND_ADDR=0.0.0.0,FIREBASE_PROJECT_ID=${PROJECT_ID},RUST_LOG=nexcore_api=info" \
    --concurrency 250 \
    --ingress all
fi

echo ""
echo "=== Deployment complete ==="

# Get service URL
SERVICE_URL=$(gcloud run services describe "${SERVICE_NAME}" --region "${REGION}" --format="value(status.url)" 2>/dev/null || echo "")
if [[ -n "${SERVICE_URL}" ]]; then
  echo "Service URL: ${SERVICE_URL}"
  echo "Health check: ${SERVICE_URL}/health"
  echo "API docs: ${SERVICE_URL}/docs"
  echo "OpenAPI spec: ${SERVICE_URL}/openapi.json"
fi
