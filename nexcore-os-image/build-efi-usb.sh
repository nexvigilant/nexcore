#!/usr/bin/env bash
# NexCore OS — EFI Stub USB Builder (No GRUB Required)
# Creates a bootable USB using the kernel's built-in EFI stub.
# This is the simplest and most reliable method for UEFI systems.
#
# Usage:
#   sudo ./build-efi-usb.sh /dev/sdX
#
# Prerequisites: dosfstools (mkfs.vfat), parted
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"
INITRAMFS="$BUILD_DIR/initramfs.cpio.gz"
USB_DEV="${1:-}"

# Kernel selection
KERNEL=""
for k in /boot/vmlinuz-6.18* /boot/vmlinuz-6.17* /boot/vmlinuz-*; do
    if [ -f "$k" ]; then
        KERNEL="$k"
        break
    fi
done

echo "╔══════════════════════════════════════════════════╗"
echo "║   NexCore OS — EFI USB Builder (No Bootloader)   ║"
echo "╚══════════════════════════════════════════════════╝"
echo ""

# Checks
if [ "$(id -u)" -ne 0 ]; then
    echo "ERROR: Run with sudo"
    exit 1
fi

if [ -z "$USB_DEV" ]; then
    echo "Usage: sudo $0 /dev/sdX"
    echo ""
    echo "Available devices:"
    lsblk -d -o NAME,SIZE,MODEL,TYPE | grep disk
    exit 1
fi

if [ ! -b "$USB_DEV" ]; then
    echo "ERROR: $USB_DEV is not a block device"
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
echo "  Target:    $USB_DEV"
echo ""

# Safety confirmation
echo "⚠ WARNING: ALL DATA ON $USB_DEV WILL BE DESTROYED"
lsblk "$USB_DEV" 2>/dev/null
echo ""
read -rp "Type 'NEXCORE' to confirm: " confirm
if [ "$confirm" != "NEXCORE" ]; then
    echo "Aborted."
    exit 0
fi

echo ""
echo "[1/6] Unmounting existing partitions..."
umount "${USB_DEV}"* 2>/dev/null || true

echo "[2/6] Creating GPT partition table..."
parted -s "$USB_DEV" mklabel gpt
parted -s "$USB_DEV" mkpart ESP fat32 1MiB 512MiB
parted -s "$USB_DEV" set 1 esp on
parted -s "$USB_DEV" mkpart NexCoreData ext4 512MiB 100%

# Detect partition naming (nvme vs sd)
if [[ "$USB_DEV" == *nvme* ]] || [[ "$USB_DEV" == *loop* ]]; then
    PART1="${USB_DEV}p1"
    PART2="${USB_DEV}p2"
else
    PART1="${USB_DEV}1"
    PART2="${USB_DEV}2"
fi

# Wait for partitions to appear
sleep 2
partprobe "$USB_DEV" 2>/dev/null || true
sleep 1

echo "[3/6] Formatting partitions..."
mkfs.vfat -F32 -n "NEXCORE-EFI" "$PART1"
mkfs.ext4 -L "NEXCORE-DATA" "$PART2" -q 2>/dev/null || {
    echo "  ext4 formatting skipped (mkfs.ext4 not available)"
    echo "  Data partition will need manual formatting"
}

echo "[4/6] Mounting EFI partition..."
MOUNT_DIR=$(mktemp -d)
mount "$PART1" "$MOUNT_DIR"

echo "[5/6] Installing NexCore OS..."
# EFI boot structure
mkdir -p "$MOUNT_DIR/EFI/BOOT"
mkdir -p "$MOUNT_DIR/EFI/nexcore"
mkdir -p "$MOUNT_DIR/boot"

# Copy kernel as EFI bootloader (Linux kernel has built-in EFI stub)
cp "$KERNEL" "$MOUNT_DIR/EFI/BOOT/BOOTX64.EFI"
cp "$KERNEL" "$MOUNT_DIR/EFI/nexcore/vmlinuz"

# Copy initramfs
cp "$INITRAMFS" "$MOUNT_DIR/EFI/nexcore/initramfs.cpio.gz"
cp "$INITRAMFS" "$MOUNT_DIR/boot/initramfs.cpio.gz"

# Create startup.nsh for EFI shell fallback
cat > "$MOUNT_DIR/startup.nsh" << 'NSH'
@echo -off
echo NexCore OS 1.0.0 — Booting...
\EFI\nexcore\vmlinuz initrd=\EFI\nexcore\initramfs.cpio.gz console=tty0 quiet loglevel=3
NSH

# Create GRUB config as well (if GRUB is present in UEFI firmware)
mkdir -p "$MOUNT_DIR/boot/grub"
cat > "$MOUNT_DIR/boot/grub/grub.cfg" << 'GRUBCFG'
set timeout=3
set default=0
set color_normal=white/black
set color_highlight=cyan/black

menuentry "NexCore OS 1.0.0" {
    linux /EFI/nexcore/vmlinuz console=tty0 quiet loglevel=3
    initrd /EFI/nexcore/initramfs.cpio.gz
}

menuentry "NexCore OS 1.0.0 (Verbose)" {
    linux /EFI/nexcore/vmlinuz console=tty0 loglevel=7
    initrd /EFI/nexcore/initramfs.cpio.gz
}
GRUBCFG

# Also try installing GRUB if available
if command -v grub-install &>/dev/null; then
    echo "  Installing GRUB EFI bootloader..."
    grub-install --target=x86_64-efi \
        --efi-directory="$MOUNT_DIR" \
        --boot-directory="$MOUNT_DIR/boot" \
        --removable \
        --no-nvram \
        2>/dev/null && echo "  → GRUB installed (UEFI)" || echo "  → GRUB install skipped"
fi

# Create NexCore OS identification
cat > "$MOUNT_DIR/NEXCORE_OS" << 'INFO'
╔══════════════════════════════════════╗
║         NexCore OS 1.0.0             ║
║   © 2026 NexVigilant                 ║
║   100% Rust · Immune-System Design   ║
╚══════════════════════════════════════╝

Boot: EFI/BOOT/BOOTX64.EFI
Kernel: Linux 6.x
Init: /init (nexcore-init, PID 1)
INFO

echo "[6/6] Syncing and unmounting..."
sync
umount "$MOUNT_DIR"
rmdir "$MOUNT_DIR"

TOTAL_SIZE=$(du -sh "$MOUNT_DIR" 2>/dev/null | cut -f1 || echo "~15MB")

echo ""
echo "═══════════════════════════════════════════════════"
echo "  ✓ NexCore OS USB drive ready!"
echo ""
echo "  Device: $USB_DEV"
echo "  Partition 1: EFI System (512MB FAT32)"
echo "  Partition 2: NexCore Data (remaining, ext4)"
echo ""
echo "  Boot Instructions (Lenovo):"
echo "    1. Insert USB into Lenovo laptop"
echo "    2. Power on → Press F12 for boot menu"
echo "         (or F2/Fn+F2 for BIOS/UEFI setup)"
echo "    3. Select the USB drive"
echo "    4. If Secure Boot blocks it:"
echo "         → Enter BIOS (F2)"
echo "         → Security → Secure Boot → Disable"
echo "         → Save & Exit → Boot from USB"
echo ""
echo "  First Boot:"
echo "    → You'll see the NexCore splash screen"
echo "    → Create your owner account"
echo "    → Log in to the NexCore shell"
echo "═══════════════════════════════════════════════════"
