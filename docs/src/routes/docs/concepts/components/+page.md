---
title: "Components and reconciliation"
---

## Components

A **component** is the general Avena unit of managed functionality. It's deliberately broader
than "container". A component can be:

- an Avena core service;
- a host systemd service;
- a container/Quadlet workload;
- a networking provider;
- a hardware provider;
- a device configuration action;
- a product-specific service;
- a conditional service, activated only when hardware or policy requires it.

Avena reasons about desired and actual state for each component, whatever mechanism runs it.

### Component kinds

| `kind` | Implemented by | Typical use |
|---|---|---|
| `systemd` | a host unit | platform or hardware-facing code: overlay daemon, BlueZ glue, CAN, watchdog |
| `quadlet` | a Podman container via Quadlet | ordinary application software |
| `network` | a networking provider | Tailscale, Avena overlay, cellular. Health includes connectivity, not just "process active" |
| `hardware` | a hardware provider | discovers/configures GNSS, CAN, cameras, modems, IMUs, C-V2X and exposes them to workloads |

## Desired state

Each component has one desired state:

| Desired | Meaning |
|---|---|
| `required-running` | Must run. Failure degrades the system and fails boot probation. Not stoppable without an authorized override. |
| `desired-running` | Should run. Failure is reported, but it isn't fatal. |
| `desired-stopped` | Should not run. |
| `conditional` | Runs when its `condition` (configuration or hardware predicate) is true. |
| `disabled` | Not managed on this machine. |

## Actual state

Tracked separately:

`unknown` · `starting` · `running` · `degraded` · `failed` · `stopping` · `stopped` · `blocked`

`blocked` means the component can't start because something it `requires` isn't available:
a hardware device, another component, or an image. The agent records the reason.

Keeping these two models separate means the execution backend never leaks into the control
model. A `quadlet` component and a `systemd` component look the same to operators.

## Dependencies

Components declare **semantic** dependencies on other Avena components without reproducing
systemd's dependency language:

```text
TSD acquisition
    requires:
      hardware.gps
      hardware.can

Trellis remote access
    wants:
      network.tailscale OR network.avena-overlay
```

Fine-grained process ordering can still use systemd `After=`/`Requires=` where that makes more
sense.

## The reconciliation loop

```text
load immutable flavor
        │
load machine config/state
        │
load runtime/dev overrides
        ▼
compute desired component graph
        │
inspect actual machine
        ▼
      diff
        ▼
ordered actions
        ├── start/stop systemd unit
        ├── write/remove runtime Quadlet
        ├── configure provider
        ├── request image pull
        └── report blocked/failed
        ▼
health evaluation
        ▼
publish state through Trellis
```

The loop is **level-triggered** and **idempotent**. Events (a unit stopping, a USB device
appearing, a Trellis request) only *request* another pass. The source of truth is always the
computed desired state compared with what's observed, never the event stream.

`avena flavor reconcile` requests a pass. It never changes the immutable flavor definition.

## Failure behavior

Avena prefers explicit degraded states over terminating the daemon.

Tailscale fails, overlay still works:

```text
Avena: degraded
Trellis: connected
network.tailscale: failed
network.avena-overlay: healthy
```

GPS is missing on a TSD, where it's required:

```text
hardware.gps: failed
TSD acquisition: blocked
Avena system: degraded/failed
```

The same thing on a lab flavor where GPS is optional:

```text
hardware.gps: absent
system: healthy
```

Whether a failure is fatal to the flavor is a flavor decision.
