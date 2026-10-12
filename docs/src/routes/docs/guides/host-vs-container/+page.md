---
title: "Host service or container?"
---

Avena doesn't require everything to be a container. Use this guide to decide where a piece of
software belongs.

## Decision guide

```text
Does it need any of:
  NET_ADMIN / CAP_SYS_ADMIN, host networking, the system D-Bus,
  many devices, or control over other system services?
        │
   yes ─┴─► Host component (kind = "systemd")
        │
   no
        │
Does it own a shared, system-wide hardware subsystem
(GNSS, Bluetooth, CAN fan-out, modem)?
        │
   yes ─┴─► Host component + Trellis API for consumers
        │
   no
        │
Does it need one narrow, stable device?
        │
   yes ─┴─► Container with device pass-through by Avena hardware ID
        │
   no
        │
        └─► Container (kind = "quadlet")
```

## Examples

| Software | Placement | Why |
|---|---|---|
| Avena overlay (WireGuard mesh) | host | needs `NET_ADMIN`, `/dev/net/tun`, routing tables |
| Cellular manager | host | system D-Bus, modem control, NetworkManager |
| Bluetooth broker | host + API | BlueZ is system-wide; several apps consume |
| GNSS | host + API | shared by acquisition, UI, time sync |
| CAN logger reading one bus | container + pass-through | one interface, one consumer |
| TSD processing | container | pure application logic |
| Web UI | container | pure application logic |
| Hardware watchdog | host | lifecycle-critical |

## Smells

- **"It just needs `--privileged`."** Then it's a host service.
- **"It runs in a container but manages host units."** It isn't really isolated, and it
  confuses ownership. Make it a host component.
- **"Two containers both open `/dev/ttyACM0`."** Broker it.
- **"The host service is just an app with a web UI."** Containerize it.

## Shipping a host component

Host components ship in the **flavor image**: a package or binary under `/usr`, a unit under
`/usr/lib/systemd/system`, and a component entry in `flavor.toml`. They're versioned and rolled
back with the system. During development, iterate with
[`avena dev service use`](/docs/tutorials/06-dev-loop/#Loop-1--replace-a-host-binary).
