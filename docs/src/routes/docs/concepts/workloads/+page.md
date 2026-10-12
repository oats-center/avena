---
title: "Workloads"
---

A **workload** is application-level software managed by Avena. Containers are the normal
form. A workload may be host-native if hardware or system integration makes a container
counterproductive.

## Why Podman + Quadlet by default

- dependency isolation
- clean image distribution
- reproducible application environments
- easy per-workload upgrades
- resource controls
- native systemd integration
- simple rollback when bound to a flavor release

Avena **generates or installs Quadlet definitions**. It doesn't run its own process
supervisor next to systemd.

## Two classes

### Flavor-bound

Part of the tested system composition, versioned with the flavor release:

- Quadlet shipped in `/usr/share/containers/systemd/` (immutable);
- image pinned by digest;
- registered as a bootc logically bound image;
- updated and rolled back together with the system.

Examples: the TSD acquisition service, a roadside data service that has to be there at boot,
a product monitoring sidecar.

### Dynamic

Managed independently of the host release:

- declared with a [`workload.toml`](/docs/reference/workload-toml/) and `avena workload apply`;
- persisted under `/var/lib/avena/workloads/desired/`;
- rendered at runtime into `/run/containers/systemd/`;
- stored in normal Podman storage, not bound.

Examples: optional modules, experiments, development images, temporary diagnostics.

```text
/var/lib/avena/workloads/desired/*      durable Avena specs
              │
              ▼
         avena-agent
              │
              ▼
     /run/containers/systemd/*          rendered each boot
              │
              ▼
      Quadlet generator / systemd
```

> **The durable source of truth is an Avena workload specification, not a mutable generated
> systemd file.**

## Image availability

- **Flavor-bound**: the image is fetched when the system release is staged, so it's
  guaranteed present at boot and on rollback.
- **Dynamic**: if a new image can't be pulled, Avena keeps the current one running and
  reports the update failure.

## Inspecting a workload

```console
$ avena workload inspect tsd-processing
Name:         tsd-processing
Source:       flavor-bound
Image:        ghcr.io/oats-center/tsd-processing@sha256:…
Bound:        yes
System unit:  tsd-processing.service
State:        running
Health:       healthy
Restarts:     0
```

## Privileges

Container privileges must be explicit. A workload that needs `NET_ADMIN`, the system D-Bus,
broad device access, or host networking should trigger an architecture review: that
functionality probably belongs in a host component. See
[Host service or container?](/docs/guides/host-vs-container/).
