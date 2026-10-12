---
title: "CLI reference"
---

```text
avena

USAGE:
    avena [OPTIONS] <COMMAND>

COMMANDS:
    status       Show overall Avena system status
    system       System image, update, reboot and rollback
    flavor       Installed flavor and reconciliation
    component    Managed component state
    workload     Application workloads
    network      Network providers, peers and routes
    hardware     Detected and managed hardware
    logs         View Avena/component logs
    diagnostics  Health and support information
    config       Machine configuration (/etc/avena)
    dev          Development overrides and writable modes
    identity     Device identity information

WORKSTATION COMMANDS:
    context      Manage fleet contexts
    fleet        List and select devices
    release      Promote and export releases
    completion   Generate shell completions
```

## Principles

- Nouns are stable Avena concepts. Verbs are predictable across resources.
- Default output is for humans. `--json` gives stable, machine-readable output.
- Desired and actual state are always shown separately.
- Destructive operations need confirmation or `--yes`.
- `--target` works the same way for every command.
- Development commands live under `avena dev`.
- Raw bootc/systemd detail is hidden unless you pass `--verbose`.

## Global options

| Option | Env | Description |
|---|---|---|
| `-t, --target <TARGET>` | `AVENA_TARGET` | `local`, a device identity, `context/device`, or a selector like `flavor=tsd,channel=canary` |
| `--context <NAME>` | `AVENA_CONTEXT` | fleet context to use |
| `--json` | | machine-readable output |
| `-v, --verbose` | | include implementation detail (bootc, unit names, …) |
| `-y, --yes` | | don't prompt for confirmation |
| `--timeout <DUR>` | | Trellis request timeout (default `30s`) |

---

## `avena status`

One-screen summary of system, control, networking, components, workloads, update, and
inhibitors.

## `avena system`

| Command | Description | Scope |
|---|---|---|
| `system status` | booted / staged / rollback deployments, inhibitors | `system.read` |
| `system update check` | query the channel for a newer release | `system.read` |
| `system update download` | stage the update (bootc download-only), don't apply | `system.update` |
| `system update apply [--when-safe] [--force]` | activate staged update; `--when-safe` records intent; `--force` ignores inhibitors/windows | `system.update` (+`system.reboot`) |
| `system update status` | update state machine | `system.read` |
| `system update unblock <digest>` | allow a known-bad target again | `system.update` |
| `system update import <file>` | stage from offline OCI media | `system.update` |
| `system rollback [--apply]` | stage (and reboot into) the rollback deployment | `system.update` |
| `system channel show` / `set <channel>` | view/change release channel | `system.read` / `system.update` |
| `system switch <image> [--no-auto-rollback]` | switch to a specific image (policy may forbid) | `system.update` |
| `system reboot [--when-safe] [--force]` | reboot, respecting inhibitors by default | `system.reboot` |
| `system shutdown [--when-safe] [--force]` | shut down | `system.reboot` |
| `system inhibitors` | list active inhibitors | `system.read` |
| `system inhibit --owner <o> --reason <r> [--scope reboot] -- <cmd>` | hold an inhibitor while `cmd` runs | `system.inhibit` |

## `avena flavor`

| Command | Description |
|---|---|
| `flavor show` | name, version, base, hardware profile, condition values |
| `flavor components` | components declared by the flavor |
| `flavor health` | health evaluation against flavor policy |
| `flavor reconcile` | request a reconciliation pass |
| `flavor validate <file>` | validate a `flavor.toml` (offline, no device needed) |

## `avena component`

| Command | Description |
|---|---|
| `component list [--unhealthy]` | ID, type, desired, actual, health |
| `component status <id>` | detail, reason, dependents, recent events |
| `component start\|stop\|restart <id>` | change desired state (required components need override authority) |
| `component logs <id> [-f] [-n N]` | logs for the component |

## `avena workload`

| Command | Description |
|---|---|
| `workload list` | all workloads with source (flavor-bound/dynamic) |
| `workload status <name>` | state, generation, last error |
| `workload inspect <name>` | source, image digest, unit, bound, restarts |
| `workload apply <file>` | create/update a dynamic workload |
| `workload remove <name> [--purge-volumes]` | remove a dynamic workload |
| `workload start\|stop\|restart <name>` | lifecycle |
| `workload logs <name> [-f] [-n N]` | logs |

## `avena network`

| Command | Description |
|---|---|
| `network status` | overall connectivity summary |
| `network providers` | provider, desired, state, role |
| `network peers` | peers and which provider produced each |
| `network routes` | routes by provider |
| `network interfaces` | interfaces and addresses |
| `network provider <name> status` | provider-specific detail |
| `network provider <name> reconnect` | ask a provider to re-establish |

## `avena hardware`

| Command | Description |
|---|---|
| `hardware list` | ID, class, state, owner |
| `hardware inspect <id>` | match rules, device node, owner |
| `hardware status` | summary vs. flavor requirements |
| `hardware rescan` | re-run detection |

## `avena logs`

```bash
avena logs [--follow] [--since 10m] [--component <id>] [--priority warning]
```

## `avena diagnostics`

| Command | Description |
|---|---|
| `diagnostics summary [--update-history]` | key facts for a report |
| `diagnostics health` | every health check and its result |
| `diagnostics support-bundle [-o FILE]` | redacted support archive |

## `avena config`

| Command | Description |
|---|---|
| `config show` | effective configuration with source of each value |
| `config get <key>` / `set <key> <value>` / `unset <key>` | edit `/etc/avena/config.toml` |

## `avena dev`

| Command | Description |
|---|---|
| `dev status` | published system + all active overrides |
| `dev service use <component> <binary> [--allow-volatile]` | runtime `ExecStart` override |
| `dev service clear <component>` | remove it |
| `dev workload use <name> <image>` | runtime image override |
| `dev workload clear <name>` | remove it |
| `dev system usr-overlay` | transient writable `/usr` |
| `dev system status` | overlay state |
| `dev reset [--reboot] [--state]` | return to the published composition |
| `dev image build <dir>` | build + push a flavor image to your dev registry |
| `dev image boot <image>` | switch to it and reboot |
| `dev image return` | switch back to the channel's image |

All `dev` commands require `avena.development`.

## `avena identity`

| Command | Description |
|---|---|
| `identity show` | device ID, name, key type, and related provider identities |

## Workstation commands

| Command | Description |
|---|---|
| `context add <name> --trellis <url> --credentials <file>` | add a fleet context |
| `context list` / `use <name>` / `remove <name>` / `whoami` | manage contexts |
| `fleet list [--flavor F] [--channel C] [--state S] [--columns …]` | list devices |
| `release promote <flavor> <version> --from <ch> --to <ch>` | move a tested digest between channels |
| `release export <image> [--include-bound-images] -o <file>` | export for offline media |
| `completion <shell>` | shell completions |

## Exit codes

| Code | Meaning |
|---|---|
| 0 | success |
| 1 | general error |
| 2 | usage error |
| 3 | not authorized (missing scope) |
| 4 | target unreachable |
| 5 | blocked by policy (e.g. inhibitor, required component) |
| 6 | operation accepted but deferred (`--when-safe`) |
