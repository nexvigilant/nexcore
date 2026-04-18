FROM rust:1.85-bookworm AS chef
RUN cargo install cargo-chef --locked
WORKDIR /build

FROM chef AS planner
COPY . .
RUN cargo chef prepare --bin nexcore-vault-api --recipe-path recipe.json

FROM chef AS builder
COPY --from=planner /build/recipe.json recipe.json
RUN cargo chef cook --release --bin nexcore-vault-api --recipe-path recipe.json
COPY . .
RUN cargo build --release --bin nexcore-vault-api -p nexcore-vault-api

FROM debian:bookworm-slim AS runtime
RUN apt-get update -qq && apt-get install -y -qq --no-install-recommends \
    ca-certificates libssl3 && rm -rf /var/lib/apt/lists/*
WORKDIR /app
COPY --from=builder /build/target/release/nexcore-vault-api /app/nexcore-vault-api
ENV PORT=8080 RUST_LOG=info VAULT_ROOT=/vault
EXPOSE 8080
CMD ["/app/nexcore-vault-api"]
