#!/bin/bash
# Deploy NexCore API to Cloud Run
# Usage: ./deploy-cloudrun.sh [--always-on] [--private]
#
# Uses Dockerfile.api (excludes crates with local-only path deps like prima)
# Two-step: gcloud builds submit → gcloud run deploy

set -euo pipefail

PROJECT_ID="${PROJECT_ID:-$(gcloud config get-value project 2>/dev/null)}"
REGION="${REGION:-us-central1}"
SERVICE_NAME="nexcore-api"
IMAGE="gcr.io/${PROJECT_ID}/${SERVICE_NAME}"
TAG="${TAG:-latest}"
MIN_INSTANCES=0
AUTH_FLAG="--allow-unauthenticated"

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --always-on)
            MIN_INSTANCES=1
            echo "Always-on mode: min-instances=1 (no cold starts, higher cost)"
            shift
            ;;
        --private)
            AUTH_FLAG="--no-allow-unauthenticated"
            echo "Private mode: authentication required"
            shift
            ;;
        *)
            echo "Unknown option: $1"
            exit 1
            ;;
    esac
done

echo "Building and deploying NexCore API to Cloud Run..."
echo "   Project:  $PROJECT_ID"
echo "   Region:   $REGION"
echo "   Service:  $SERVICE_NAME"
echo "   Image:    $IMAGE:$TAG"
echo ""

if ! command -v gcloud &> /dev/null; then
    echo "gcloud CLI not found. Install from: https://cloud.google.com/sdk/docs/install"
    exit 1
fi

# Step 1: Build image via Cloud Build using Dockerfile.api
echo "Step 1/2: Building container image via Cloud Build..."
gcloud builds submit \
    --project "$PROJECT_ID" \
    --config cloudbuild-api.yaml \
    --substitutions "_TAG=$TAG" \
    --timeout 1800s

# Step 2: Deploy to Cloud Run
echo "Step 2/2: Deploying to Cloud Run..."
gcloud run deploy "$SERVICE_NAME" \
    --project "$PROJECT_ID" \
    --region "$REGION" \
    --image "$IMAGE:$TAG" \
    --platform managed \
    $AUTH_FLAG \
    --memory 512Mi \
    --cpu 1 \
    --min-instances "$MIN_INSTANCES" \
    --max-instances 10 \
    --timeout 60s \
    --concurrency 80 \
    --set-env-vars "RUST_LOG=nexcore_api=info"

# Get the service URL
SERVICE_URL=$(gcloud run services describe "$SERVICE_NAME" \
    --project "$PROJECT_ID" \
    --region "$REGION" \
    --format 'value(status.url)')

echo ""
echo "Deployment complete!"
echo ""
echo "Service URL: $SERVICE_URL"
echo "API Docs:    $SERVICE_URL/docs"
echo "Health:      $SERVICE_URL/health/ready"
echo ""
echo "Example calls:"
echo "  # Signal detection"
echo "  curl -X POST $SERVICE_URL/api/v1/pv/signal/complete \\"
echo "    -H 'Content-Type: application/json' \\"
echo "    -d '{\"a\": 15, \"b\": 100, \"c\": 20, \"d\": 10000}'"
echo ""
echo "  # Harm types"
echo "  curl $SERVICE_URL/api/v1/vigilance/harm-types"
