---
title: "flavor.toml reference"
---

The flavor declaration ships at **`/usr/lib/avena/flavor.toml`** in every flavor image. It's
immutable at runtime. Validate it with `avena flavor validate <file>`.

The schema is versioned with the top-level `schema` key. This page documents **schema 1**.

```toml
schema = 1

[flavor]
# …
[health]
# …
[update]
# …
[release]
# …
[[hardware]]
# …
[[components]]
# …
```

## `[flavor]`

| Key | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | short flavor name (`tsd`, `isoblue`, …). Lowercase, `[a-z0-9-]` |
| `version` | string | yes | flavor release version, e.g. `2026.10.3` |
| `display_name` | string | no | human-friendly name (`Avena TSD`) |
| `hardware_profile` | string | no | identifies the target hardware family |

## `[health]`

| Key | Type | Default | Description |
|---|---|---|---|
| `boot_probation_seconds` | int | `180` | how long all required checks must stay healthy after an update boot |
| `require_trellis` | bool | `false` | remote Trellis reachability is a required health check |
| `max_component_restarts` | int | `3` | restarts tolerated per component during probation |
| `waive_hardware` | [string] | `[]` | hardware IDs excluded from health |

## `[update]`

| Key | Type | Default | Description |
|---|---|---|---|
| `reboot_policy` | `"immediate"` \| `"when-safe"` \| `"maintenance-window"` \| `"manual"` | `"when-safe"` | when a downloaded update may be activated |
| `maintenance_window` | string `HH:MM-HH:MM` | — | required for `maintenance-window` |
| `timezone` | string | `"local"` | timezone for the window |
| `auto_download` | bool | `false` | download new channel releases automatically |
| `default_channel` | string | `"production"` | channel for newly installed devices |

## `[release]`

| Key | Type | Default | Description |
|---|---|---|---|
| `rollback_unsafe` | bool | `false` | release contains an irreversible data migration |
| `rollback_unsafe_reason` | string | — | shown to operators |

## `[[hardware]]`

Declares a stable hardware ID and how to recognize it.

| Key | Type | Required | Description |
|---|---|---|---|
| `id` | string | yes | stable ID, `<class-ish>.<name>` (`gps.primary`) |
| `class` | string | yes | `gnss`, `can`, `serial`, `camera`, `bluetooth`, `modem`, `imu`, `v2x`, … |
| `match` | table | yes | `{ udev = { PROP = "value", … } }` or `{ netdev = "can0" }` |
| `required` | bool | no (`false`) | missing hardware degrades the system |

## `[[components]]`

| Key | Type | Required | Description |
|---|---|---|---|
| `id` | string | yes | **stable** component ID, `<category>.<name>` (`network.tailscale`, `workload.tsd-ui`) |
| `kind` | `"systemd"` \| `"quadlet"` \| `"network"` \| `"hardware"` | yes | implementation |
| `unit` | string | for `systemd`/`quadlet` | systemd unit name |
| `provider` | string | for `network`/`hardware` | provider name (`tailscale`, `avena-overlay`, `mdns`, `ethernet`, `cellular`, …) |
| `role` | string | no | network role: `underlay`, `uplink`, `remote`, `overlay`, `discovery`, `application` |
| `required` | bool | no (`false`) | `true` ⇒ desired `required-running` |
| `enabled` | bool | no (`true`) | `false` ⇒ desired `desired-stopped` |
| `condition` | string | no | predicate making the component conditional (see below) |
| `bound` | bool | no | `quadlet` only: the image is a logically bound image (must be digest-pinned) |
| `requires` | [string] | no | hard semantic dependencies: component IDs, or `hardware:<id>` |
| `wants` | [string] | no | soft dependencies; `"a \| b"` means either |
| `phase` | `"always"` \| `"probation"` | no | `probation` runs only during boot probation (self-tests) |
| `health` | table | no | kind-specific health options, e.g. `{ min_peers = 1 }` |
| `dev_override` | bool | no (`true`) | whether `avena dev` may override this component |
| `manual_control` | `"any"` \| `"authorized"` \| `"none"` | no | who may start/stop it manually |

### Component ID rules

- Lowercase, `[a-z0-9.-]`, a category prefix: `core.`, `network.`, `hardware.`, `workload.`,
  `selftest.`, or `<product>.`.
- **Never rename an ID** between releases. Dashboards, policy, and operators depend on them.

### Conditions

```text
config.<key>              # truthy value in /etc/avena/config.toml
hardware.<id>             # hardware present
!<expr>                   # negation
<expr> && <expr>          # conjunction
<expr> || <expr>          # disjunction
```

Example: `condition = "config.overlay_enabled || hardware.v2x.radio"`.

## Things the schema deliberately doesn't have

No arbitrary shell scripting. If you need logic, put it in a host component or a self-test
unit, where it's versioned, logged, and health-checked like everything else.
