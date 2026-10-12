---
title: "Roadside"
---

**Image:** `ghcr.io/oats-center/avena-roadside` · **Hardware profile:** `roadside-cabinet`

Avena Roadside runs unattended in roadside cabinets: sensing, V2X processing, and telemetry.
Nobody is around to press a reset button.

## Composition

```text
Avena Roadside
│
├── Avena agent
├── Trellis
│
├── Networking
│   ├── Ethernet/cellular underlay
│   ├── Avena overlay
│   ├── optional Tailscale
│   └── C-V2X/application-specific network
│
├── Hardware
│   ├── sensors
│   ├── radios
│   └── timing/GNSS
│
└── Workloads
    ├── roadside acquisition
    ├── V2X processing
    └── telemetry
```

## Operational notes

- **Unattended recovery comes first.** Roadside uses a longer probation
  (`boot_probation_seconds = 600`), requires remote Trellis, and includes C-V2X radio self-tests.
  A release that can't phone home is rolled back.
- **Maintenance window:** 02:00–04:00 local. Units download during the day and apply overnight.
- **Overlay required:** neighboring units form an Avena overlay mesh along a corridor. One
  unit's cellular uplink can serve its neighbors.
