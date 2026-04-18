# Multi-stage: builder with full workspace, runtime slim.
# Uses cargo-chef for layer caching.
FROM rust:1.85-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /build

FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin nexcore-brain-api --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --bin nexcore-brain-api --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin nexcore-brain-api -p nexcore-brain-api

FROM debian:bookworm-slim AS runtime
RUN apt-get update -qq && apt-get install -y -qq --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/target/release/nexcore-brain-api /app/nexcore-brain-api
COPY crates/nexcore-brain-api/schema.sql /app/schema.sql
ENV PORT=8080 RUST_LOG=info
EXPOSE 8080
CMD ["/app/nexcore-brain-api"]
