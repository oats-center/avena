---
title: "Tutorial 3: Build your own flavor"
---

**You will learn:** how to derive a product image from the Avena base, declare its components,
ship a flavor-bound container, and boot it.

**You need:** Podman, a container registry you can push to (we use `ghcr.io/you`), and the
[quickstart VM](/docs/getting-started/quickstart-vm/) to boot the result.

We'll build **`weather`**, a flavor for a solar-powered weather station. It has:

- a USB weather sensor read by a host service, `weather-sensord`;
- the `weather-logger` container from [Tutorial 2](/docs/tutorials/02-first-workload/), now **flavor-bound**;
- Tailscale for remote access;
- mDNS so field laptops can find it.

## 1. Lay out the source

```text
avena-weather/
├── Containerfile
├── flavor.toml
└── usr/
    ├── lib/
    │   ├── bootc/bound-images.d/
    │   │   └── weather-logger.container -> ../../../share/containers/systemd/weather-logger.container
    │   ├── systemd/system/
    │   │   └── weather-sensord.service
    │   └── udev/rules.d/
    │       └── 70-weather-sensor.rules
    └── share/containers/systemd/
        └── weather-logger.container
```

## 2. The Containerfile

Always pin the base **by digest**. Then a flavor rebuild produces the same platform, and you can
tell exactly which Avena release a flavor contains.

```dockerfile
# Containerfile
FROM ghcr.io/oats-center/avena@sha256:5b1e…   # avena 1.4.0

# Host-side packages this product needs
RUN dnf install -y tailscale weather-sensord && dnf clean all

# Host integration: units, udev rules, quadlets, bound-image links
COPY usr/ /usr/

# The flavor declaration the agent reconciles
COPY flavor.toml /usr/lib/avena/flavor.toml

# Catch mistakes early
RUN avena flavor validate /usr/lib/avena/flavor.toml && bootc container lint
```

:::note[Note]
Installing `tailscale` here only makes it **available**. It runs only if the flavor
declares it. *Installed or available does not mean desired or running.*
:::

## 3. The flavor declaration

```toml
# flavor.toml
schema = 1

[flavor]
name = "weather"
version = "2026.10.0"
display_name = "Avena Weather Station"
hardware_profile = "weather-v1"

[health]
boot_probation_seconds = 120
require_trellis = false        # stations may boot with no uplink

[update]
reboot_policy = "maintenance-window"
maintenance_window = "02:00-04:00"

# ── networking ────────────────────────────────────────────
[[components]]
id = "network.tailscale"
kind = "network"
provider = "tailscale"
required = false

[[components]]
id = "network.mdns"
kind = "systemd"
unit = "avahi-daemon.service"
required = true

# ── hardware ──────────────────────────────────────────────
[[hardware]]
id = "weather.sensor"
class = "serial"
match = { udev = { ID_VENDOR_ID = "0403", ID_MODEL_ID = "6015" } }
required = true

[[components]]
id = "hardware.weather-sensor"
kind = "systemd"
unit = "weather-sensord.service"
required = true
requires = ["hardware:weather.sensor"]

# ── workloads ─────────────────────────────────────────────
[[components]]
id = "workload.weather-logger"
kind = "quadlet"
unit = "weather-logger.service"
required = true
bound = true
requires = ["hardware.weather-sensor"]
```

A few details worth noticing:

- `id` values are **stable**. Operators, dashboards, and health policy all refer to them.
- `requires` is a **semantic** dependency. If the sensor is missing, `weather-logger` shows as
  `blocked` with a reason instead of crash-looping.
- Validate locally at any time with `avena flavor validate flavor.toml`.

## 4. The flavor-bound workload

The container is defined as a normal Quadlet. Pin the image by digest:

```ini
# usr/share/containers/systemd/weather-logger.container
[Unit]
Description=Weather logger

[Container]
Image=ghcr.io/oats-center/examples/weather-logger@sha256:c01a77…
Environment=SAMPLE_INTERVAL=10s
Environment=SENSOR_SOCKET=/run/weather-sensord/sensor.sock
Volume=/run/weather-sensord:/run/weather-sensord:ro
Volume=weather-data:/data
PublishPort=8080:8080

[Service]
Restart=always

[Install]
WantedBy=multi-user.target
```

The symlink in `/usr/lib/bootc/bound-images.d/` registers it as a **logically bound image**. When
a device stages this flavor release, bootc fetches the container image at the same time. So
the workload is guaranteed to be on disk at first boot, and it stays available for rollback.

```bash
ln -s ../../../share/containers/systemd/weather-logger.container \
      usr/lib/bootc/bound-images.d/weather-logger.container
```

Note the design choice: the sensor is read by a **host service**, and the container only gets a
narrow Unix socket. The container never touches the USB device. See
[Host service or container?](/docs/guides/host-vs-container/).

## 5. Build and push

```bash
podman build -t ghcr.io/you/avena-weather:2026.10.0 .
podman push ghcr.io/you/avena-weather:2026.10.0
```

## 6. Boot it

On your quickstart VM (which is running the `lab` flavor), switch to your image. On
development devices this is allowed. Production policy normally restricts it to signed images
from approved registries.

Your VM has no weather sensor, so this release *will* fail boot probation, and Avena would
normally roll it back right away. To look around inside it anyway, pass
`--no-auto-rollback`. The failure is still evaluated and reported, but the agent doesn't
act on it. This flag only works on development devices.

```console
$ avena system switch --no-auto-rollback ghcr.io/you/avena-weather:2026.10.0
Target:   ghcr.io/you/avena-weather:2026.10.0 (sha256:77ad…)
Flavor:   weather 2026.10.0  (was: lab 2026.10.1)
Bound images: 1 (weather-logger)
Downloading… done
Staged. Reboot to activate.
Automatic rollback disabled for this deployment (development device).

$ avena system reboot
```

After it comes back:

```console
$ avena flavor show
Name:          weather
Version:       2026.10.0
Avena base:    1.4.0
Hardware:      weather-v1
Manifest:      /usr/lib/avena/flavor.toml

Required components: 3
Optional components: 1
Active conditions:
  hardware.weather.sensor = false

$ avena component list
ID                         TYPE      DESIRED   ACTUAL    HEALTH
network.tailscale          network   stopped   stopped   n/a
network.mdns               systemd   running   running   healthy
hardware.weather-sensor    systemd   running   blocked   failed
workload.weather-logger    quadlet   running   blocked   failed
```

The required hardware is missing. The device shows as **degraded** and tells you exactly
why. It doesn't fail silently. Pass a USB sensor through to the VM and run:

```bash
avena hardware rescan
avena flavor reconcile
```

and the blocked components start. `avena system status` shows that probation failed and that
automatic rollback was suppressed. [Tutorial 5](/docs/tutorials/05-rollback/) shows what happens when it isn't.

To go back to the lab flavor:

```bash
avena system rollback --apply
```

## Recap

- A flavor = `FROM avena@digest` + host integration + `flavor.toml`.
- Flavor-bound containers are Quadlets, pinned by digest, with a symlink in
  `bound-images.d`.
- Components have stable IDs, explicit `required` semantics, and semantic dependencies.
- The same image becomes the ISO, the disk image, and the over-the-air update. See
  [Release and promote a flavor](/docs/guides/release-pipeline/).
