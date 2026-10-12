---
title: "Repository layout"
---

## `oats-center/avena`: agent, CLI, libraries

```text
oats-center/avena
├── crates/
│   ├── avena-agent/      the privileged host daemon
│   ├── avena-cli/        the `avena` command
│   ├── avena-core/       shared types: IDs, states, errors
│   ├── avena-flavor/     flavor.toml schema + validation
│   ├── avena-system/     bootc, reboot, inhibitors, health
│   ├── avena-workload/   workload specs, Quadlet rendering
│   ├── avena-network/    provider abstraction
│   ├── avena-hardware/   inventory and providers
│   └── avena-trellis/    service contracts + typed clients
├── providers/
│   ├── overlay/          Avena overlay (WireGuard mesh)
│   ├── tailscale/
│   └── …
├── docs/                 this site
└── tests/
```

## `oats-center/avena-images`: base and flavor images

```text
oats-center/avena-images
├── base/
│   └── Containerfile
├── flavors/
│   ├── tsd/
│   ├── isoblue/
│   ├── roadside/
│   ├── lab/
│   └── test/
└── installer/
```

The base and flavor images are official Avena artifacts, with a defined release relationship
to the agent source.

## Lineage

Avena brings together several earlier efforts:

- **`avenad`** (earlier versions of this repo) contributed the systemd D-Bus
  control, workload reconciliation, Quadlet generation, device identity, and status concepts.
  Its assumptions that NATS *is* the control plane, that JetStream KV is the source of
  desired state, that every workload is a Podman container, and that NATS leaf-node links
  define networking have been replaced by the Trellis-based architecture.
- **`avena-ng`** became the **Avena overlay** networking provider: identity, discovery,
  cryptographic handshakes, WireGuard, and routing. It runs as a host service rather than a
  privileged container.
- **`avena-os-bootc`** established the base-image-plus-derived-flavors pattern. It continues as
  `avena-images`.
