---
title: "Hardware"
---

> **Avena owns system-wide hardware and infrastructure. Applications consume narrowly
> exposed hardware capabilities where practical.**

## Stable hardware IDs

Avena gives hardware **stable IDs** such as `gps.primary`, `can.vehicle`, and `camera.left`. A
flavor declares how to recognize each one (usually udev properties), and the agent maps the ID
to whatever `/dev` node, interface, or bus address it gets on this boot.

```console
$ avena hardware list
ID              CLASS       STATE     OWNER
gps.primary     gnss        healthy   hardware.gps
can.vehicle     can         healthy   hardware.can
bluetooth.0     bluetooth   healthy   hardware.bluetooth
camera.left     camera      healthy   workload.tsd-camera
```

Each device has at most one **owner**: the component that claimed it. Hardware can be
`required` by the flavor. A missing required device makes its dependents `blocked` and the
system `degraded`.

## Hardware providers

A hardware provider is a component that discovers and configures hardware and may expose it
to workloads. Examples: GNSS, CAN, cameras, Bluetooth, cellular modems, IMUs, C-V2X radios.

## "Uses hardware" ≠ "can't be a container"

Use the least-coupled model that works.

### Direct device pass-through

Good for stable, narrow interfaces: a specific serial port, `/dev/video*`, a specific USB
device, a CAN interface. The workload claims the device by Avena ID.

### Host facility + workload API

Prefer this when the subsystem is shared or system-wide:

```text
                Host
                 │
               BlueZ
                 │
       Avena Bluetooth service
             / Trellis API
                 │
          ┌──────┴──────┐
          ▼             ▼
     TSD workload   other workload
```

### Host-native component

Use this when a container would need broad privileges, host networking, the system D-Bus, many
devices, or deep lifecycle control, and so would give little isolation benefit anyway.

See [Tutorial 8](/docs/tutorials/08-hardware/) and
[Host service or container?](/docs/guides/host-vs-container/).
