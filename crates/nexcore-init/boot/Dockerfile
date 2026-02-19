# NexCore OS — Docker PID 1 Boot Test
#
# Tests nexcore-init as PID 1 inside a minimal container.
# This validates the boot sequence, shell lifecycle, and shutdown
# without requiring a full kernel or QEMU.
#
# Usage:
#   docker build -t nexcore-os -f boot/Dockerfile .
#   docker run --rm nexcore-os                          # desktop, 10 ticks
#   docker run --rm nexcore-os --form-factor watch      # watch, 10 ticks
#   docker run --rm nexcore-os --form-factor phone      # phone, 10 ticks
#   docker run --rm nexcore-os --ticks 50               # longer run

# Stage 1: Build (uses workspace context)
FROM rust:1.85-bookworm AS builder

WORKDIR /build
COPY Cargo.toml Cargo.lock ./
COPY crates/ crates/
COPY nexcore-cli/ nexcore-cli/

# Build only nexcore-init in release mode
RUN cargo build --release -p nexcore-init 2>&1 \
    && strip target/release/nexcore-init

# Stage 2: Runtime (minimal — nexcore-init IS pid 1)
FROM debian:bookworm-slim

# Copy the init binary
COPY --from=builder /build/target/release/nexcore-init /sbin/init

# Default: desktop form factor, virtual platform, 10 ticks
ENTRYPOINT ["/sbin/init", "--virtual", "--ticks", "10"]
