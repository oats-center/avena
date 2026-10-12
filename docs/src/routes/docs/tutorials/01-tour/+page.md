---
title: "Tutorial 1: Tour a running device"
---

**You will learn:** how to read the state of an Avena device the way an operator does, using
status, components, workloads, networking, hardware, logs, and diagnostics.

**You need:** a running Avena device or the [quickstart VM](/docs/getting-started/quickstart-vm/).
The example output below comes from a TSD truck, `tsd-02`.

## Start with `avena status`

`avena status` is the one-screen summary. Run it first, every time.

```console
$ avena -t tsd-02 status
Avena TSD — tsd-02

System
  Avena:       1.4.0
  Flavor:      tsd 2026.10.3
  Image:       sha256:8f0c…
  Channel:     production
  Boot state:  healthy

Control
  Trellis:     connected
  Identity:    tsd-02

Networking
  Tailscale:   healthy
  Overlay:     disabled
  Cellular:    healthy

Components
  12 running
   0 degraded
   0 failed

Workloads
  tsd-acquisition    running
  tsd-processing     running
  tsd-ui             running

Update
  2026.10.4 downloaded
  waiting for maintenance window

Inhibitors
  TSD.Acquisition — Survey recording active
```

What it tells you:

- **System**: which flavor release is booted, the exact image digest, and whether the boot was
  judged healthy.
- **Control**: whether the device can be reached over Trellis.
- **Networking**: the state of each provider this flavor uses.
- **Update**: a newer release (2026.10.4) is already on disk, but the device is recording a
  survey, so it won't reboot into the update yet.

## Desired versus actual

```console
$ avena -t tsd-02 component list
ID                         TYPE       DESIRED   ACTUAL    HEALTH
network.tailscale          systemd    running   running   healthy
network.avena-overlay      systemd    stopped   stopped   n/a
network.cellular           network    running   running   healthy
hardware.bluetooth         systemd    running   running   healthy
hardware.gps               systemd    running   running   healthy
hardware.can               systemd    running   running   healthy
workload.tsd-acquisition   quadlet    running   running   healthy
workload.tsd-processing    quadlet    running   running   healthy
workload.tsd-ui            quadlet    running   running   healthy
```

Avena always shows **desired** and **actual** in separate columns. When they differ, the
agent is either working on closing the gap or will tell you why it can't:

```console
$ avena -t isoblue-12 component status hardware.can
ID:         hardware.can
Type:       systemd (isoblue-can.service)
Desired:    required-running
Actual:     blocked
Health:     failed
Reason:     required hardware can.implement not present
Dependents: workload.isoblue-logger (blocked)
```

## Workloads

```console
$ avena -t tsd-02 workload inspect tsd-processing
Name:         tsd-processing
Source:       flavor-bound
Image:        ghcr.io/oats-center/tsd-processing@sha256:41be…
Bound:        yes
System unit:  tsd-processing.service
State:        running
Health:       healthy
Restarts:     0
```

`Source: flavor-bound` tells you this container ships with the flavor release. You can't
replace it with `avena workload apply`. It changes only through a system update, or a dev
override on a development device.

## Networking and hardware

```console
$ avena -t tsd-02 network providers
PROVIDER         DESIRED   STATE     ROLE
ethernet         running   healthy   underlay
cellular         running   healthy   uplink
tailscale        running   healthy   remote
avena-overlay    stopped   disabled  overlay

$ avena -t tsd-02 hardware list
ID              CLASS       STATE     OWNER
gps.primary     gnss        healthy   hardware.gps
can.vehicle     can         healthy   hardware.can
bluetooth.0     bluetooth   healthy   hardware.bluetooth
camera.left     camera      healthy   workload.tsd-camera
```

Hardware has **stable Avena IDs** like `gps.primary`. Applications use these IDs instead of
`/dev/ttyACM0`, which can change between boots.

## Logs

```bash
avena -t tsd-02 logs --since 10m                       # everything Avena manages
avena -t tsd-02 logs --component hardware.gps --follow # one component
avena -t tsd-02 workload logs tsd-processing -n 200    # one workload
```

The logs come from journald on the device. You don't need SSH to read them.

## Diagnostics

```console
$ avena -t tsd-02 diagnostics health
Boot:          healthy (known-good since 2026-10-09 14:02 UTC)
Agent:         running, up 2d 4h
Flavor:        valid (schema 1)
Components:    12/12 healthy
Hardware:      4/4 required present
Networking:    remote reachable via tailscale
Trellis:       local ok, remote ok
Storage:       /var 38% used
systemd:       0 failed units
```

If something is wrong and you need help, capture a support bundle. Secrets are redacted
automatically:

```bash
avena -t tsd-02 diagnostics support-bundle -o tsd-02-bundle.tar.zst
```

## Recap

| Question                               | Command                                   |
|----------------------------------------|-------------------------------------------|
| Is it healthy? What's it running?      | `avena status`                            |
| Why is something not running?          | `avena component status <id>`             |
| Where did this container come from?    | `avena workload inspect <name>`           |
| Which networks are up?                 | `avena network providers`                 |
| Is the hardware there?                 | `avena hardware list`                     |
| What happened?                         | `avena logs`                              |
| Give me everything for a bug report    | `avena diagnostics support-bundle`        |
