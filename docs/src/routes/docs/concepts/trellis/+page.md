---
title: "Trellis control plane"
---

**Trellis** is the OATS service communication and control platform. Avena uses it for
everything it exposes.

## One control plane

Trellis is how:

- `avena-agent` exposes system services;
- operators control Avena machines;
- the `avena` CLI reaches local and remote agents;
- Avena-hosted applications expose and consume services;
- TSD and other workloads talk to other services.

Avena has **no second, parallel RPC protocol**. The CLI, fleet tools, applications, and other
Avena nodes all use the same service contracts. On a healthy device the CLI even talks to the
*local* agent over Trellis, the same way a remote controller does.

Break-glass host repair is still possible with plain Linux, bootc, and systemd tools, but that
isn't the Avena control API.

## Services

| Service | Operations |
|---|---|
| `Avena.System` | Status, Release, CheckForUpdate, DownloadUpdate, ApplyUpdate, Rollback, Reboot, Shutdown, ListInhibitors, AcquireInhibitor, ReleaseInhibitor |
| `Avena.Flavor` | Describe, Components, Reconcile, Health |
| `Avena.Component` | List, Status, Start, Stop, Restart, Logs |
| `Avena.Workload` | List, Status, Apply, Remove, Start, Stop, Restart, Logs, Inspect |
| `Avena.Network` | Status, Providers, Peers, Routes, Interfaces, ProviderStatus, Reconnect |
| `Avena.Hardware` | List, Inspect, Status, Rescan |
| `Avena.Diagnostics` | Summary, SupportBundle, Journal, Health |
| `Avena.Dev` | development images only, see below |

Full signatures are in the [Trellis services reference](/docs/reference/trellis-services/).

## Authorization

Every operation maps to a Trellis **scope**, such as `avena.system.read`, `avena.system.update`,
`avena.workload.deploy`, or `avena.development`. See
[Authorization scopes](/docs/reference/authorization/).

> The Avena agent is privileged locally. That does not make remote callers privileged.

Policy also limits *which* components can be controlled manually. Stopping a `required`
component is rejected unless the caller holds an authorized override.

## Development service

Development images can expose `Avena.Dev` for service, workload, and `/usr` overrides. It's
off by default on production flavors and needs the separate `avena.development` scope.

## Calling Avena from your application

```rust
use avena_trellis::system::SystemClient;

let system = SystemClient::connect_local().await?;
let status = system.status().await?;
println!("{} {} — {:?}", status.flavor.name, status.flavor.version, status.boot_state);
```

The `avena-trellis` crate ships typed clients for every service. Applications in other
languages can use the Trellis client for that language with the published Avena contracts.
