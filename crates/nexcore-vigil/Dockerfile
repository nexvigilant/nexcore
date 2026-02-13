# Build stage
FROM rust:1.77-slim-bookworm as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    libasound2-dev \
    libclang-dev \
    cmake \
    && rm -rf /var/lib/apt/lists/*

COPY . .

RUN cargo build --release

# Final stage
FROM debian:bookworm-slim

WORKDIR /app

RUN apt-get update && apt-get install -y \
    libssl3 \
    libasound2 \
    espeak \
    git \
    && rm -rf /var/lib/apt/lists/*

COPY --from:builder /app/target/release/friday-rs /usr/local/bin/friday

ENV RUST_LOG=info

ENTRYPOINT ["friday"]
CMD ["run"]
