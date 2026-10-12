---
title: "Architecture overview"
---

Avena is the operating platform for OATS edge systems. It isn't an application, a container
orchestrator, or a networking stack. It's the layer that turns a general-purpose Linux
computer into a managed OATS field-computing system.

It rests on four foundations:

1. **bootc**: transactional, OCI-delivered Linux system images, upgrades, image switching, and
   rollback.
2. **systemd**: the host service manager and boot dependency graph.
3. **Podman/Quadlet**: the normal execution environment for containerized applications.
4. **Trellis**: the communication and control plane through which Avena and Avena-hosted
   applications expose services and are remotely controlled.

## The stack

```text
┌─────────────────────────────────────────────────────────────┐
│                        Applications                         │
│       TSD       ISOBlue apps      Roadside apps      Other  │
│                      Trellis services                       │
└──────────────────────────────┬──────────────────────────────┘
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                           Trellis                           │
│   service contracts • RPC • events • identity • permissions │
│       Normal control/communication plane for Avena itself   │
└──────────────────────────────┬──────────────────────────────┘
                               ▼
┌─────────────────────────────────────────────────────────────┐
│                         avena-agent                         │
│  flavor reconciliation   bootc lifecycle   health/recovery  │
│  workloads               systemd           hardware         │
│  networking providers    update policy     observability    │
└───────────────┬───────────────────────────┬─────────────────┘
          ┌─────┴─────┐               ┌─────┴──────────────────┐
          │ systemd   │               │ networking             │
          │ Podman    │               │ Tailscale              │
          │ Quadlet   │               │ Avena overlay          │
          │ bootc     │               │ custom/application     │
          └─────┬─────┘               │ cellular / Wi-Fi / V2X │
                ▼                     └────────────────────────┘
┌─────────────────────────────────────────────────────────────┐
│                     bootc-managed Linux                     │
│    kernel • firmware • drivers • systemd • Podman • Avena   │
└─────────────────────────────────────────────────────────────┘
```

## Who does what

| Piece | Responsibility | Doesn't do |
|---|---|---|
| **bootc** | Stage, switch, and roll back OS images | Decide whether a release is healthy |
| **systemd** | Run and order processes | Know about flavors or products |
| **Podman/Quadlet** | Run containers as systemd units | Decide which containers should exist |
| **avena-agent** | Interpret the flavor, reconcile, apply update policy, judge health | Replace any of the above |
| **Trellis** | Typed services, events, identity, authorization | Provide connectivity |
| **Network providers** | Provide connectivity (Tailscale, overlay, cellular…) | Define "Avena networking" by themselves |

## The agent

`avena-agent` is the privileged daemon in every Avena image. It's started directly by
systemd and is the authoritative local owner of Avena system state. It:

- reads the immutable flavor declaration;
- loads machine-local configuration, state, and policy;
- reconciles desired components with actual system state;
- manages systemd units and Quadlet/Podman workloads;
- coordinates networking and hardware providers;
- manages bootc updates and rollback, and enforces inhibitors;
- reports health and release state;
- exposes the Avena control interface through Trellis.

It brings up enough local state and networking for Trellis to work **without first needing a
Trellis request**. After that, Trellis is the control path.

## Boot sequence

```text
1.  Bootloader selects the bootc deployment.
2.  Linux and systemd start.
3.  systemd starts avena-agent unconditionally.
4.  Agent loads /usr/lib/avena/flavor.toml, /etc/avena, /var/lib/avena,
    detects dev/runtime overrides, and checks whether this is a pending-update boot.
5.  Agent detects hardware.
6.  Agent establishes required bootstrap networking.
7.  Agent starts/joins Trellis.
8.  Agent computes the desired component graph.
9.  Agent reconciles networking → hardware → host services → bound workloads → dynamic workloads.
10. Agent evaluates health.
11. On an update boot: mark the release healthy, or trigger rollback policy.
12. Agent keeps going: watches events, responds to Trellis, reconciles, reports.
```

## Read next

- [Images and flavors](/docs/concepts/images-and-flavors/): what a machine *is*
- [Components and reconciliation](/docs/concepts/components/): what should be running
- [Trellis control plane](/docs/concepts/trellis/): how you talk to it
