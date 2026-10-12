---
title: "Updates and channels"
---

## bootc is the mechanism; Avena is the policy

Avena uses bootc's stable CLI/JSON interface, never its internals:

```text
Trellis request ──► Avena.System ──► avena-agent policy ──► bootc
```

Operators shouldn't normally need to run bootc commands themselves. Avena adds what bootc
doesn't know about: channels, maintenance windows, inhibitors, health probation, and
known-bad tracking.

## Update states

Avena exposes its own state machine, which is more useful to applications and operators than
raw bootc output:

`current` → `checking` → `available` → `downloading` → `downloaded` →
`waiting-for-maintenance` → `applying` → `reboot-required` → `boot-probation` →
`healthy` *(or `rollback-required` → `rolling-back`)* · `failed`

See [State machines](/docs/reference/states/#Update-states).

## Download and apply are separate

For mobile and roadside systems, getting the bits and switching to them happen at different
times:

| Command | bootc | When |
|---|---|---|
| `avena system update download` | `bootc upgrade --download-only` | when connectivity is good |
| `avena system update apply` | `bootc upgrade --from-downloaded --apply` | when it's safe to reboot. **Doesn't need the registry** |

A truck can download at the depot and reboot only after its survey. A roadside unit can
download during the day and apply in its 02:00 maintenance window.

## Inhibitors

Applications can tell Avena that a reboot or update is temporarily unsafe. An inhibitor has:

```text
owner       TSD.Acquisition
reason      "Survey recording active"
created_at  2026-10-12T14:03:11Z
scope       reboot | update | shutdown
ttl         60 s, renewable (lease)
```

The holder renews the lease for as long as the condition holds. If the holder crashes, the
lease expires, so a dead process can't block updates forever. Avena **may download** while
inhibited, but won't **activate** a deployment that needs a reboot until policy allows it.

See [Tutorial 4](/docs/tutorials/04-safe-updates/).

## Channels

Devices follow a **flavor + channel**, not a raw image name:

```text
Flavor:  tsd
Channel: production
Image:   ghcr.io/oats-center/avena-tsd@sha256:...
```

| Channel | Who | Purpose |
|---|---|---|
| `development` | dev devices | every build |
| `candidate` | lab and test fleet | release candidates |
| `canary` | a few field devices | real-world validation |
| `production` | everyone else | known-good |

Promotion moves an **already tested digest** to the next channel. It never rebuilds. Every
device records the exact digest it booted.

```bash
avena system channel show
avena system channel set candidate
```

Production policy may forbid arbitrary `avena system switch <image>`. Images must come from
allowed registries and carry valid signatures. See [Security model](/docs/concepts/security/#Image-trust).

## Poor connectivity is normal

- download without activating;
- apply a previously downloaded update offline;
- keep running workloads when the registry is unreachable;
- keep flavor-bound images for rollback;
- use registry mirrors;
- install from offline OCI media where needed.

See [Update devices with poor connectivity](/docs/guides/offline-updates/).
