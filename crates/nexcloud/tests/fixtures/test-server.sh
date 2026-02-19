#!/usr/bin/env zsh
# Tiny HTTP server for nexcloud integration testing.
# Responds to any request with 200 OK on the PORT env var.
set -euo pipefail

PORT="${PORT:-8080}"

# Use socat for a simple HTTP responder
while true; do
    echo -e "HTTP/1.1 200 OK\r\nContent-Type: text/plain\r\nContent-Length: 2\r\n\r\nok" | \
        socat - TCP-LISTEN:"${PORT}",reuseaddr,fork 2>/dev/null || true
done
