---
title: The operating platform for edge systems
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

<Section eyebrow="Why Avena" title="Field computers that look after themselves" lede="Edge devices live in difficult places: patchy connectivity, power cuts, and nobody on site. Avena builds on bootc, systemd, Podman, and Trellis and adds the policy layer these machines are missing.">
<Bento />
</Section>

<Section eyebrow="How it works" title="Four layers, one contract" lede="The image defines what the machine is. The flavor declares what the product needs. The agent reconciles what should be running. Trellis is how the machine is controlled." link={{ to: '/docs/concepts/architecture/', label: 'Explore the architecture' }}>
<Stack />
</Section>

<Section split eyebrow="Declarative" title="A product is an image and a manifest" lede="Derive from the Avena base, add your drivers and services, and declare your components. The same image becomes the installer ISO, the VM image, and the over-the-air update." link={{ to: '/docs/tutorials/03-build-a-flavor/', label: 'Build your first flavor' }}>
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

<Section eyebrow="Policy-aware updates" title="Update the fleet without stopping the work" lede="Stage releases whenever there's bandwidth. Applications tell Avena when a reboot is unsafe, and the agent applies the update once it can. If a release isn't healthy, it rolls back on its own." link={{ to: '/docs/tutorials/04-safe-updates/', label: 'Walk through a safe update' }}>
<UpdateFlow />
</Section>

<Section split reverse eyebrow="For applications" title="Your code tells Avena when to wait" lede="Inhibitors are leases. Hold one while your app does critical work, such as recording a survey or flushing data. If your process dies, the lease expires on its own." link={{ to: '/docs/reference/trellis-services/', label: 'Trellis services reference' }}>
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

<Section split eyebrow="Developer experience" title="Experiment freely. Get back with one command." lede="Production images are immutable, but development doesn't have to be slow. Overrides live in /run, so they disappear on reboot, and avena dev status always shows what you've changed." link={{ to: '/docs/tutorials/06-dev-loop/', label: 'See the development loops' }}>

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

<Section eyebrow="Built for the field" title="One platform, many products" lede="These aren't separate operating systems. They're flavors of Avena.">
<Flavors />
</Section>

<Closing />

</div>
