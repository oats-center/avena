---
title: "Tutorial 6: Fast development loops"
---

**You will learn:** the four development loops Avena supports, from "swap one binary in seconds"
to "build and boot a whole system image", and how to always get back to the published system.

**You need:** a development device or VM, and the `avena.development` scope. Production flavors
disable the development service unless it is explicitly enabled.

## The idea

A development device always has a known **published composition** underneath whatever you're
experimenting with:

```text
published flavor  (image + bound workloads, immutable)
      │
      ├── transient host overrides        /run/systemd/system/…
      ├── transient workload overrides    /run/avena/dev/…
      └── optional writable overlays      /usr (bootc usr-overlay)
```

All the transient layers live in `/run`. A **reboot erases them**. `avena dev reset` erases them
without one. So on Avena it's easy to experiment, and it's easy to get back to a known-good
system.

| Loop | Use it for | Speed | Survives reboot? |
|---|---|---|---|
| [Host service override](#Loop-1--replace-a-host-binary) | `avena-agent`, `avena-overlay`, hardware daemons | seconds | no |
| [Workload override](#Loop-2--replace-a-container-image) | application containers | seconds | no |
| [Writable `/usr`](#Loop-3--make--usr-writable) | debuggers, trying a package | seconds | no |
| [System image](#Loop-4--build-and-boot-a-system-image) | testing what will actually ship | minutes | yes, it *is* the system |

## Loop 1: replace a host binary

You're working on the Avena overlay networking daemon. Build it on (or for) the device:

```bash
cd ~/src/avena
cargo build -p avena-overlay
```

Point the published service at your build:

```console
$ avena dev service use avena-overlay ./target/debug/avena-overlay
Override active until reset/reboot.
  unit     avena-overlay.service
  exec     /var/home/you/src/avena/target/debug/avena-overlay
  dropin   /run/systemd/system/avena-overlay.service.d/50-avena-dev.conf
Restarted avena-overlay.service.
```

Avena canonicalizes the path and refuses paths that will disappear unexpectedly, such as
something under `/tmp` (override with `--allow-volatile`). Edit, `cargo build`, and run it again:

```bash
cargo build -p avena-overlay && avena dev service use avena-overlay ./target/debug/avena-overlay
```

Under the hood this is a runtime systemd drop-in that replaces `ExecStart`:

```text
/usr/bin/avena-overlay                       published binary
            X
/run/systemd/system/…/50-avena-dev.conf      temporary ExecStart
            ▼
~/src/avena/target/debug/avena-overlay
```

Clear it:

```bash
avena dev service clear avena-overlay
```

## Loop 2: replace a container image

You're working on the TSD operator UI, a flavor-bound workload:

```console
$ podman build -t localhost/tsd-ui:dev .
$ avena dev workload use tsd-ui localhost/tsd-ui:dev
Override active until reset/reboot.
  workload  tsd-ui (flavor-bound)
  image     ghcr.io/oats-center/tsd-ui@sha256:… → localhost/tsd-ui:dev
Restarted tsd-ui.service.
```

The override goes in `/run/avena/dev/`, and the agent renders a runtime Quadlet that shadows
the published one. The flavor itself is untouched. Every other setting (ports, volumes,
devices, health checks) still comes from the published definition, so you're testing your code
in its real environment.

## Loop 3: make `/usr` writable

You need `gdb`, or want to try a host package before adding it to the flavor:

```console
$ avena dev system usr-overlay
/usr is now writable. Changes are discarded on reboot.

$ sudo dnf install -y gdb strace
```

This wraps `bootc usr-overlay`. It's great for exploration, but it is **not** a substitute for
testing the real image. Anything you find you need belongs in the Containerfile.

## Loop 4: build and boot a system image

When you change the Containerfile, the flavor declaration, or bound images, test the real
artifact:

```bash
podman build -t registry.dev.oats/you/avena-tsd:test .
podman push registry.dev.oats/you/avena-tsd:test
avena system switch registry.dev.oats/you/avena-tsd:test
avena system reboot
```

or use the convenience wrapper:

```bash
avena dev image build .                         # build + push to your dev registry
avena dev image boot registry.dev.oats/you/avena-tsd:test
avena dev image return                          # back to the channel's published image
```

This is slower, but it tests exactly what will ship.

:::tip[Tip]
For daily work, use the **Avena dev image** (`ghcr.io/oats-center/avena-<flavor>:dev`). It has
a transient writable root, debug symbols, Rust tooling, and diagnostic utilities. The *whole*
root filesystem is writable during a boot and reset on the next one. Your source and caches
live under `/var/home`, which persists.
:::

## See what you've changed

```console
$ avena dev status
Development mode

Published system:
  tsd 2026.10.3
  sha256:8f0c…

Host overrides:
  avena-overlay
    /var/home/you/src/avena/target/debug/avena-overlay

Workload overrides:
  tsd-ui
    localhost/tsd-ui:dev

Writable overlays:
  /usr    active

Persistent state:
  unchanged
```

`avena status` on a device with overrides always shows a `DEV OVERRIDES ACTIVE` banner, so
nobody mistakes your truck for a stock one.

## Get back to published

```bash
avena dev reset              # clear service + workload overrides, reconcile
avena dev reset --reboot     # …and reboot to drop /usr overlay too
```

```console
$ avena dev reset --reboot
Development overrides cleared.
Rebooting into published Avena TSD 2026.10.3.
```

`avena dev reset` **never** touches `/var/lib/avena`. Your device identity, dynamic workloads,
and update history are kept. On a disposable test device you can also wipe state:

```console
$ avena dev reset --state
This will delete application and Avena state under /var/lib/avena
(device identity is preserved). Type the device name to confirm: lab-01
```
