---
title: "Trellis services"
---

All Avena management is exposed as Trellis services on each device, addressed by the device's
Trellis identity. Every operation has a required [scope](/docs/reference/authorization/).

## `Avena.System`

| Operation | Scope | Description |
|---|---|---|
| `Status` | `system.read` | booted/staged/rollback deployments, boot state, update state |
| `Release` | `system.read` | flavor + base versions and image digests |
| `CheckForUpdate` | `system.read` | query the channel |
| `DownloadUpdate` | `system.update` | stage without applying |
| `ApplyUpdate { when_safe, force }` | `system.update` (+`system.reboot`) | activate staged update |
| `Rollback { apply }` | `system.update` | stage/apply rollback |
| `Reboot { when_safe, force }` | `system.reboot` | |
| `Shutdown { when_safe, force }` | `system.reboot` | |
| `ListInhibitors` | `system.read` | |
| `AcquireInhibitor { owner, reason, scope, ttl }` | `system.inhibit` | returns a lease ID |
| `RenewInhibitor { lease }` | `system.inhibit` | |
| `ReleaseInhibitor { lease }` | `system.inhibit` | |

**Events:** `UpdateStateChanged`, `BootStateChanged`, `InhibitorsChanged`.

## `Avena.Flavor`

| Operation | Scope | Description |
|---|---|---|
| `Describe` | `system.read` | name, version, base, hardware profile, conditions |
| `Components` | `component.read` | declared components |
| `Reconcile` | `component.control` | request a pass |
| `Health` | `system.read` | health evaluation |

## `Avena.Component`

| Operation | Scope |
|---|---|
| `List`, `Status` | `component.read` |
| `Start`, `Stop`, `Restart` | `component.control` (required components: plus override authority) |
| `Logs` | `component.read` |

**Events:** `ComponentStateChanged { id, desired, actual, health, reason }`.

## `Avena.Workload`

| Operation | Scope |
|---|---|
| `List`, `Status`, `Inspect`, `Logs` | `workload.read` |
| `Apply { spec }`, `Remove { name }` | `workload.deploy` |
| `Start`, `Stop`, `Restart` | `workload.deploy` |

## `Avena.Network`

| Operation | Scope |
|---|---|
| `Status`, `Providers`, `Peers`, `Routes`, `Interfaces`, `ProviderStatus` | `network.read` |
| `Reconnect { provider }` | `network.control` |

## `Avena.Hardware`

| Operation | Scope |
|---|---|
| `List`, `Inspect`, `Status` | `hardware.read` |
| `Rescan` | `hardware.read` |
| `Subscribe { id }` (brokered classes such as GNSS) | `hardware.read` or a per-ID capability granted to the workload |

## `Avena.Diagnostics`

| Operation | Scope |
|---|---|
| `Summary`, `Health`, `Journal` | `system.read` |
| `SupportBundle` | `system.read` (output is always redacted) |

## `Avena.Dev`

Present only on images with development enabled.

| Operation | Scope |
|---|---|
| `Status`, `ServiceUse`, `ServiceClear`, `WorkloadUse`, `WorkloadClear`, `UsrOverlay`, `Reset` | `development` |

## Client libraries

- **Rust:** the `avena-trellis` crate has typed clients (`SystemClient`, `WorkloadClient`,
  `HardwareClient`, …).
- **Other languages:** use the Trellis client for your language with the Avena contract
  definitions published with each release.
