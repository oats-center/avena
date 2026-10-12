---
title: "Avena in five minutes"
---

You only need a few ideas to work with Avena. Each one has a full page under
[Concepts](/docs/concepts/architecture/).

## 1. The machine is an image

An Avena machine boots a **bootc image**: a full operating system delivered as an OCI container
image. The root filesystem (`/usr`) is read-only and comes from that image. Updating means
staging a new image and rebooting into it. The previous deployment stays on disk as a rollback
target.

```text
quay.io/fedora/fedora-bootc
          │
          ▼
ghcr.io/oats-center/avena            ← the Avena base: agent, Podman, Trellis, tooling
          │
   ┌──────┼──────────┬─────────────┐
   ▼      ▼          ▼             ▼
  tsd   isoblue   roadside        lab   ← flavors
```

## 2. A flavor is a product

A **flavor** is a bootc image derived from the Avena base. It adds what one product needs:
drivers, firmware, udev rules, host services, container definitions. It also ships a
machine-readable declaration at `/usr/lib/avena/flavor.toml`:

```toml
schema = 1

[flavor]
name = "tsd"
version = "2026.10.3"

[[components]]
id = "hardware.gps"
kind = "systemd"
unit = "gpsd.service"
required = true

[[components]]
id = "workload.tsd-ui"
kind = "quadlet"
unit = "tsd-ui.service"
required = true
bound = true
```

## 3. Components are the unit of management

Each entry in the flavor is a **component**. A component can be a host systemd service, a
Quadlet container, a networking provider, or a hardware provider. Avena tracks a **desired
state** (`required-running`, `desired-stopped`, …) and an **actual state** (`running`,
`degraded`, `blocked`, …) for every component, whatever mechanism runs it.

## 4. The agent reconciles

`avena-agent` is a privileged daemon in every image, started by systemd at boot. It works out
what *should* be running from the flavor, machine configuration, and any overrides. It compares
that to what *is* running and closes the gap. It does this again on every event and on a timer.
Reconciliation is idempotent, so running it twice changes nothing.

## 5. Trellis is how you talk to it

The agent exposes typed services such as `Avena.System`, `Avena.Workload`, and `Avena.Network`
over **Trellis**, the OATS service communication platform. The `avena` CLI is a Trellis client.
It talks to the local device the same way it talks to a device across the country.

```bash
avena status                      # this machine
avena --target tsd-02 status      # a machine in the fleet
```

## 6. Workloads are (usually) containers

Application software normally runs as a **Podman container managed by Quadlet**.

- **Flavor-bound** workloads ship inside the flavor image, pinned by digest, and are updated and
  rolled back with it.
- **Dynamic** workloads are applied at runtime with `avena workload apply` and updated on their
  own schedule.

Hardware-heavy or system-wide software can run as a host service instead. Containers are a
tool, not a religion.

## 7. State has a place

| Path              | What lives there                    | Survives reboot? | Survives update? |
|-------------------|-------------------------------------|------------------|------------------|
| `/usr/lib/avena`  | Flavor declaration, image defaults  | yes              | replaced         |
| `/etc/avena`      | Small operator configuration        | yes              | yes (merged)     |
| `/var/lib/avena`  | Identity, desired workloads, update history | yes      | yes              |
| `/run/avena`      | Dev overrides, runtime state        | **no**           | **no**           |

That last row is why development on Avena is safe. Overrides live in `/run`, so a reboot clears
them.

## 8. Updates are policy, not just mechanism

bootc moves bits. Avena decides **when**. Updates can be downloaded now and applied later.
Applications can hold reboot **inhibitors**. A new release boots into **probation** and is only
marked known-good once the flavor's health checks pass. Otherwise Avena rolls back.
