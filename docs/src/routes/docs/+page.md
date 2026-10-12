---
title: "Overview"
description: "What Avena is, what it gives you, and where to start."
---

Avena is the operating platform for OATS edge systems. It turns a general-purpose Linux
computer into a managed, reproducible, remotely controllable field-computing system. TSD
survey trucks, ISOBlue tractor loggers, unattended roadside units, lab testbeds, and gateways
all run the same platform, each in its own **flavor**.

> **The image defines what the machine is. The flavor declares what this product needs.
> The Avena agent reconciles what should be running. Trellis is how the system communicates
> and is controlled.**

## What Avena gives you

- **Transactional, image-based systems.** Every Avena machine boots an OCI image through
  [bootc](https://bootc.dev). Updates are atomic, and the previous deployment is always one
  rollback away.
- **One platform, many products.** TSD, ISOBlue, and Roadside are *flavors*: images derived
  from the common Avena base. Each one declares the networking, hardware, host services, and
  application workloads that product needs.
- **A reconciling agent.** `avena-agent` reads the flavor and keeps the machine in that state.
  It starts the systemd services, Quadlet containers, network providers, and hardware services
  the flavor asks for, and it reports desired and actual state separately.
- **Policy-aware updates.** You can download an update at the depot and apply it later in a
  maintenance window. Applications can hold an *inhibitor* so the device doesn't reboot in the
  middle of a survey. New releases go through boot probation and roll back on their own if they
  aren't healthy.
- **Composable networking.** Tailscale, the Avena overlay, mDNS neighborhood discovery,
  cellular, and C-V2X are all providers. A flavor picks which ones it uses.
- **One control plane.** Everything Avena does is exposed as typed
  [Trellis](/docs/concepts/trellis/) services. The `avena` CLI, fleet tools, and your own
  applications all call the same contracts.
- **Development without fear.** You can override a host binary, swap a container image, or make
  `/usr` writable. Then `avena dev reset --reboot` puts you back on the published system.

## Who this documentation is for

| You are…                       | Start with                                                                                                   |
|--------------------------------|--------------------------------------------------------------------------------------------------------------|
| An **operator** running devices in the field | [Tour a running device](/docs/tutorials/01-tour/), [Operate a fleet](/docs/guides/fleet/)                |
| An **application developer** shipping software onto Avena | [Deploy your first workload](/docs/tutorials/02-first-workload/), [Fast development loops](/docs/tutorials/06-dev-loop/) |
| A **product/flavor engineer** building a new device type | [Build your own flavor](/docs/tutorials/03-build-a-flavor/), [`flavor.toml` reference](/docs/reference/flavor-toml/) |
| A **platform contributor** working on Avena itself | [Architecture overview](/docs/concepts/architecture/), [Agent internals](/docs/project/agent-internals/) |

## What Avena is not

Avena is not Kubernetes, a multi-node scheduler, a replacement for systemd, Podman, or
Trellis, a package manager, or a configuration-management tool that keeps rewriting the root
filesystem. It is the integration and policy layer that gives these field systems one coherent
way of operating. See [Design principles](/docs/concepts/principles/).
