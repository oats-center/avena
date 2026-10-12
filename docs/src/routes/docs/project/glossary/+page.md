---
title: "Glossary"
---

**Avena**  
The complete OATS edge operating platform: base image, agent, lifecycle management,
reconciliation, networking, identity, hardware integration, workloads, health, Trellis
integration, and CLI.

**Avena base**  
The common bootc image (`ghcr.io/oats-center/avena`) all flavors derive from.

**avena-agent**  
The privileged daemon in every Avena image. It's the authoritative local owner of Avena
system state.

**bootc**  
Upstream tool for transactional, OCI-image-based Linux systems. It's Avena's update and
rollback *mechanism*.

**Bound image (logically bound)**  
A container image registered in `/usr/lib/bootc/bound-images.d` so that bootc fetches it
together with the system image.

**Boot probation**  
The period after booting a new release during which health checks must stay green before the
release is marked known-good.

**Channel**  
A release track: `development`, `candidate`, `canary`, `production`.

**Component**  
The general unit of managed functionality: systemd service, Quadlet workload, networking
provider, or hardware provider.

**Desired / actual state**  
What a component should be doing versus what it's doing. Always tracked separately.

**Dynamic workload**  
A workload applied at runtime with `avena workload apply`, independent of the system release.

**Flavor**  
A product-specific bootc image derived from the Avena base, plus its `flavor.toml`
declaration.

**Flavor-bound workload**  
A workload shipped and versioned with the flavor, pinned by digest, usually a bound image.

**Inhibitor**  
A leased, renewable request from an application to delay reboot, update, or shutdown.

**Known-good**  
A release that passed boot probation on this device.

**Override (dev)**  
A transient replacement of a host binary or workload image, stored under `/run`.

**Provider**  
A pluggable implementation of networking (Tailscale, overlay, cellular, …) or hardware
(GNSS, CAN, …).

**Published composition**  
The known-good flavor image plus its pinned bound workloads, with no overrides.

**Quadlet**  
Podman's systemd generator. It turns `.container` files into systemd services.

**Reconciliation**  
The agent's level-triggered, idempotent loop that brings actual state to desired state.

**Trellis**  
The OATS service communication and control platform. It's Avena's control plane.
