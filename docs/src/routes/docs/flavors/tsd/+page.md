---
title: "TSD"
---

**Image:** `ghcr.io/oats-center/avena-tsd` · **Hardware profile:** `tsd-truck`

Avena TSD runs on survey trucks. It collects GNSS, CAN, and camera data on the road, uploads
opportunistically over cellular, and is serviced at the depot.

## Composition

```text
Avena TSD
│
├── Avena agent
├── Trellis
│
├── Networking
│   ├── cellular manager
│   ├── Tailscale
│   ├── local discovery
│   └── optional Avena overlay
│
├── Hardware
│   ├── GNSS
│   ├── CAN
│   ├── Bluetooth
│   ├── cameras
│   └── TSD-specific interfaces
│
├── Host services
│   ├── hardware brokers
│   └── services too privileged for useful container isolation
│
└── Bound application workloads
    ├── TSD acquisition
    ├── TSD processing
    ├── TSD operator UI
    └── support services
```

## Flavor declaration (excerpt)

```toml
schema = 1

[flavor]
name = "tsd"
version = "2026.10.3"
display_name = "Avena TSD"
hardware_profile = "tsd-truck"

[health]
boot_probation_seconds = 180
require_trellis = true

[update]
reboot_policy = "when-safe"

[[components]]
id = "network.tailscale"
kind = "systemd"
unit = "tailscaled.service"
required = true

[[components]]
id = "network.avena-overlay"
kind = "systemd"
unit = "avena-overlay.service"
required = false
condition = "config.overlay_enabled"

[[components]]
id = "hardware.bluetooth"
kind = "systemd"
unit = "bluetooth.service"
required = true

[[components]]
id = "hardware.gps"
kind = "systemd"
unit = "gpsd.service"
required = true

[[components]]
id = "workload.tsd-acquisition"
kind = "quadlet"
unit = "tsd-acquisition.service"
required = true
bound = true
requires = ["hardware.gps", "hardware.can"]

[[components]]
id = "workload.tsd-ui"
kind = "quadlet"
unit = "tsd-ui.service"
required = true
bound = true
```

## Operational notes

- **Inhibitors:** `tsd-acquisition` holds a `reboot` inhibitor (`TSD.Acquisition`) for the whole
  time a survey is recording.
- **Updates:** download at the depot; `apply --when-safe` is the fleet default.
- **GNSS** is brokered through `Avena.Hardware`. Workloads never open the receiver directly.
