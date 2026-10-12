---
title: "Development model"
---

Production Avena is immutable. Development shouldn't mean rebuilding and rebooting a whole
bootc image for every code change.

## Principle

A development machine always has a known published system underneath temporary overrides:

```text
published flavor
      │
      ├── transient host overrides
      ├── transient workload overrides
      └── optional writable overlays
```

Resetting or rebooting removes the transient layer. That gives Avena development a stronger
property than an ordinary mutable Linux machine:

> It is easy to experiment, and easy to know how to get back to a known-good system.

## Development loops

| Loop | Command | Mechanism |
|---|---|---|
| Host binary override | `avena dev service use <component> <binary>` | runtime systemd drop-in under `/run/systemd/system` replacing `ExecStart` |
| Container override | `avena dev workload use <name> <image>` | override in `/run/avena/dev/`, runtime Quadlet rendered over the published one |
| Writable `/usr` | `avena dev system usr-overlay` | `bootc usr-overlay`, discarded at reboot |
| Transient-root dev image | boot `avena-<flavor>:dev` | bootc/OSTree transient root (and optionally `/etc`) |
| Full system image | `avena system switch` / `avena dev image boot` | real bootc deployment from a dev registry |

### The dev image

```text
ghcr.io/oats-center/avena:dev
    ├── root transient = true
    ├── optional /etc transient = true
    ├── build/debug tools
    ├── symbols
    ├── cargo/rust tooling
    └── diagnostic utilities
```

Source and caches live under `/var/home`, which persists. Transient root is a property of how
the dev image is built and booted. A normal immutable image can't switch into it after boot.

### System image loop

```text
edit Containerfile → build derived bootc image → push to dev registry
  → avena system switch <dev image> → reboot
```

This is slower, but it's what ships. bootc's experimental unified-storage/local-build support
may speed it up later behind an explicit experimental flag. Avena's core developer
experience doesn't depend on it.

## Reset

| Command | Effect |
|---|---|
| `avena dev reset` | remove runtime service and workload overrides, clear dev-generated files, reconcile against published flavor |
| `avena dev reset --reboot` | the above + reboot (also drops a `/usr` overlay). **The one-command "return to published system".** |
| `avena dev reset --state` | *disposable test devices only*: also delete `/var/lib/avena` state (identity preserved), with strong confirmation |

`avena dev reset` never deletes `/var/lib/avena`.

See [Tutorial 6](/docs/tutorials/06-dev-loop/).
