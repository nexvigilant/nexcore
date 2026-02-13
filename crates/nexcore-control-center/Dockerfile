# Build stage
FROM rustlang/rust:nightly-slim AS builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y \
    pkg-config \
    libssl-dev \
    nodejs \
    npm \
    && rm -rf /var/lib/apt/lists/*

# Copy Cargo files
COPY Cargo.toml Cargo.lock ./

# Copy source
COPY src ./src
COPY tailwind.config.js input.css ./

# Install Tailwind and build CSS
RUN npm install -D tailwindcss@3
RUN mkdir -p style && ./node_modules/.bin/tailwindcss -i input.css -o style/output.css --minify
RUN cp style/output.css style/style.css

# Build release binary
RUN cargo build --release --features ssr

# Runtime stage
FROM debian:bookworm-slim

WORKDIR /app

# Install runtime dependencies
RUN apt-get update && apt-get install -y \
    ca-certificates \
    libssl3 \
    && rm -rf /var/lib/apt/lists/*

# Copy binary and assets
COPY --from=builder /app/target/release/nexcore-control-center ./
COPY --from=builder /app/style ./style
COPY --from=builder /app/Cargo.toml ./

# Configure for Cloud Run
ENV PORT=8080
ENV LEPTOS_SITE_ADDR=0.0.0.0:8080

EXPOSE 8080

CMD ["./nexcore-control-center"]
