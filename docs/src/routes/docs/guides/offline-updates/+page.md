---
title: "Update devices with poor connectivity"
---

Avena treats poor connectivity as a normal deployment condition, not an exception.

## Pattern 1: download at the depot, apply in the field

```bash
# At the depot (good Wi-Fi)
avena -t tsd-02 system update download

# Any time later. The registry isn't needed.
avena -t tsd-02 system update apply --when-safe
```

The staged deployment *and* its bound images are on local disk. `apply` doesn't touch the
network.

## Pattern 2: maintenance windows

Set it in the flavor:

```toml
[update]
reboot_policy = "maintenance-window"
maintenance_window = "02:00-04:00"
timezone = "local"
```

or per machine:

```toml
# /etc/avena/config.toml
[update]
maintenance_window = "01:00-03:00"
```

## Pattern 3: registry mirrors

Point devices at a closer registry (a site gateway, say) without changing image names:

```toml
# /etc/containers/registries.conf.d/50-oats-mirror.conf
[[registry]]
location = "ghcr.io/oats-center"

[[registry.mirror]]
location = "registry.site-gw.local/oats-center"
```

Signatures are verified against the original image reference, so a mirror can't substitute
content.

## Pattern 4: offline media

For sites with no usable uplink, export a release to removable media:

```bash
avena release export ghcr.io/oats-center/avena-roadside:2026.10.1 \
    --include-bound-images -o /media/usb/roadside-2026.10.1.oci
```

On the device:

```console
$ avena system update import /media/usb/roadside-2026.10.1.oci
Verifying signature… ok (oats-center release key)
Staged roadside 2026.10.1. Apply with `avena system update apply`.
```

## What keeps working offline

- Local reconciliation. It never depends on remote control.
- Running dynamic workloads. A failed pull keeps the current image.
- Rollback. Flavor-bound images for the rollback deployment are kept.
- Local Trellis. The CLI on the device works.
- Health probation, with `require_trellis = false` for flavors that may boot without uplink.
