#!/usr/bin/env bash
# install.sh — build + install vigil-nexdev as a user systemd service on nexdev.
# Idempotent. Run as the `matthew` user on the nexdev VM.
set -euo pipefail

CRATE_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
WORKSPACE_DIR="$(cd "$CRATE_DIR/../.." && pwd)"
UNIT_SRC="$CRATE_DIR/systemd/vigil-nexdev.service"
UNIT_DIR="$HOME/.config/systemd/user"
UNIT_DST="$UNIT_DIR/vigil-nexdev.service"

echo "== vigil-nexdev install =="
echo "crate: $CRATE_DIR"
echo "workspace: $WORKSPACE_DIR"

# 1) Build release binary.
echo "-> cargo build -p vigil-nexdev --release"
cd "$WORKSPACE_DIR"
cargo build -p vigil-nexdev --release

# 2) Install user systemd unit.
mkdir -p "$UNIT_DIR"
cp "$UNIT_SRC" "$UNIT_DST"
echo "-> installed $UNIT_DST"

# 3) Reload + enable + (re)start.
systemctl --user daemon-reload
systemctl --user enable vigil-nexdev.service
systemctl --user restart vigil-nexdev.service

# 4) Persist across logout. Requires loginctl enable-linger (one-time).
if ! loginctl show-user "$USER" --property=Linger 2>/dev/null | grep -q "Linger=yes"; then
    echo "-> enabling lingering for $USER (requires sudo)"
    sudo loginctl enable-linger "$USER"
fi

sleep 1
echo
echo "== status =="
systemctl --user status vigil-nexdev.service --no-pager | head -15
echo
echo "== smoke test =="
curl -sS http://127.0.0.1:7823/health | python3 -m json.tool || echo "(health endpoint not yet responding — check journalctl)"
