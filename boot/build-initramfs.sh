#!/usr/bin/env bash
# NexCore OS — Build initramfs for QEMU boot
#
# Creates a minimal initramfs (cpio archive) containing nexcore-init
# as /init (PID 1). The binary must be statically linked.
#
# Prerequisites:
#   rustup target add x86_64-unknown-linux-musl
#   sudo apt install musl-tools
#
# Usage:
#   ./boot/build-initramfs.sh [--form-factor watch|phone|desktop] [--ticks N]
#
# Output:
#   boot/initramfs.cpio.gz

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
WORKSPACE_DIR="$(dirname "$SCRIPT_DIR")"
INITRAMFS_DIR="$SCRIPT_DIR/initramfs-root"
OUTPUT="$SCRIPT_DIR/initramfs.cpio.gz"

# Default init args
FORM_FACTOR="desktop"
TICKS="10"

while [[ $# -gt 0 ]]; do
    case $1 in
        --form-factor|-f) FORM_FACTOR="$2"; shift 2 ;;
        --ticks|-t)       TICKS="$2"; shift 2 ;;
        --help|-h)
            echo "Usage: $0 [--form-factor watch|phone|desktop] [--ticks N]"
            exit 0
            ;;
        *) echo "Unknown arg: $1"; exit 1 ;;
    esac
done

echo "=== NexCore OS — Building initramfs ==="
echo "  Form factor: $FORM_FACTOR"
echo "  Ticks:       $TICKS"
echo ""

# Step 1: Build statically-linked nexcore-init
echo ">>> Building nexcore-init (musl, static)..."
cd "$WORKSPACE_DIR"

if ! rustup target list --installed | grep -q x86_64-unknown-linux-musl; then
    echo "ERROR: x86_64-unknown-linux-musl target not installed."
    echo "  Run: rustup target add x86_64-unknown-linux-musl"
    echo "  And: sudo apt install musl-tools"
    exit 1
fi

RUSTFLAGS="-C target-feature=+crt-static" \
    cargo build --release --target x86_64-unknown-linux-musl -p nexcore-init 2>&1

INIT_BIN="$WORKSPACE_DIR/target/x86_64-unknown-linux-musl/release/nexcore-init"

if [ ! -f "$INIT_BIN" ]; then
    echo "ERROR: Binary not found at $INIT_BIN"
    exit 1
fi

echo "  Binary: $(du -h "$INIT_BIN" | cut -f1)"
echo "  Linkage: $(file "$INIT_BIN" | grep -o 'statically linked' || echo 'dynamically linked (WARNING)')"

# Step 2: Create initramfs directory structure
echo ""
echo ">>> Creating initramfs root..."
rm -rf "$INITRAMFS_DIR"
mkdir -p "$INITRAMFS_DIR"/{dev,proc,sys,tmp,var/lib/nexcore}

# /init — the kernel exec's this as PID 1
# We wrap nexcore-init in a shell script that mounts essential filesystems
cat > "$INITRAMFS_DIR/init" << 'INIT_SCRIPT'
#!/bin/sh
# NexCore OS init wrapper — mount virtual filesystems then exec nexcore-init

# Mount essential virtual filesystems
mount -t proc proc /proc 2>/dev/null
mount -t sysfs sysfs /sys 2>/dev/null
mount -t devtmpfs devtmpfs /dev 2>/dev/null

# Print banner
echo ""
echo "  ╔══════════════════════════════════════╗"
echo "  ║       NexCore OS — Booting...        ║"
echo "  ║  100% Rust  •  Primitive-Grounded    ║"
echo "  ╚══════════════════════════════════════╝"
echo ""

# Exec the real init (replaces this shell process)
INIT_SCRIPT

# Append the actual exec line with form factor and ticks
echo "exec /sbin/nexcore-init --virtual --form-factor $FORM_FACTOR --ticks $TICKS" >> "$INITRAMFS_DIR/init"
chmod +x "$INITRAMFS_DIR/init"

# Copy the binary
cp "$INIT_BIN" "$INITRAMFS_DIR/sbin/nexcore-init"
mkdir -p "$INITRAMFS_DIR/sbin"
cp "$INIT_BIN" "$INITRAMFS_DIR/sbin/nexcore-init"

# Step 3: Build cpio archive
echo ""
echo ">>> Packing initramfs..."
cd "$INITRAMFS_DIR"
find . | cpio -o -H newc --quiet | gzip > "$OUTPUT"
cd "$SCRIPT_DIR"

echo "  Output: $OUTPUT ($(du -h "$OUTPUT" | cut -f1))"

# Cleanup
rm -rf "$INITRAMFS_DIR"

echo ""
echo "=== initramfs built successfully ==="
echo ""
echo "To boot with QEMU:"
echo "  ./boot/qemu-run.sh"
