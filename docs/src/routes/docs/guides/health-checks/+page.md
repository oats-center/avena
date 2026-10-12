---
title: "Write flavor health checks"
---

Boot probation decides whether a new release is kept. This guide covers how to tell Avena
what "healthy" means for your product.

## Built-in checks

These run for every flavor:

| Check | Passes when |
|---|---|
| `agent` | `avena-agent` is running and reconciling |
| `flavor` | the flavor declaration parses and validates |
| `components.required` | every `required = true` component is `running` and healthy |
| `hardware.required` | every required hardware ID is present (or waived) |
| `storage` | `/var` is mounted and writable, above minimum free space |
| `systemd` | no failed units marked critical |
| `trellis.local` | the local Trellis endpoint answers |
| `trellis.remote` | remote Trellis is reachable, *only if* `require_trellis = true` |

## Tuning probation

```toml
[health]
boot_probation_seconds = 180   # how long checks must stay green
require_trellis = true         # remote reachability is required
max_component_restarts = 3     # restarts tolerated during probation
waive_hardware = []            # hardware IDs that don't count against health
```

Pick `boot_probation_seconds` to cover your slowest legitimate startup *plus* time to see a
crash loop. 120–300 s is typical.

## Component health

A component is healthy when it's running and its own health signal is green:

- **systemd** components: active, plus `sd_notify` `READY=1` / watchdog if the unit uses them.
- **quadlet** components: the container's `HealthCmd`, or the workload spec's `[health]`.
- **network** components: provider-specific connectivity (for example, Tailscale has a
  coordination-server connection; the overlay has ≥ N peers if configured).

```toml
[[components]]
id = "network.avena-overlay"
kind = "network"
provider = "avena-overlay"
required = true
health = { min_peers = 1 }
```

## Product self-tests

For checks that only your product understands, add a `selftest` component. It's a oneshot
unit that exits 0 on success:

```toml
[[components]]
id = "selftest.can-loopback"
kind = "systemd"
unit = "tsd-can-selftest.service"
required = true
phase = "probation"          # run during probation only
requires = ["hardware.can"]
```

```ini
# tsd-can-selftest.service
[Service]
Type=oneshot
ExecStart=/usr/libexec/tsd/can-selftest --iface can0 --timeout 10s
```

Keep self-tests **externally meaningful** ("CAN frames round-trip", "the UI answers HTTP 200").
Don't use them to check incidental implementation details.

## Testing your checks

Before shipping, make a deliberately broken build and confirm it rolls back. See
[Tutorial 5](/docs/tutorials/05-rollback/). Flavor CI does this automatically. See
[Release and promote a flavor](/docs/guides/release-pipeline/).
