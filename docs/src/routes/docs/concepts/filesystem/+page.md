---
title: "Filesystem and state"
---

Avena keeps a strict separation between what the image owns, what the operator configures,
what the agent persists, and what disappears at reboot.

## Four layers

| Layer | Location | Owned by | Lifetime |
|---|---|---|---|
| Image-managed | `/usr`, `/usr/lib/avena`, `/usr/share/containers/systemd`, `/usr/lib/systemd/system`, `/usr/lib/udev`, `/opt` | the bootc image | replaced on update |
| Machine/operator config | `/etc/avena/` | the operator | persistent, small, human-editable |
| Durable dynamic state | `/var/lib/avena/` | the agent | persistent across updates and rollbacks |
| Ephemeral runtime | `/run/avena/` | the agent | erased at every reboot |

Avena never treats image-managed locations as a database. It also resists moving large
amounts of mutable state into `/etc`.

## Configuration precedence

```text
/usr/lib/avena/        image defaults        (immutable, versioned with the image)
      ▲ overridden by
/etc/avena/            machine config        (small, explicit)
      ▲ overridden by
/run/avena/            runtime/dev overlays  (transient)
```

`/var/lib/avena` holds **state**, not configuration: things the agent learned or was told to
do, such as identity, desired dynamic workloads, and update history.

## `/var/lib/avena`

```text
/var/lib/avena/
├── identity/
│   ├── device-id
│   ├── keys/
│   └── enrollment/
├── state/
│   ├── components/
│   ├── network/
│   └── hardware/
├── workloads/
│   ├── desired/
│   └── runtime/
├── updates/
│   ├── pending.json
│   ├── last-good.json
│   └── history/
├── trellis/
├── cache/
└── recovery/
```

Secrets have strict ownership and permissions. Where the hardware supports it, they're backed
by hardware-protected storage (TPM).

## `/run/avena`

- development overrides;
- generated runtime files;
- sockets and locks;
- transient desired-state overlays;
- transient Quadlet definitions;
- update progress;
- current reconciliation snapshots.

A reboot removes this layer, and the [development model](/docs/concepts/development/) depends on that.

## `/var` is not rolled back

bootc deployments share one `/var`. A system rollback changes `/usr` and the bound images. It
does **not** change application data. See
[Make data migrations rollback-safe](/docs/guides/data-migrations/).
