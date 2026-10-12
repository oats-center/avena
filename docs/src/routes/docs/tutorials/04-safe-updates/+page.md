---
title: "Tutorial 4: Safe updates with inhibitors"
---

**You will learn:** how to stage an update without rebooting, how an application prevents an
unsafe reboot, and how to let the device apply the update on its own once it's safe.

**Scenario:** truck `tsd-02` is parked at the depot with good Wi-Fi. A new TSD release,
2026.10.4, is out. The truck leaves on a survey in ten minutes. It must not reboot while
recording, but it should be updated before tomorrow.

## 1. Check for an update

```console
$ avena -t tsd-02 system update check
Current:    tsd 2026.10.3 (sha256:8f0c…)
Available:  tsd 2026.10.4 (sha256:a913…)  channel: production
Size:       212 MB system + 1 bound image changed (tsd-processing, 64 MB)
Notes:      https://github.com/oats-center/avena-images/releases/tag/tsd-2026.10.4
```

## 2. Download now, apply later

```console
$ avena -t tsd-02 system update download
Downloading tsd 2026.10.4…
  system image   ███████████████████████  212 MB
  tsd-processing ███████████████████████   64 MB
Staged. Activation is waiting for a safe maintenance window.
```

The new deployment and its bound images are now on the truck's disk. **Applying it later does
not need the registry**, which matters for a truck that will be out of coverage for most of
the day.

```console
$ avena -t tsd-02 system update status
State:      downloaded
Target:     tsd 2026.10.4 (sha256:a913…)
Apply:      maintenance-window 02:00–04:00 local
Inhibitors: none
```

## 3. The application takes an inhibitor

When the survey starts, the TSD acquisition service tells Avena that a reboot is unsafe. It
acquires a **leased inhibitor** over Trellis and renews it while recording:

```rust
use avena_trellis::system::{InhibitorScope, SystemClient};
use std::time::Duration;

let system = SystemClient::connect_local().await?;

let inhibitor = system
    .acquire_inhibitor("TSD.Acquisition")
    .reason("Survey recording active")
    .scope(InhibitorScope::Reboot)
    .ttl(Duration::from_secs(60))
    .send()
    .await?;

// The handle renews the lease in the background every ttl/3.
// If this process dies, the lease expires within 60 s. A crashed
// recorder cannot block updates forever.
record_survey().await?;

inhibitor.release().await?;
```

Not using Rust? Shell scripts and other languages can wrap a command, just like
`systemd-inhibit`:

```bash
avena system inhibit --owner TSD.Acquisition --reason "Survey recording active" \
    -- ./record-survey --route 42
```

The inhibitor is held while the command runs and released when it exits.

## 4. Try to apply while inhibited

```console
$ avena -t tsd-02 system update apply
Cannot apply update: reboot is inhibited.

TSD.Acquisition
  Survey recording active
  held 12m, lease renews every 20s
```

Nothing is lost. The update stays downloaded.

## 5. Apply when safe

Register your intent and let the agent take it from there:

```console
$ avena -t tsd-02 system update apply --when-safe
Update tsd 2026.10.4 will be applied when:
  ✓ downloaded
  ✗ no reboot inhibitors (1 active)
  ✗ inside maintenance window (02:00–04:00)
Intent recorded.
```

The truck finishes its survey, returns to the depot, and at 02:00 the agent applies the update
and reboots. That evening, `avena fleet list` shows `tsd-02` on `2026.10.4`.

## 6. Overriding (rarely)

```bash
avena -t tsd-02 system update apply --force --yes
```

This ignores inhibitors and maintenance windows. It needs the `avena.system.update` *and*
`avena.system.reboot` scopes, and it's recorded as an audit event on the device and in Trellis.
Use it for emergencies, not for convenience.

## How it fits together

```text
             check ──► available ──► downloading ──► downloaded
                                                         │
                         ┌───────────────────────────────┤
                         ▼                               │ inhibitors clear
            waiting-for-maintenance ◄────────────────────┘ + window open
                         │
                         ▼
                     applying ──► reboot-required ──► boot-probation ──► healthy
                                                             │
                                                             └──► rollback-required ──► rolling-back
```

See [Update states](/docs/reference/states/#Update-states) and
[Updates and channels](/docs/concepts/updates/).
