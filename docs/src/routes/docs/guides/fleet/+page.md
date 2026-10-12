---
title: "Operate a fleet"
---

Day-to-day recipes for operators who look after many devices.

## Morning check

```bash
avena fleet list --state degraded,failed,probation
```

```text
DEVICE         FLAVOR     RELEASE      CHANNEL      STATE      SEEN
isoblue-12     isoblue    2026.09.2    production   degraded   1m
tsd-07         tsd        2026.10.4    canary       probation  31s
```

Drill in:

```bash
avena -t isoblue-12 status
avena -t isoblue-12 component list --unhealthy
avena -t isoblue-12 logs --since 1h --priority warning
```

## Run a command across devices

`--target` accepts a selector as well as a single device:

```bash
avena -t 'flavor=tsd' system update status
avena -t 'flavor=roadside,channel=canary' flavor health --json > canary-health.json
```

Read-only commands fan out in parallel. Commands that change state ask for confirmation and
show the device list first, unless you pass `--yes`.

## Roll an update through a channel

```bash
# Stage on everyone in production; apply in each device's window
avena -t 'flavor=tsd,channel=production' system update download
avena -t 'flavor=tsd,channel=production' system update apply --when-safe --yes

# Watch progress
avena fleet list --flavor tsd --columns device,release,update,inhibitors
```

```text
DEVICE   RELEASE     UPDATE                      INHIBITORS
tsd-01   2026.10.4   healthy                     -
tsd-02   2026.10.3   waiting-for-maintenance     TSD.Acquisition
tsd-03   2026.10.4   boot-probation (92s left)   -
```

## Find who's blocking a reboot

```bash
avena -t tsd-02 system inhibitors
```

```text
OWNER             SCOPE    REASON                     HELD    EXPIRES
TSD.Acquisition   reboot   Survey recording active    1h12m   in 41s (renewing)
```

## Move a device between channels

```bash
avena -t tsd-07 system channel set canary
avena -t tsd-07 system update check
```

## JSON for scripting

Every command supports `--json`, with a stable, versioned output schema:

```bash
avena fleet list --json \
  | jq -r '.[] | select(.update.state=="rollback-required" or .boot_state=="failed") | .device'
```

## Get help from the platform team

```bash
avena -t isoblue-12 diagnostics support-bundle -o isoblue-12.tar.zst
```

Attach the bundle to your issue. It has versions, digests, bootc status, component state,
failed units, network and Trellis state, journal excerpts, hardware inventory, update history,
and the latest rollback reason, all with secrets redacted.
