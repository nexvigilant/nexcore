# Stage 1: Cargo Chef Preparation
FROM lukemathwalker/cargo-chef:latest-rust-1.85-bookworm AS chef
WORKDIR /app

# Stage 2: Recipe Planner
FROM chef AS planner
COPY . .
RUN cargo chef prepare --recipe-path recipe.json

# Stage 3: Cacher (Builds dependencies only)
FROM chef AS cacher
COPY --from=planner /app/recipe.json recipe.json
RUN cargo chef cook --release --recipe-path recipe.json

# Stage 4: Builder
FROM chef AS builder
COPY . .
# Copy dependencies from cacher
COPY --from=cacher /app/target target
COPY --from=cacher /usr/local/cargo /usr/local/cargo
RUN cargo build --release --bin nexcore-mcp --bin nexcore-api

# Stage 5: Runtime
FROM debian:bookworm-slim AS runtime
WORKDIR /app
RUN apt-get update && apt-get install -y --no-install-recommends \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binaries
COPY --from=builder /app/target/release/nexcore-mcp /usr/local/bin/
COPY --from=builder /app/target/release/nexcore-api /usr/local/bin/

# Default port for nexcore-api
EXPOSE 3030

# Use nexcore-api as the default entry point
CMD ["nexcore-api"]