---
title: "Paths"
---

## Image-managed (read-only)

| Path | Contents |
|---|---|
| `/usr/lib/avena/flavor.toml` | flavor declaration |
| `/usr/lib/avena/trust.toml` | image trust policy (registries, signatures) |
| `/usr/lib/avena/` | other image defaults |
| `/usr/share/containers/systemd/` | flavor-bound Quadlets |
| `/usr/lib/bootc/bound-images.d/` | logically bound image registrations |
| `/usr/lib/systemd/system/` | host units, incl. `avena-agent.service` |
| `/usr/lib/udev/rules.d/` | flavor udev rules |

## Machine configuration

| Path | Contents |
|---|---|
| `/etc/avena/config.toml` | machine configuration (conditions, update window, …) |
| `/etc/avena/provision.toml` | provisioning material (consumed at first boot) |
| `/etc/avena/workloads/` | environment files referenced by workloads |

## Durable state

| Path | Contents |
|---|---|
| `/var/lib/avena/identity/` | device ID, keys, enrollment |
| `/var/lib/avena/state/` | component, network, hardware state |
| `/var/lib/avena/workloads/desired/` | dynamic workload specs (source of truth) |
| `/var/lib/avena/workloads/runtime/` | workload runtime bookkeeping |
| `/var/lib/avena/updates/pending.json` | in-flight update transaction |
| `/var/lib/avena/updates/last-good.json` | last known-good release |
| `/var/lib/avena/updates/history/` | update/rollback history |
| `/var/lib/avena/trellis/` | Trellis credentials and state |
| `/var/lib/avena/cache/` | caches (safe to delete) |
| `/var/lib/avena/recovery/` | failure diagnostics from rollbacks |

## Runtime (erased at reboot)

| Path | Contents |
|---|---|
| `/run/avena/` | sockets, locks, reconciliation snapshots, update progress |
| `/run/avena/dev/` | development overrides |
| `/run/containers/systemd/` | rendered dynamic and override Quadlets |
| `/run/systemd/system/*.d/50-avena-dev.conf` | host service dev overrides |

## Workstation

| Path | Contents |
|---|---|
| `~/.config/avena/contexts.toml` | fleet contexts |
| `~/.config/avena/*.creds` | Trellis credentials |
