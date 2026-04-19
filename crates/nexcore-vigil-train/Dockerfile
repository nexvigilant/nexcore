# Vigil API — Cloud Run deployable image
#
# Multi-stage build: compile the Rust binary in a full-featured stage, then
# copy the thin binary + model weights + tokenizer into a small runtime image.
#
# Build locally:
#   docker build -t gcr.io/nexvigilant-digital-clubhouse/vigil-api:latest .
#
# Deploy to Cloud Run:
#   docker push gcr.io/nexvigilant-digital-clubhouse/vigil-api:latest
#   gcloud run deploy vigil-api \
#     --image gcr.io/nexvigilant-digital-clubhouse/vigil-api:latest \
#     --region us-central1 \
#     --memory 4Gi --cpu 2 \
#     --allow-unauthenticated \
#     --port 8080
#
# Expected cold-start: ~8-12s (model load from disk). Warm request: 3-10s
# depending on token count. Memory steady-state: ~2 GB with f32 model.

# ─── Stage 1: build the Rust binary ──────────────────────────────────────────
FROM rust:1.85-slim-bookworm AS builder

WORKDIR /build

# System deps for candle + tokenizers (oniguruma needs cmake + clang)
RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev cmake clang build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy the entire nexcore workspace. This is a pragmatic choice — Cargo's
# workspace resolver needs to see the root Cargo.toml to resolve `workspace =
# true` dependencies. Use .dockerignore to exclude non-essential crates.
COPY . .

# Build ONLY the nexcore-vigil-train crate in release mode.
RUN cargo build -p nexcore-vigil-train --release

# ─── Stage 2: runtime image ─────────────────────────────────────────────────
FROM debian:bookworm-slim

WORKDIR /app

# Minimal runtime deps (libssl for any TLS the app might do)
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy the built binary from the builder stage.
COPY --from=builder /build/target/release/nexcore-vigil-train /app/bin/nexcore-vigil-train

# The model + tokenizer are baked into the image. Copy them in before build:
#   cp -r ~/.claude/brain/vigil-qwen-v1 /path/to/this/context/model/
# (Or use gcsfuse to mount from GCS at runtime — see notes below.)
# For first deploy, assume model is in ./model/ relative to the build context.
COPY model/ /app/model/

# Cloud Run conventions: port via $PORT env var, defaults to 8080.
ENV PORT=8080
ENV RUST_LOG=info
EXPOSE 8080

# Healthcheck Cloud Run will poll.
HEALTHCHECK --interval=30s --timeout=3s --start-period=15s \
    CMD curl -f http://localhost:${PORT}/health || exit 1

# Entrypoint: the serve command, binding to 0.0.0.0:$PORT.
# Note: Cloud Run passes the port as $PORT; we bake the expansion into a shell form
# because the Rust binary needs the address in --bind, not env vars directly.
CMD /app/bin/nexcore-vigil-train \
    --tokenizer /app/model/tokenizer.json \
    serve \
    --bind "0.0.0.0:${PORT}" \
    --model-dir /app/model \
    --dtype f32 \
    --model-id vigil-qwen-v1
