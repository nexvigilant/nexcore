# NVOS Phase 1 Deployment Runbook

**Status:** Ready to execute (blocked on VPN/GCP connectivity)
**Prerequisite:** WireGuard (wg0) must allow HTTPS to *.googleapis.com

## Pre-flight Check

```bash
# Verify GCP connectivity
timeout 10 gcloud run services list --project nexvigilant-digital-clubhouse --region us-central1

# If timeout: VPN is blocking. Either:
# 1. Disconnect VPN temporarily
# 2. Add googleapis.com to VPN split tunnel
# 3. Route GCP traffic outside tunnel
```

## Step 1: Deploy nexcore-api to Cloud Run

```bash
cd ~/nexcore
bash deploy-cloudrun.sh
```

This will:
1. Upload workspace to Cloud Build (respects .gcloudignore)
2. Build using Dockerfile.api (excludes nexcore-mcp, nexcore-renderer — prima path deps)
3. Push image to gcr.io/nexvigilant-digital-clubhouse/nexcore-api:latest
4. Deploy to Cloud Run (us-central1, 512Mi, 1 CPU, 0-10 instances)
5. Print the SERVICE_URL

**Expected build time:** ~15-25 min (Rust release build on E2_HIGHCPU_8)

Options:
- `--always-on` — min-instances=1 (no cold starts, $$$)
- `--private` — requires authentication (use with OIDC token)

## Step 2: Verify nexcore-api

```bash
# Health check
curl $SERVICE_URL/health/ready

# API docs
open $SERVICE_URL/docs

# Signal detection test
curl -X POST $SERVICE_URL/api/v1/pv/signal/complete \
  -H 'Content-Type: application/json' \
  -d '{"a": 15, "b": 100, "c": 20, "d": 10000}'

# Harm types
curl $SERVICE_URL/api/v1/vigilance/harm-types
```

## Step 3: Activate Signal Screening Scheduler

```bash
# Replace $SERVICE_URL with actual URL from Step 1
gcloud scheduler jobs create http nexcore-signal-scan \
  --project nexvigilant-digital-clubhouse \
  --schedule="0 */6 * * *" \
  --uri="$SERVICE_URL/api/v1/pv/signal/complete" \
  --http-method=POST \
  --headers="Content-Type=application/json" \
  --message-body='{"a":15,"b":100,"c":20,"d":10000}' \
  --time-zone="America/New_York" \
  --location="us-central1"

# Verify
gcloud scheduler jobs list --project nexvigilant-digital-clubhouse --location us-central1
```

## Step 4: Verify Metrics Server

```bash
# Check if claude-metrics is already deployed
gcloud run services describe claude-metrics \
  --project nexvigilant-digital-clubhouse \
  --region us-central1 \
  --format='value(status.url)'

# Health check
curl $METRICS_URL/health

# Status (requires API key)
curl -H "X-API-Key: $API_KEY" $METRICS_URL/api/v1/status
```

## Architecture After Deployment

```
Cloud Run (nexvigilant-digital-clubhouse, us-central1)
├── nexcore-api          → :3030  (84+ REST routes, signal detection)
├── claude-metrics       → :9091  (14 endpoints, Prometheus metrics)
└── Cloud Scheduler
    └── nexcore-signal-scan (every 6h → nexcore-api signal/complete)
```

## Deployment Files

| File | Purpose |
|------|---------|
| `Dockerfile.api` | Multi-stage build, excludes prima-dependent crates |
| `Dockerfile` | Full build (nexcore-api + nexcore-mcp) — use for local Docker |
| `Dockerfile.slim` | Pre-built binary only (fastest deploy) |
| `Dockerfile.cloud` | Pre-built binary (minimal, no healthcheck) |
| `cloudbuild-api.yaml` | Cloud Build config for nexcore-api |
| `cloudbuild.yaml` | Cloud Build config for full build (legacy) |
| `deploy-cloudrun.sh` | Automated deploy script |
| `.gcloudignore` | Excludes target/, .git/, archives from upload |
| `scheduler/signal-screening-job.yaml` | Scheduler job spec |

## Rollback

```bash
# List revisions
gcloud run revisions list --service nexcore-api \
  --project nexvigilant-digital-clubhouse --region us-central1

# Route traffic to previous revision
gcloud run services update-traffic nexcore-api \
  --to-revisions=REVISION_NAME=100 \
  --project nexvigilant-digital-clubhouse --region us-central1
```
