#!/usr/bin/env bash
# NexCore OS — Bootable USB Image Builder
# Creates a UEFI+BIOS bootable USB image with GRUB.
#
# Usage:
#   sudo ./build-usb.sh              # Build ISO image
#   sudo ./build-usb.sh /dev/sdX     # Write directly to USB device
#
# Prerequisites (installed automatically):
#   grub-pc-bin grub-efi-amd64-bin grub-common xorriso mtools
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
BUILD_DIR="$SCRIPT_DIR/build"
INITRAMFS="$BUILD_DIR/initramfs.cpio.gz"
ISO_DIR="$BUILD_DIR/iso"
OUTPUT_ISO="$BUILD_DIR/nexcore-os.iso"
USB_DEV="${1:-}"

# Kernel selection: prefer latest
KERNEL=""
for k in /boot/vmlinuz-6.18* /boot/vmlinuz-6.17* /boot/vmlinuz-*; do
    if [ -f "$k" ]; then
        KERNEL="$k"
        break
    fi
done

echo "╔══════════════════════════════════════════════╗"
echo "║   NexCore OS — Bootable USB Image Builder    ║"
echo "╚══════════════════════════════════════════════╝"
echo ""

# Check root
if [ "$(id -u)" -ne 0 ]; then
    echo "ERROR: This script must be run as root (sudo)"
    echo "Usage: sudo $0 [/dev/sdX]"
    exit 1
fi

# Check initramfs
if [ ! -f "$INITRAMFS" ]; then
    echo "ERROR: Initramfs not found at $INITRAMFS"
    echo "Run ./build-initramfs.sh first (no sudo needed)"
    exit 1
fi

# Check kernel
if [ -z "$KERNEL" ]; then
    echo "ERROR: No Linux kernel found in /boot/"
    exit 1
fi
echo "  Kernel:    $KERNEL"
echo "  Initramfs: $INITRAMFS"
echo ""

# Install dependencies
echo "[1/6] Installing build dependencies..."
apt-get install -y grub-pc-bin grub-efi-amd64-bin grub-common xorriso mtools 2>/dev/null || {
    echo "WARNING: Could not install all dependencies. Trying with what's available..."
}

# Create ISO directory structure
echo "[2/6] Creating ISO filesystem..."
rm -rf "$ISO_DIR"
mkdir -p "$ISO_DIR/boot/grub"

# Copy kernel and initramfs
echo "[3/6] Installing kernel and initramfs..."
cp "$KERNEL" "$ISO_DIR/boot/vmlinuz"
cp "$INITRAMFS" "$ISO_DIR/boot/initramfs.cpio.gz"
echo "  → $(du -h "$ISO_DIR/boot/vmlinuz" | cut -f1) kernel"
echo "  → $(du -h "$ISO_DIR/boot/initramfs.cpio.gz" | cut -f1) initramfs"

# Create GRUB config
echo "[4/6] Writing GRUB configuration..."
cat > "$ISO_DIR/boot/grub/grub.cfg" << 'GRUBCFG'
set timeout=3
set default=0

# NexCore OS color scheme
set color_normal=white/black
set color_highlight=cyan/black

menuentry "NexCore OS 1.0.0" {
    linux /boot/vmlinuz console=tty0 quiet loglevel=3
    initrd /boot/initramfs.cpio.gz
}

menuentry "NexCore OS 1.0.0 (Verbose Boot)" {
    linux /boot/vmlinuz console=tty0 loglevel=7
    initrd /boot/initramfs.cpio.gz
}

menuentry "NexCore OS 1.0.0 (Recovery - Single User)" {
    linux /boot/vmlinuz console=tty0 single
    initrd /boot/initramfs.cpio.gz
}

menuentry "Reboot" {
    reboot
}

menuentry "Shutdown" {
    halt
}
GRUBCFG

# Build ISO with GRUB (BIOS + UEFI hybrid)
echo "[5/6] Building bootable ISO..."
if command -v grub-mkrescue &>/dev/null; then
    grub-mkrescue -o "$OUTPUT_ISO" "$ISO_DIR" 2>&1 | tail -5
elif command -v grub-mkimage &>/dev/null; then
    # Fallback: manual GRUB image creation
    echo "  Using manual GRUB image creation..."

    # Create BIOS boot image
    grub-mkimage -O i386-pc -o "$BUILD_DIR/core.img" \
        -p /boot/grub \
        biosdisk iso9660 normal search linux initrd \
        2>/dev/null || echo "  BIOS image skipped (missing modules)"

    # Try xorriso directly
    if command -v xorriso &>/dev/null; then
        xorriso -as mkisofs \
            -R -J -V "NEXCORE_OS" \
            -b boot/grub/i386-pc/eltorito.img \
            -no-emul-boot -boot-load-size 4 -boot-info-table \
            -o "$OUTPUT_ISO" "$ISO_DIR" 2>/dev/null || {
            # Simple fallback: just create a raw image
            echo "  Falling back to raw disk image..."
            dd if=/dev/zero of="$OUTPUT_ISO" bs=1M count=256 2>/dev/null
            # Create partition table
            parted -s "$OUTPUT_ISO" mklabel msdos 2>/dev/null || true
            parted -s "$OUTPUT_ISO" mkpart primary fat32 1MiB 100% 2>/dev/null || true
            echo "  Raw image created (needs manual GRUB install)"
        }
    fi
else
    echo "ERROR: Neither grub-mkrescue nor grub-mkimage found"
    echo "Install: sudo apt-get install grub-pc-bin grub-efi-amd64-bin grub-common"
    exit 1
fi

if [ -f "$OUTPUT_ISO" ]; then
    ISO_SIZE=$(du -h "$OUTPUT_ISO" | cut -f1)
    echo ""
    echo "════════════════════════════════════════════════"
    echo "  ISO image built: $OUTPUT_ISO"
    echo "  Size: $ISO_SIZE"
    echo "════════════════════════════════════════════════"
fi

# Write to USB if device specified
if [ -n "$USB_DEV" ]; then
    echo ""
    echo "[6/6] Writing to USB device: $USB_DEV"

    # Safety check
    if [ ! -b "$USB_DEV" ]; then
        echo "ERROR: $USB_DEV is not a block device"
        exit 1
    fi

    # Confirm
    echo ""
    echo "  ⚠ WARNING: This will ERASE ALL DATA on $USB_DEV"
    echo "  Device info:"
    lsblk "$USB_DEV" 2>/dev/null || true
    echo ""
    read -rp "  Type 'YES' to proceed: " confirm
    if [ "$confirm" != "YES" ]; then
        echo "  Aborted."
        exit 0
    fi

    echo "  Writing ISO to $USB_DEV..."
    dd if="$OUTPUT_ISO" of="$USB_DEV" bs=4M status=progress conv=fdatasync
    sync

    echo ""
    echo "════════════════════════════════════════════════"
    echo "  USB drive ready!"
    echo "  Insert into Lenovo laptop and boot from USB."
    echo ""
    echo "  BIOS Setup:"
    echo "    1. Power on → F12 (or F2 for BIOS)"
    echo "    2. Select USB drive from boot menu"
    echo "    3. If Secure Boot enabled, disable it first"
    echo "════════════════════════════════════════════════"
else
    echo ""
    echo "To write to USB:"
    echo "  sudo dd if=$OUTPUT_ISO of=/dev/sdX bs=4M status=progress conv=fdatasync"
    echo "  (Replace /dev/sdX with your USB device — check with 'lsblk')"
    echo ""
    echo "To test in QEMU:"
    echo "  qemu-system-x86_64 -m 512 -cdrom $OUTPUT_ISO -boot d"
fi
