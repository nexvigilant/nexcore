# NexCore Evaluator Container
# Extends the base Dockerfile with Python SDK + examples for zero-config evaluation.
#
# Usage:
#   docker build -f sdk/Dockerfile -t nexvigilant/nexcore .
#   docker run -p 3030:3030 nexvigilant/nexcore
#
# Then visit http://localhost:3030/docs for the Scalar API explorer.

# ── Stage 1-4: Rust build (reuse base Dockerfile) ───────

FROM lukemathwalker/cargo-chef:latest-rust-1.85-bookworm AS chef
WORKDIR /app

FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

FROM chef AS builder
COPY . .
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --bin nexcore-mcp --bin nexcore-api

# ── Stage 5: Runtime with Python SDK ────────────────────

FROM debian:bookworm-slim AS runtime
WORKDIR /app

RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    python3 \
    python3-pip \
    python3-venv \
    && rm -rf /var/lib/apt/lists/*

# Copy Rust binaries
COPY --from=builder /app/target/release/nexcore-mcp /usr/local/bin/
COPY --from=builder /app/target/release/nexcore-api /usr/local/bin/

# Install Python SDK
COPY sdk/python /app/sdk/python
RUN python3 -m venv /app/.venv \
    && /app/.venv/bin/pip install --no-cache-dir /app/sdk/python

# Copy examples to accessible location
COPY sdk/python/examples /app/examples

# No API_KEY = auth disabled (open mode for evaluation)
# No Firebase = MockPersistence fallback
# Scalar UI auto-served at /docs

EXPOSE 3030

# Entrypoint script: start API and print helpful info
COPY sdk/entrypoint.sh /app/entrypoint.sh
RUN chmod +x /app/entrypoint.sh

ENTRYPOINT ["/app/entrypoint.sh"]
