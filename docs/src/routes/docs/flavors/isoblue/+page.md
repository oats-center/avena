---
title: "ISOBlue"
---

**Image:** `ghcr.io/oats-center/avena-isoblue` · **Hardware profile:** `isoblue-v3`

Avena ISOBlue runs on tractors and other agricultural machines. It logs ISOBUS/CAN and GNSS
data and serves it to farm applications.

## Composition

```text
Avena ISOBlue
│
├── Avena agent
├── Trellis
│
├── Networking
│   ├── Wi-Fi
│   ├── cellular
│   ├── Tailscale and/or Avena overlay
│   └── neighborhood discovery
│
├── Hardware
│   ├── CAN
│   ├── GNSS
│   ├── modem
│   └── tractor-specific interfaces
│
└── Workloads
    ├── ISOBlue domain services
    ├── logging
    └── user/application services
```

The exact network combination is a flavor decision, not a platform assumption. Machines that
work together in a field (a combine and a grain cart, say) enable the Avena overlay so they
can talk directly, with no uplink.

## Operational notes

- **CAN** comes in two kinds: `can.tractor` (required) and `can.implement` (optional, present
  only when an implement is hitched). The `isoblue-logger` workload requires the first and wants
  the second.
- **Inhibitors:** the logger holds an inhibitor while it's flushing critical data.
- **Dynamic workloads** are common on ISOBlue. Research groups deploy experiment containers
  with `avena workload apply` without rebuilding the flavor.
