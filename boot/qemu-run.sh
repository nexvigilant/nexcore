#!/usr/bin/env bash
# NexCore OS — QEMU Boot Script
#
# Boots NexCore OS in QEMU using a Linux kernel + our initramfs.
# nexcore-init runs as PID 1.
#
# Prerequisites:
#   sudo apt install qemu-system-x86
#   ./boot/build-initramfs.sh  (builds the initramfs)
#   A Linux kernel image (bzImage) — see below for how to get one
#
# Usage:
#   ./boot/qemu-run.sh                    # Boot with defaults
#   ./boot/qemu-run.sh --kernel /path/to/bzImage
#   ./boot/qemu-run.sh --ram 512          # RAM in MB
#   ./boot/qemu-run.sh --graphic          # Enable VGA display
#
# Getting a kernel:
#   # Option 1: Use host kernel
#   cp /boot/vmlinuz-$(uname -r) boot/bzImage
#
#   # Option 2: Download minimal kernel
#   curl -L -o boot/bzImage https://github.com/cirosantilli/linux-kernel-module-cheat/releases/download/v1.0/bzImage

set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
INITRAMFS="$SCRIPT_DIR/initramfs.cpio.gz"
KERNEL="$SCRIPT_DIR/bzImage"
RAM="256"
GRAPHIC=""
EXTRA_ARGS=""

while [[ $# -gt 0 ]]; do
    case $1 in
        --kernel|-k)  KERNEL="$2"; shift 2 ;;
        --ram|-m)     RAM="$2"; shift 2 ;;
        --graphic|-g) GRAPHIC="yes"; shift ;;
        --help|-h)
            echo "NexCore OS — QEMU Boot"
            echo ""
            echo "Usage: $0 [OPTIONS]"
            echo ""
            echo "Options:"
            echo "  -k, --kernel PATH   Linux kernel image (default: boot/bzImage)"
            echo "  -m, --ram MB        RAM allocation (default: 256)"
            echo "  -g, --graphic       Enable VGA display (default: serial-only)"
            echo "  -h, --help          Show this help"
            exit 0
            ;;
        *) EXTRA_ARGS="$EXTRA_ARGS $1"; shift ;;
    esac
done

# Validate prerequisites
if [ ! -f "$INITRAMFS" ]; then
    echo "ERROR: initramfs not found at $INITRAMFS"
    echo "  Run: ./boot/build-initramfs.sh"
    exit 1
fi

if [ ! -f "$KERNEL" ]; then
    echo "ERROR: Kernel image not found at $KERNEL"
    echo ""
    echo "  Get a kernel with one of:"
    echo "    cp /boot/vmlinuz-\$(uname -r) $KERNEL"
    echo "    # or download a minimal kernel"
    exit 1
fi

if ! command -v qemu-system-x86_64 &>/dev/null; then
    echo "ERROR: qemu-system-x86_64 not found"
    echo "  Install: sudo apt install qemu-system-x86"
    exit 1
fi

echo "=== NexCore OS — QEMU Boot ==="
echo "  Kernel:   $KERNEL"
echo "  Initramfs: $INITRAMFS ($(du -h "$INITRAMFS" | cut -f1))"
echo "  RAM:      ${RAM}M"
echo ""

# Build QEMU command
QEMU_CMD=(
    qemu-system-x86_64
    -kernel "$KERNEL"
    -initrd "$INITRAMFS"
    -m "${RAM}M"
    -append "console=ttyS0 quiet loglevel=3"
    -no-reboot
)

if [ -z "$GRAPHIC" ]; then
    # Serial-only mode (output to terminal)
    QEMU_CMD+=(-nographic)
else
    # VGA display mode
    QEMU_CMD+=(-vga std)
fi

echo ">>> ${QEMU_CMD[*]}"
echo ""
exec "${QEMU_CMD[@]}" $EXTRA_ARGS
