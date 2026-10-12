---
title: "Images and flavors"
---

## Image hierarchy

```text
quay.io/fedora/fedora-bootc:<fedora-version>
                    │
                    ▼
        ghcr.io/oats-center/avena:<version>          Avena base
                    │
       ┌────────────┼───────────────┬──────────────┐
       ▼            ▼               ▼              ▼
 avena-tsd    avena-isoblue   avena-roadside    avena-lab   flavors
```

The **Avena base** is a real release artifact with its own version, provenance, tests, and
update policy. Flavors build `FROM` it, pinned by digest.

## What's in the base

- Linux kernel and the standard Fedora bootc userspace
- bootc, systemd, Podman, and Quadlet
- `avena-agent` and the `avena` CLI
- Trellis runtime and client support
- device identity support
- common logging, diagnostic, networking, and hardware-discovery tooling
- standard Avena systemd units
- SSH and recovery tooling, according to deployment policy

Optional providers such as Tailscale may be physically present in the base. That doesn't mean
they run.

> **Installed or available does not mean desired or running.**

## Flavors

A **flavor** is a complete product-specific system composition. It has two parts:

1. **Build-time composition**: packages, firmware, kernel modules, udev rules, host binaries,
   Quadlets, and configuration shipped in the image.
2. **The flavor declaration**: `/usr/lib/avena/flavor.toml`, a machine-readable list of the
   components and capabilities the agent reconciles at runtime.

```dockerfile
FROM ghcr.io/oats-center/avena@sha256:<digest>

# Product-specific packages, firmware and host integrations
RUN dnf install -y ...
COPY usr/ /usr/

# Product flavor declaration
COPY flavor.toml /usr/lib/avena/flavor.toml
```

A flavor may add kernel modules and firmware, udev rules, host services, network defaults,
CAN tooling, GPS services, Bluetooth configuration, camera dependencies, modem or C-V2X support,
product Quadlets, application image references, health criteria, and update/reboot policy
defaults.

TSD, ISOBlue, and Roadside are **not separate operating systems**. They're compositions of the
same platform.

## Flavor-bound container images

When a container is part of the tested product, its Quadlet ships in the flavor image and its
image is **pinned by digest**. It's also registered as a bootc **logically bound image**:

```text
Avena TSD bootc image
       │
       ├── Avena system
       ├── TSD host integration
       ├── TSD flavor manifest
       ├── tsd-acquisition.container ──► image@sha256:A
       ├── tsd-processing.container  ──► image@sha256:B
       └── tsd-ui.container          ──► image@sha256:C
```

What this gets you:

- system and application layers stay separate and deduplicated;
- large application images aren't copied into the host root;
- required images are already on disk at boot;
- rollback deployments keep their matching bound images;
- the production composition can be reproduced exactly by digest.

Dynamic or development workloads should **not** be bound just because Avena can run them.

## The published composition

The **published composition** is the known-good flavor image plus its pinned flavor-bound
workload images. Development overrides are always layered on top of it, and resetting returns
to it. See [Development model](/docs/concepts/development/).

## Installer artifacts

The same flavor OCI image is the source of truth for:

- the factory/installer ISO,
- raw disk images,
- VM images (qcow2),
- over-the-air updates to existing machines.

There are never separately maintained "installer OS" and "OTA OS" definitions.
