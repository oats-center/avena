---
title: "State machines"
---

## Component desired states

| State | Meaning |
|---|---|
| `required-running` | must run; failure degrades the system and fails probation |
| `desired-running` | should run; failure reported, not fatal |
| `desired-stopped` | should not run |
| `conditional` | runs when its `condition` is true |
| `disabled` | not managed on this machine |

## Component actual states

| State | Meaning |
|---|---|
| `unknown` | not yet observed |
| `starting` | start requested, not ready |
| `running` | active and ready |
| `degraded` | running, but its health signal is partially failing |
| `failed` | exited/crashed, or exceeded restart policy |
| `stopping` | stop requested |
| `stopped` | not running |
| `blocked` | can't start: a requirement is missing (reason recorded) |

## Update states

| State | Meaning |
|---|---|
| `current` | running the channel's latest release |
| `checking` | querying the channel |
| `available` | newer release exists, not downloaded |
| `downloading` | staging system image and bound images |
| `downloaded` | staged on disk; can apply offline |
| `waiting-for-maintenance` | apply intent recorded; waiting for window and/or inhibitors |
| `applying` | finalizing the staged deployment |
| `reboot-required` | applied; reboot pending |
| `boot-probation` | booted new release; health checks running |
| `healthy` | probation passed; release is known-good |
| `rollback-required` | probation failed; rollback pending |
| `rolling-back` | rebooting into previous deployment |
| `failed` | update failed without needing rollback (e.g. download error) |

```text
current ─► checking ─► available ─► downloading ─► downloaded
                                                       │
                                            waiting-for-maintenance
                                                       │
                                      applying ─► reboot-required
                                                       │
                                                 boot-probation
                                                   │       │
                                              healthy   rollback-required ─► rolling-back
```

## Boot states (`avena status` → Boot state)

| State | Meaning |
|---|---|
| `healthy` | booted deployment is known-good |
| `probation` | booted deployment under evaluation |
| `degraded` | known-good deployment, but a required check is currently failing |
| `rolled-back` | running the previous deployment after a failed update |

## Inhibitor scopes

| Scope | Blocks |
|---|---|
| `reboot` | update activation, reboot (`update` downloads still allowed) |
| `update` | update download and activation |
| `shutdown` | shutdown and reboot |
