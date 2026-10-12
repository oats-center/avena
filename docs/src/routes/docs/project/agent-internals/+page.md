---
title: "Agent internals"
---

For contributors working on `avena-agent`.

## Module layout

```text
avena-agent
├── flavor        schema · loader · conditions
├── reconcile     planner · executor · state
├── component     systemd · quadlet · network · hardware
├── system        bootc · reboot · inhibitor · health
├── network       provider · tailscale · overlay
├── hardware      inventory · provider
├── workload      spec · quadlet · images
├── trellis       system_service · component_service · workload_service
│                 network_service · hardware_service
├── update        state · health · rollback
└── dev           override · reset
```

Execution backends sit **behind traits**. The reconciler works on Avena concepts
(components, desired/actual state), never on raw shell commands.

```rust
#[async_trait]
pub trait ComponentBackend {
    async fn observe(&self, c: &ComponentSpec) -> Result<ActualState>;
    async fn start(&self, c: &ComponentSpec) -> Result<()>;
    async fn stop(&self, c: &ComponentSpec) -> Result<()>;
    async fn health(&self, c: &ComponentSpec) -> Result<Health>;
}
```

Implementations: `SystemdBackend` (systemd over D-Bus), `QuadletBackend` (renders runtime
Quadlets, then delegates to systemd), `NetworkProviderBackend`, `HardwareProviderBackend`.

## Reconciliation

- **Level-triggered:** each pass computes desired state from scratch (flavor + `/etc/avena` +
  `/var/lib/avena` + `/run/avena` overrides) and compares it to what's observed.
- **Idempotent:** running a pass twice with nothing changed does nothing.
- **Event-requested, not event-driven:** systemd signals, udev events, provider events, and
  Trellis calls *request* a pass. A periodic timer covers anything that was missed.
- **Ordered:** networking → hardware → host services → bound workloads → dynamic workloads,
  respecting component `requires`.

## Talking to the system

| Concern | Interface |
|---|---|
| systemd | D-Bus (`org.freedesktop.systemd1`) |
| bootc | `bootc … --json --format-version=1` (stable CLI) |
| Podman | Quadlet files + systemd; Podman API for image pulls and inspect |
| hardware | udev monitor + provider-specific APIs |
| control | Trellis services in `trellis::*` |

## Testing philosophy

Tests validate **externally meaningful behavior**:

- a flavor boots;
- the agent reconciles required components;
- a Trellis command changes the intended system state;
- a workload starts and does its job;
- a provider forms the expected network;
- an update can be downloaded and applied, and an inhibited update waits;
- a failed release rolls back;
- a dev override disappears after reset/reboot;
- device identity survives a system upgrade.

Avoid tests that only freeze incidental implementation shape. The `avena-test` crate provides
multi-node clusters and chaos tooling (latency, partitions via toxiproxy) for integration
tests. The `scripts/` harnesses boot real bootc VMs and nspawn containers.
