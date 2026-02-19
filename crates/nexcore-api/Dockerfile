# NexVigilant Core API — Production Container
# Multi-stage: build on host, copy binary into minimal runtime
#
# Usage:
#   docker build -t nexcore-api .
#   docker run -p 8080:8080 -e PORT=8080 -e BIND_ADDR=0.0.0.0 nexcore-api
#
# Cloud Run:
#   gcloud run deploy nexcore-api --source . --region us-central1

# ── Stage 1: Build (only if binary not pre-built) ──
FROM rust:1.85-slim-bookworm AS builder

RUN apt-get update && apt-get install -y --no-install-recommends \
    pkg-config libssl-dev ca-certificates \
    && rm -rf /var/lib/apt/lists/*

WORKDIR /build

# Copy the entire crates directory for dependency resolution
COPY . /build/nexcore-api/
WORKDIR /build/nexcore-api

# Build release binary
RUN --mount=type=cache,target=/usr/local/cargo/registry \
    --mount=type=cache,target=/build/nexcore-api/target \
    cargo build --release && \
    cp target/release/nexcore-api /build/nexcore-api-bin

# ── Stage 2: Runtime (minimal) ──
FROM debian:bookworm-slim AS runtime

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates libssl3 curl \
    && rm -rf /var/lib/apt/lists/*

# Non-root user
RUN groupadd --gid 1000 appuser && \
    useradd --uid 1000 --gid 1000 --create-home appuser

WORKDIR /app

# Copy binary from builder
COPY --from=builder /build/nexcore-api-bin /app/nexcore-api

RUN chmod +x /app/nexcore-api

USER appuser

# Cloud Run expects PORT env var (default 8080)
ENV PORT=8080
ENV BIND_ADDR=0.0.0.0
ENV RUST_LOG=nexcore_api=info,tower_http=info

EXPOSE 8080

HEALTHCHECK --interval=30s --timeout=5s --start-period=5s --retries=3 \
    CMD curl -f http://localhost:${PORT}/health || exit 1

ENTRYPOINT ["/app/nexcore-api"]
