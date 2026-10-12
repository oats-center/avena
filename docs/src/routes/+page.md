---
title: Avena
description: Avena is the operating platform for OATS edge systems. It provides image-based updates, automatic rollback, composable networking, and one control plane for trucks, tractors, and roadside units.
layout: false
---

<script>
  import Hero from '#lib/landing/Hero.svelte'
  import Section from '#lib/landing/Section.svelte'
  import Bento from '#lib/landing/Bento.svelte'
  import Stack from '#lib/landing/Stack.svelte'
  import UpdateFlow from '#lib/landing/UpdateFlow.svelte'
  import Flavors from '#lib/landing/Flavors.svelte'
  import Closing from '#lib/landing/Closing.svelte'
</script>

<div class="family-page">

<Hero />

<Section eyebrow="Features" title="What Avena does" lede="Edge devices often have intermittent connectivity, unplanned power loss, and no one on site. Avena builds on bootc, systemd, Podman, and Trellis, and adds update policy, health checks, and reconciliation.">
<Bento />
</Section>

<Section eyebrow="Architecture" title="Layers" lede="The image defines what the machine is. The flavor declares what the product needs. The agent reconciles what should be running. Trellis is how the machine is controlled." link={{ to: '/docs/concepts/architecture/', label: 'Architecture overview' }}>
<Stack />
</Section>

<Section split eyebrow="Flavors" title="Images and manifests" lede="A product is an image derived from the Avena base, with its drivers and services, plus a manifest of its components. The same image is used for the installer ISO, the VM image, and over-the-air updates." link={{ to: '/docs/tutorials/03-build-a-flavor/', label: 'Build a flavor' }}>
<Tabs activeName="flavor.toml">
<TabPanel name="flavor.toml">

```toml
schema = 1

[flavor]
name = "weather"
version = "2026.10.0"

[health]
boot_probation_seconds = 120

[[components]]
id = "network.tailscale"
kind = "network"
provider = "tailscale"

[[components]]
id = "workload.weather-logger"
kind = "quadlet"
unit = "weather-logger.service"
required = true
bound = true   # shipped & rolled back with the image
```

</TabPanel>
<TabPanel name="Containerfile">

```dockerfile
FROM ghcr.io/oats-center/avena@sha256:5b1e…

RUN dnf install -y tailscale weather-sensord
COPY usr/ /usr/
COPY flavor.toml /usr/lib/avena/flavor.toml

RUN avena flavor validate /usr/lib/avena/flavor.toml \
 && bootc container lint
```

</TabPanel>
<TabPanel name="workload.toml">

```toml
schema = 1

[workload]
name = "weather-logger"

[container]
image = "ghcr.io/oats-center/examples/weather-logger:1.3.0"
ports = ["8080:8080/tcp"]
volumes = ["weather-data:/data"]

[hardware]
devices = [{ id = "serial.aux", as = "/dev/aux" }]

[health]
http = "http://localhost:8080/healthz"
```

</TabPanel>
</Tabs>
</Section>

<Section eyebrow="Updates" title="Update lifecycle" lede="Releases are downloaded when bandwidth is available and applied later. Applications can block reboots while they work. After a reboot, the release must pass health checks or the device rolls back." link={{ to: '/docs/tutorials/04-safe-updates/', label: 'Safe updates tutorial' }}>
<UpdateFlow />
</Section>

<Section split reverse eyebrow="Applications" title="Reboot inhibitors" lede="Inhibitors are leases. An application holds one during critical work, such as recording a survey or flushing data. If the process dies, the lease expires." link={{ to: '/docs/reference/trellis-services/', label: 'Trellis services reference' }}>
<Tabs activeName="Rust">
<TabPanel name="Rust">

```rust
let system = SystemClient::connect_local().await?;

let _guard = system
    .acquire_inhibitor("TSD.Acquisition")
    .reason("Survey recording active")
    .ttl(Duration::from_secs(60))   // renewed automatically
    .send()
    .await?;

record_survey().await?;
```

</TabPanel>
<TabPanel name="Shell">

```bash
avena system inhibit \
    --owner TSD.Acquisition \
    --reason "Survey recording active" \
    -- ./record-survey --route 42
```

</TabPanel>
<TabPanel name="Fleet">

```bash
avena -t 'flavor=tsd' system update download
avena -t 'flavor=tsd' system update apply --when-safe
avena fleet list --flavor tsd --columns device,release,update
```

</TabPanel>
</Tabs>
</Section>

<Section split eyebrow="Development" title="Development overrides" lede="Production images are immutable. For development, overrides of host binaries and container images live in /run and are cleared on reboot. avena dev status lists the active overrides." link={{ to: '/docs/tutorials/06-dev-loop/', label: 'Development loops' }}>

```console
$ cargo build -p avena-overlay
$ avena dev service use avena-overlay ./target/debug/avena-overlay
Override active until reset/reboot.

$ podman build -t localhost/tsd-ui:dev .
$ avena dev workload use tsd-ui localhost/tsd-ui:dev
Override active until reset/reboot.

$ avena dev reset --reboot
Development overrides cleared.
Rebooting into published Avena TSD 2026.10.3.
```

</Section>

<Section eyebrow="Flavors" title="Products built on Avena" lede="Each product is a flavor of the same Avena base image.">
<Flavors />
</Section>

<Closing />

</div>
