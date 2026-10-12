---
title: "workload.toml reference"
---

Dynamic workloads are declared in a `workload.toml` and deployed with
`avena workload apply <file>`. The agent persists the spec under
`/var/lib/avena/workloads/desired/<name>.toml` and renders it into
`/run/containers/systemd/<name>.container` on every boot.

```toml
schema = 1

[workload]
name = "weather-logger"
description = "Samples the weather station and serves readings"

[container]
image = "ghcr.io/oats-center/examples/weather-logger:1.2.0"
command = ["weather-logger", "--serve"]
environment = { SAMPLE_INTERVAL = "10s" }
ports = ["8080:8080/tcp"]
volumes = ["weather-data:/data"]

[restart]
policy = "always"

[resources]
memory_max = "256M"
cpu_weight = 50

[hardware]
devices = [{ id = "serial.aux", as = "/dev/aux" }]

[trellis]
provides = ["Weather.Logger"]
consumes = ["Avena.Hardware/gps.primary"]

[requires]
components = ["network.mdns"]

[health]
http = "http://localhost:8080/healthz"
interval = "15s"
```

## `[workload]`

| Key | Type | Required | Description |
|---|---|---|---|
| `name` | string | yes | unique on the device; `[a-z0-9-]`; becomes `<name>.service` |
| `description` | string | no | |

## `[container]`

| Key | Type | Description |
|---|---|---|
| `image` | string | **required.** Tag or `@sha256:` digest. Tags are resolved and recorded at apply time |
| `command` | [string] | override entrypoint arguments |
| `environment` | table | environment variables |
| `environment_files` | [string] | paths under `/etc/avena/workloads/` |
| `ports` | [string] | `host:container[/proto]` |
| `volumes` | [string] | `named:/path` (named volume) or `/var/lib/…:/path` |
| `user` | string | user inside the container |
| `read_only` | bool | read-only root filesystem (default `true`) |
| `capabilities` | [string] | extra capabilities; privileged ones are subject to flavor policy |
| `network` | `"private"` \| `"host"` | default `"private"`; `host` is subject to flavor policy |

## `[restart]`

| Key | Default | Description |
|---|---|---|
| `policy` | `"always"` | `always`, `on-failure`, `no` |
| `max_restarts` | `5` | within `window` before the workload is marked failed |
| `window` | `"5m"` | |

## `[resources]`

| Key | Description |
|---|---|
| `memory_max` | hard memory limit (`256M`, `1G`) |
| `cpu_weight` | relative CPU weight (1–10000) |
| `cpu_max` | CPU quota (`0.5` = half a core) |

## `[hardware]`

| Key | Description |
|---|---|
| `devices` | list of `{ id = "<avena hardware id>", as = "<path in container>" }`. Claims the device |

## `[trellis]`

| Key | Description |
|---|---|
| `provides` | Trellis services this workload exposes; credentials are issued accordingly |
| `consumes` | Trellis services/capabilities it calls |

## `[requires]`

| Key | Description |
|---|---|
| `components` | component IDs that must be running; otherwise the workload is `blocked` |

## `[health]`

One of:

| Key | Description |
|---|---|
| `http` | URL that must return 2xx |
| `command` | command run inside the container, exit 0 = healthy |
| `tcp` | `host:port` that must accept connections |

plus `interval` (default `30s`), `timeout` (`5s`), `start_period` (`30s`).

## Generations

Every successful `apply` bumps the workload's **generation**. If the new generation can't be
realized (for example, a failed pull), the previous generation keeps running and
`avena workload status` reports both.
