---
title: "Authorization scopes"
---

Avena operations map to Trellis capabilities. All scopes are prefixed `avena.`.

| Scope | Grants |
|---|---|
| `avena.system.read` | status, release, update state, inhibitors, diagnostics |
| `avena.system.update` | check/download/apply updates, rollback, channel change, switch |
| `avena.system.reboot` | reboot and shutdown |
| `avena.system.inhibit` | acquire/renew/release reboot/update inhibitors |
| `avena.component.read` | component list/status/logs |
| `avena.component.control` | start/stop/restart components, request reconcile |
| `avena.workload.read` | workload list/status/inspect/logs |
| `avena.workload.deploy` | apply/remove/start/stop dynamic workloads |
| `avena.network.read` | network status, providers, peers, routes |
| `avena.network.control` | reconnect providers |
| `avena.hardware.read` | hardware inventory and brokered hardware data |
| `avena.development` | `Avena.Dev`: overrides, writable `/usr`, dev reset |

## Typical roles

| Role | Scopes |
|---|---|
| Viewer / dashboard | all `*.read` |
| Field operator | viewer + `system.reboot` |
| Fleet admin | operator + `system.update`, `component.control`, `workload.deploy`, `network.control` |
| Application workload | `system.inhibit`, `hardware.read` (or specific hardware IDs) |
| Developer (dev devices) | fleet admin + `development` |

## Rules

- The agent being root locally **does not** make callers privileged.
- Stopping a `required` component or forcing an update/reboot past inhibitors needs an
  **override** grant on top of the base scope, and it produces an audit event.
- `avena.development` is ignored on production images unless development is explicitly enabled.
