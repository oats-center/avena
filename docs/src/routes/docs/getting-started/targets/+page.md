---
title: "Connect to devices and fleets"
---

Every `avena` command runs against a **target**. When you're on an Avena device, the default
target is the device itself. From a workstation, you pick a device through a **fleet context**.

## Targets

```bash
avena --target local status          # the machine you're on (default on devices)
avena --target tsd-02 status         # a device by Trellis identity
avena -t roadside-047 system status  # short form
```

Avena resolves a target in this order:

1. `local`: the agent on this machine, through its local Trellis endpoint.
2. A **Trellis device or service identity** in the current fleet context, such as `tsd-02`.
3. A **named fleet context** plus device: `avena -t lab-fleet/lab-01 status`.
4. An **explicit address**, accepted only by recovery and bootstrap tooling.

Set a default so you don't have to repeat yourself:

```bash
export AVENA_TARGET=tsd-02
avena status
```

## Fleet contexts

A context tells the CLI which Trellis deployment to join and which credentials to use.

```bash
avena context add oats-prod \
    --trellis wss://trellis.oats.example.org \
    --credentials ~/.config/avena/oats-prod.creds

avena context add lab \
    --trellis nats://lab-gw.local:4222 \
    --credentials ~/.config/avena/lab.creds

avena context list
avena context use oats-prod
```

```text
  NAME        TRELLIS                              DEFAULT TARGET
* oats-prod   wss://trellis.oats.example.org       -
  lab         nats://lab-gw.local:4222             lab-01
```

Contexts are stored in `~/.config/avena/contexts.toml`.

## Find devices

```console
$ avena fleet list
DEVICE         FLAVOR     RELEASE      CHANNEL      STATE      SEEN
tsd-01         tsd        2026.10.3    production   healthy    4s
tsd-02         tsd        2026.10.3    production   healthy    2s
tsd-07         tsd        2026.10.4    canary       probation  31s
isoblue-12     isoblue    2026.09.2    production   degraded   1m
roadside-047   roadside   2026.10.1    production   healthy    9s
```

Filter and select:

```bash
avena fleet list --flavor tsd --state degraded
avena fleet list --json | jq -r '.[] | select(.channel=="canary") | .device'
```

## What you're allowed to do

Your credentials carry Trellis **authorization scopes**. A read-only operator can run
`avena status` and `avena logs` on every device but cannot reboot one. Run this to see what
your context grants:

```console
$ avena context whoami
Context:  oats-prod
Identity: operator/jdoe
Scopes:   avena.system.read
          avena.component.read
          avena.workload.read
          avena.network.read
          avena.hardware.read
```

See [Authorization scopes](/docs/reference/authorization/) for the full list.

:::caution[Important]
The agent runs as root on the device, but that doesn't make *you* privileged. Every operation
is checked against your Trellis scopes. Avena has no "run arbitrary command" API.
:::
