#!/usr/bin/env bash
# NexCore OS — QEMU Test Boot
# Tests the initramfs with a Linux kernel in QEMU before flashing to USB.
#
# Usage:
#   ./test-qemu.sh          # BIOS mode (default)
#   ./test-qemu.sh --efi    # UEFI mode (requires OVMF)
#
# Install QEMU first:
#   sudo apt-get install -y qemu-system-x86 ovmf
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"
INITRAMFS="$BUILD_DIR/initramfs.cpio.gz"
MODE="${1:-bios}"

# Kernel selection
KERNEL=""
for k in /boot/vmlinuz-6.18* /boot/vmlinuz-6.17* /boot/vmlinuz-*; do
    if [ -f "$k" ]; then
        KERNEL="$k"
        break
    fi
done

echo "╔══════════════════════════════════════════╗"
echo "║   NexCore OS — QEMU Test Boot            ║"
echo "╚══════════════════════════════════════════╝"
echo ""

if ! command -v qemu-system-x86_64 &>/dev/null; then
    echo "ERROR: QEMU not installed"
    echo "Install: sudo apt-get install -y qemu-system-x86"
    exit 1
fi

if [ ! -f "$INITRAMFS" ]; then
    echo "ERROR: Run ./build-initramfs.sh first"
    exit 1
fi

if [ -z "$KERNEL" ]; then
    echo "ERROR: No kernel found in /boot/"
    exit 1
fi

echo "  Kernel:    $KERNEL"
echo "  Initramfs: $INITRAMFS"
echo "  Mode:      $MODE"
echo ""

QEMU_ARGS=(
    -m 512M
    -kernel "$KERNEL"
    -initrd "$INITRAMFS"
    -append "console=ttyS0 console=tty0 loglevel=3"
    -nographic
    -serial mon:stdio
    -no-reboot
)

if [ "$MODE" = "--efi" ] || [ "$MODE" = "efi" ]; then
    OVMF="/usr/share/OVMF/OVMF_CODE.fd"
    if [ ! -f "$OVMF" ]; then
        echo "ERROR: OVMF not found. Install: sudo apt-get install ovmf"
        exit 1
    fi
    QEMU_ARGS+=(-bios "$OVMF")
    echo "  Using UEFI firmware (OVMF)"
fi

echo "  Starting QEMU... (Ctrl+A then X to quit)"
echo ""
echo "════════════════════════════════════════════"

qemu-system-x86_64 "${QEMU_ARGS[@]}"
