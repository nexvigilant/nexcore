#!/usr/bin/env bash
# Build NexCore OS initramfs — no sudo required.
# Creates a self-contained initramfs with the nexcore-init binary
# and required shared libraries.
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"
INITRAMFS_DIR="$BUILD_DIR/initramfs"
BINARY="$HOME/.nexcore/target/release/nexcore-init"
OUTPUT="$BUILD_DIR/initramfs.cpio.gz"

echo "╔══════════════════════════════════════════╗"
echo "║   NexCore OS — Initramfs Builder         ║"
echo "╚══════════════════════════════════════════╝"
echo ""

# Verify binary exists
if [ ! -f "$BINARY" ]; then
    echo "ERROR: nexcore-init binary not found at $BINARY"
    echo "Build it first: cd ~/nexcore && cargo build -p nexcore-os-demo --release"
    exit 1
fi

echo "[1/5] Cleaning previous build..."
rm -rf "$INITRAMFS_DIR"
mkdir -p "$INITRAMFS_DIR"

echo "[2/5] Creating filesystem hierarchy..."
mkdir -p "$INITRAMFS_DIR"/{bin,sbin,lib,lib64,dev,proc,sys,tmp,etc,run,var/log,mnt,root}

echo "[3/5] Installing nexcore-init as /init..."
cp "$BINARY" "$INITRAMFS_DIR/init"
chmod 755 "$INITRAMFS_DIR/init"
# Also install as /sbin/init (fallback)
cp "$BINARY" "$INITRAMFS_DIR/sbin/init"
chmod 755 "$INITRAMFS_DIR/sbin/init"

echo "[4/5] Copying required shared libraries..."
# Extract library paths from ldd output
LIBS=$(ldd "$BINARY" 2>/dev/null | grep -oP '/[^\s]+' | sort -u)
for lib in $LIBS; do
    if [ -f "$lib" ]; then
        # Determine destination directory
        dir=$(dirname "$lib")
        mkdir -p "$INITRAMFS_DIR$dir"
        cp "$lib" "$INITRAMFS_DIR$lib"
        echo "  → $lib"
    fi
done

# Copy the dynamic linker explicitly
if [ -f /lib64/ld-linux-x86-64.so.2 ]; then
    mkdir -p "$INITRAMFS_DIR/lib64"
    cp /lib64/ld-linux-x86-64.so.2 "$INITRAMFS_DIR/lib64/"
    echo "  → /lib64/ld-linux-x86-64.so.2"
fi

# Add essential device nodes (these will be overridden by devtmpfs mount)
# But we need at least console for early boot output
echo "[4b/5] Creating minimal device nodes (for fallback)..."
# We can't mknod without sudo, but the kernel will mount devtmpfs
# Just create the directory structure

# Add a minimal /etc/hostname
echo "nexcore" > "$INITRAMFS_DIR/etc/hostname"

# Add /etc/os-release
cat > "$INITRAMFS_DIR/etc/os-release" << 'OSREL'
NAME="NexCore OS"
VERSION="1.0.0"
ID=nexcore
ID_LIKE=linux
VERSION_ID=1.0.0
PRETTY_NAME="NexCore OS 1.0.0 (Forge)"
HOME_URL="https://nexvigilant.com"
BUG_REPORT_URL="https://github.com/nexvigilant/nexcore/issues"
OSREL

echo "[5/5] Packing initramfs..."
cd "$INITRAMFS_DIR"
find . -print0 | cpio --null -ov --format=newc 2>/dev/null | gzip -9 > "$OUTPUT"
cd "$SCRIPT_DIR"

SIZE=$(du -h "$OUTPUT" | cut -f1)
echo ""
echo "════════════════════════════════════════════"
echo "  Initramfs built: $OUTPUT"
echo "  Size: $SIZE"
echo ""
echo "  Contents:"
echo "    /init             — NexCore OS init (PID 1)"
echo "    /sbin/init        — fallback init path"
echo "    /lib*/            — required shared libraries"
echo "    /etc/os-release   — NexCore OS identity"
echo "    /proc,/sys,/dev   — mount points (populated at boot)"
echo "════════════════════════════════════════════"
echo ""
echo "Next: Run ./build-usb.sh to create the bootable USB image"
