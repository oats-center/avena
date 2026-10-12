---
title: "Tutorial 2: Deploy your first workload"
---

**You will learn:** how to describe a container workload, deploy it to a device, update it,
watch it get reconciled, and remove it.

**You need:** a device or [quickstart VM](/docs/getting-started/quickstart-vm/), and the
`avena.workload.deploy` scope (granted by default on the `lab` flavor's local identity).

We'll deploy `weather-logger`, a small service that samples a sensor and serves the readings
over HTTP.

## 1. Describe the workload

Avena workloads are declared in a `workload.toml` file. You don't write Quadlet or systemd
files yourself. Avena renders them for you.

```toml
# weather-logger.toml
schema = 1

[workload]
name = "weather-logger"
description = "Samples the weather station every 10 s and serves readings over HTTP"

[container]
image = "ghcr.io/oats-center/examples/weather-logger:1.2.0"
environment = { SAMPLE_INTERVAL = "10s", LOG_LEVEL = "info" }
ports = ["8080:8080/tcp"]
volumes = ["weather-data:/data"]

[restart]
policy = "always"

[resources]
memory_max = "256M"
cpu_weight = 50

[health]
http = "http://localhost:8080/healthz"
interval = "15s"
```

The full schema is in the [`workload.toml` reference](/docs/reference/workload-toml/).

## 2. Apply it

```console
$ avena workload apply weather-logger.toml
Workload weather-logger: created (generation 1)
  image     ghcr.io/oats-center/examples/weather-logger:1.2.0
  resolved  sha256:9d2e4f…
  unit      weather-logger.service
Pulling image… done (38.1 MB)
Reconciling… running
```

Behind the scenes:

```text
weather-logger.toml  ──Avena.Workload.Apply──►  avena-agent
                                                   │
                         /var/lib/avena/workloads/desired/weather-logger.toml   (durable)
                                                   │
                         /run/containers/systemd/weather-logger.container      (rendered)
                                                   │
                                     Quadlet generator → weather-logger.service
```

The **durable source of truth** is the Avena workload spec under `/var/lib/avena`. The Quadlet
file under `/run` is regenerated from it on every boot. Don't edit it, because your changes
would be lost.

:::tip[Tip]
To deploy to a remote device, add a target: `avena -t lab-01 workload apply weather-logger.toml`.
Nothing else changes.
:::

## 3. Check on it

```console
$ avena workload list
NAME              SOURCE          IMAGE                                   STATE     HEALTH
lab-hello         flavor-bound    ghcr.io/oats-center/lab-hello@sha256…   running   healthy
weather-logger    dynamic         …/weather-logger:1.2.0 (9d2e4f)         running   healthy

$ avena workload logs weather-logger --follow
weather-logger[2231]: listening on 0.0.0.0:8080
weather-logger[2231]: sample t=21.4C rh=48% p=1012hPa
```

```console
$ curl -s localhost:8080/latest
{"t":21.4,"rh":48,"p":1012,"at":"2026-10-12T15:04:10Z"}
```

## 4. Update it

Change the version and apply again:

```diff
-image = "ghcr.io/oats-center/examples/weather-logger:1.2.0"
+image = "ghcr.io/oats-center/examples/weather-logger:1.3.0"
```

```console
$ avena workload apply weather-logger.toml
Workload weather-logger: updated (generation 2)
  image     1.2.0 (9d2e4f) → 1.3.0 (c01a77)
Pulling image… done
Restarting… running
```

If the pull fails, for example because the device is offline, Avena **keeps the current
version running** and reports the failure. The update doesn't take the service down:

```console
$ avena workload status weather-logger
State:       running (generation 1, image 9d2e4f)
Desired:     generation 2 (image 1.3.0)
Last error:  image pull failed: registry unreachable; will retry
```

## 5. Survive a reboot

```bash
avena system reboot
# …after it comes back:
avena workload status weather-logger
```

The workload comes back because its spec lives in `/var/lib/avena`. Dynamic workloads are
**independent of the system image**. A system update or rollback leaves them as they were.

## 6. Start, stop, remove

```bash
avena workload stop weather-logger      # desired: stopped (persists across reboot)
avena workload start weather-logger
avena workload restart weather-logger
avena workload remove weather-logger    # deletes the spec; named volumes are kept
avena workload remove weather-logger --purge-volumes --yes
```

## Flavor-bound vs. dynamic

| | Flavor-bound | Dynamic |
|---|---|---|
| Declared in | flavor image (`/usr/share/containers/systemd`) | `workload.toml` via `avena workload apply` |
| Image pinned by | digest, at flavor build time | tag or digest, at apply time |
| Updated by | system update | `avena workload apply` |
| Rolled back by | system rollback | applying the previous spec |
| Present at first boot | yes (logically bound image) | after apply + pull |
| Good for | the product itself | add-ons, experiments, diagnostics |

When a dynamic workload becomes part of the product, move it into the flavor.
[Tutorial 3](/docs/tutorials/03-build-a-flavor/) shows how.
