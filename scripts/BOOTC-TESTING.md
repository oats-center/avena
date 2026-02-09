# Bootc-based Testing Harness

Production-like testing using bootc images and systemd-nspawn. Creates immutable, bootable disk images identical to what would run on actual devices.

## Prerequisites

- Root access
- Fedora host with systemd-nspawn and podman
- Built binaries: `cargo build`

## Quick Start

```bash
cargo build

sudo ./scripts/bootc-harness.sh build

sudo ./scripts/bootc-harness.sh up 3

sudo ./scripts/bootc-harness.sh exec dev1

sudo ./scripts/bootc-harness.sh down
```

## How It Works

1. **Build Phase**: Creates a bootc container image containing avenad/avenactl, then uses bootc-image-builder to generate a raw disk image
2. **Boot Phase**: Spawns ephemeral nspawn containers from the disk image with --ephemeral (each instance gets a temporary copy-on-write overlay)
3. **Network**: Bridge network (10.42.42.0/24) connects all devices

## Commands

### Build Image

```bash
sudo ./scripts/bootc-harness.sh build
```

Builds the bootc container image and creates a bootable raw disk. Run this after rebuilding avenad/avenactl. Uses debug builds by default.

For release builds:
```bash
BUILD_PROFILE=release sudo ./scripts/bootc-harness.sh build
```

### Start Devices

```bash
sudo ./scripts/bootc-harness.sh up <count>
```

Boots N ephemeral instances from the disk image. Each instance is isolated with COW storage.

### Execute Commands

```bash
sudo ./scripts/bootc-harness.sh exec dev1
sudo ./scripts/bootc-harness.sh exec dev1 avenactl devices list
sudo ./scripts/bootc-harness.sh exec dev1 journalctl -u avenad -f
```

### Check Status

```bash
sudo ./scripts/bootc-harness.sh ip dev1
sudo ./scripts/bootc-harness.sh logs dev1
```

### Cleanup

```bash
sudo ./scripts/bootc-harness.sh down
```

## Key Differences from nspawn-harness.sh

| Feature | Old (nspawn-harness) | New (bootc-harness) |
|---------|---------------------|-------------------|
| Image type | Extracted container rootfs | Bootable disk image |
| Binary updates | Copy files into running containers | Rebuild entire image |
| Reproducibility | Manual dnf installs | Declarative Containerfile |
| Storage | Persistent directories | Ephemeral COW overlays |
| Production parity | Low | High |

## Workflow for Development

```bash
vim avena/src/...

cargo build

sudo ./scripts/bootc-harness.sh down

sudo ./scripts/bootc-harness.sh build

sudo ./scripts/bootc-harness.sh up 3

sudo ./scripts/bootc-harness.sh exec dev1 journalctl -u avenad -f
```

## Artifacts

- Container image: `localhost/avena-device:latest`
- Disk image: `/var/tmp/avena-bootc/images/avena-device.raw`
- Each running instance uses ephemeral COW storage (deleted on stop)

## Notes

- The --ephemeral flag means each device starts fresh from the base image
- Changes inside containers are lost on restart (production-like)
- Build time is slower than nspawn-harness but more realistic
- Image can be deployed to real hardware with bootc upgrade/switch
