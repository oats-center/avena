---
title: "Tutorial 8: Use hardware from a workload"
---

**You will learn:** the three ways a container workload can use hardware on Avena, and how to
pick between them.

**You need:** a device with a GNSS receiver (the examples use a TSD truck), or any USB serial
device.

## The rule of thumb

> **Avena owns system-wide hardware and infrastructure. Applications consume narrowly
> exposed hardware capabilities where practical.**

| Pattern | Use when | Example |
|---|---|---|
| **Direct pass-through** | narrow, stable device; one consumer | a camera, a dedicated serial port |
| **Host facility + Trellis API** | shared or system-wide subsystem | GNSS, Bluetooth, CAN fan-out |
| **Host-native component** | container would need broad privileges anyway | modem manager, overlay networking |

## 1. Find the hardware

```console
$ avena hardware list
ID              CLASS       STATE     OWNER
gps.primary     gnss        healthy   hardware.gps
can.vehicle     can         healthy   hardware.can
bluetooth.0     bluetooth   healthy   hardware.bluetooth
camera.left     camera      healthy   workload.tsd-camera
serial.aux      serial      healthy   -

$ avena hardware inspect serial.aux
ID:       serial.aux
Class:    serial
State:    healthy
Owner:    (unclaimed)
Device:   /dev/ttyUSB0
Match:    udev ID_VENDOR_ID=10c4 ID_MODEL_ID=ea60 ID_SERIAL_SHORT=0001
```

Applications refer to `serial.aux`. Avena maps that to whatever `/dev` node the device
ended up with on this boot.

## 2. Pattern A: direct pass-through

For a device that only your workload uses, claim it in the workload spec by **Avena hardware ID**:

```toml
# aux-reader.toml
schema = 1

[workload]
name = "aux-reader"

[container]
image = "ghcr.io/you/aux-reader:0.4.0"

[hardware]
devices = [{ id = "serial.aux", as = "/dev/aux" }]
```

```console
$ avena workload apply aux-reader.toml
Workload aux-reader: created
  hardware  serial.aux → /dev/aux (claimed)
Reconciling… running

$ avena hardware list | grep aux
serial.aux      serial      healthy   workload.aux-reader
```

The container sees exactly one device, at a stable path. If the device is unplugged, the
workload goes to `blocked` instead of crash-looping, and it comes back when the device does.

## 3. Pattern B: host facility + Trellis API

GNSS is **shared**. The acquisition service, the UI, and the time sync service all want
position. A single host service (`gpsd` plus Avena's GNSS provider) owns the receiver, and
workloads consume it through Trellis:

```text
               u-blox receiver
                     │
        gpsd + Avena GNSS provider   (host component hardware.gps)
                     │
              Avena.Hardware / Trellis
               ┌─────┴──────┐
               ▼            ▼
      tsd-acquisition     tsd-ui
```

In the workload spec, ask for the capability, not the device:

```toml
[trellis]
consumes = ["Avena.Hardware/gps.primary"]
```

and in your code:

```rust
use avena_trellis::hardware::HardwareClient;

let hw = HardwareClient::connect_local().await?;
let mut fixes = hw.gnss("gps.primary").subscribe().await?;

while let Some(fix) = fixes.next().await {
    println!("{:.6},{:.6}  ±{:.1} m", fix.lat, fix.lon, fix.h_acc_m);
}
```

No device nodes, no privileges, no fighting over the serial port.

## 4. Pattern C: host-native component

If a container would need `NET_ADMIN`, host networking, the system D-Bus, or many devices,
it isn't getting much isolation. Make it a **host component** in the flavor instead:

```toml
[[components]]
id = "hardware.cellular"
kind = "systemd"
unit = "ModemManager.service"
required = true
```

Expose what applications need (signal quality, data usage) through a Trellis API, the same way
as pattern B. See [Host service or container?](/docs/guides/host-vs-container/).

:::warning[Warning]
A workload spec asking for `privileged = true`, `NET_ADMIN`, host networking, or the system
bus is rejected on production flavors unless the flavor explicitly allows it. Treat that
rejection as a design review prompt.
:::

## Recap

- Refer to hardware by **Avena ID**, never by `/dev` path.
- Pass through narrow devices; broker shared ones through Trellis.
- If it needs broad privileges, it's a host component.

You've finished the tutorials. Continue with the [concepts](/docs/concepts/architecture/) or the
[how-to guides](/docs/guides/fleet/).
