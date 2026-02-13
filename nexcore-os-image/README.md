# NexCore OS — Bootable Image Builder

Build and flash NexCore OS to a USB drive for booting on x86_64 hardware.

## Quick Start (3 Steps)

```bash
# 1. Build initramfs (no sudo needed)
./build-initramfs.sh

# 2. Test in QEMU (optional but recommended)
sudo apt-get install -y qemu-system-x86
./test-qemu.sh

# 3. Flash to USB (requires sudo)
# Option A: EFI direct (simplest, for modern UEFI systems like Lenovo)
sudo ./build-efi-usb.sh /dev/sdX

# Option B: ISO with GRUB (BIOS + UEFI hybrid)
sudo ./build-usb.sh /dev/sdX
```

## What Gets Built

| Component | Size | Source |
|-----------|------|--------|
| Linux kernel | ~14MB | `/boot/vmlinuz-6.x` (host kernel) |
| NexCore initramfs | ~2.5MB | `nexcore-init` + glibc |
| **Total** | **~17MB** | |

## Boot Sequence

```
UEFI firmware → GRUB/EFI stub → Linux kernel → nexcore-init (PID 1)
                                                    │
                                    ┌───────────────┼───────────────┐
                                    │               │               │
                              Mount /proc      Boot STOS      Start Guardian
                              Mount /sys       Start IPC      Start Brain
                              Mount /dev       Start Vault    Start Shell
                                    │               │               │
                                    └───────────────┼───────────────┘
                                                    │
                                            First-Boot Setup
                                            (Create owner account)
                                                    │
                                              Login Prompt
                                                    │
                                            NexCore Shell ▸_
```

## Lenovo Boot Instructions

1. Insert USB into Lenovo laptop
2. Power on → **F12** for one-time boot menu
3. Select the USB drive (may show as "EFI USB Device")
4. If Secure Boot blocks it:
   - Enter BIOS: **F2** (or **Fn+F2**)
   - Navigate: Security → Secure Boot → **Disable**
   - Save & Exit → Boot from USB

## Shell Commands

Once logged in, available commands:

| Command | Description |
|---------|-------------|
| `status` | System overview |
| `services` | List all running services |
| `security` | Guardian threat status |
| `users` | User account list |
| `sessions` | Active login sessions |
| `vault` | Secure storage status |
| `boot` | Secure boot chain info |
| `uptime` | System uptime |
| `logout` | Return to login screen |
| `shutdown` | Power off |
| `reboot` | Restart |

## Files

```
nexcore-os-image/
├── build-initramfs.sh    # Step 1: Build initramfs (no sudo)
├── build-efi-usb.sh      # Step 3a: EFI USB (sudo, for UEFI systems)
├── build-usb.sh           # Step 3b: ISO USB (sudo, BIOS+UEFI)
├── test-qemu.sh           # Step 2: QEMU test (optional)
├── README.md              # This file
└── build/
    ├── initramfs.cpio.gz  # Built initramfs (~2.5MB)
    ├── initramfs/         # Unpacked initramfs tree
    └── nexcore-os.iso     # Built ISO (if using build-usb.sh)
```
