---
title: "Tutorial 7: Compose networking providers"
---

**You will learn:** how Avena networking is built from providers, how to enable Tailscale and
the Avena overlay on a flavor, how machine-level configuration switches conditional
providers on and off, and how to inspect peers and routes.

**You need:** two or more devices or VMs running a flavor of your own (see
[Tutorial 3](/docs/tutorials/03-build-a-flavor/)), on the same LAN.

## Networking is a set of providers

Avena has no single "the network". A flavor picks **providers**, each with a role:

| Role | Examples | What it does |
|---|---|---|
| `underlay` | ethernet, wifi | local link to the world |
| `uplink` | cellular | WAN when nothing else is available |
| `remote` | tailscale | reach the fleet and operators from anywhere |
| `overlay` | avena-overlay | authenticated device-to-device mesh (WireGuard) |
| `discovery` | mdns | find neighbors on the local network |
| `application` | c-v2x, custom | product-specific networks |

The agent brings up whatever bootstrap networking Trellis needs **first**, then joins Trellis,
then reconciles the rest. Avena never needs remote control in order to come online.

## 1. Declare providers in the flavor

```toml
[[components]]
id = "network.ethernet"
kind = "network"
provider = "ethernet"
required = true
role = "underlay"

[[components]]
id = "network.mdns"
kind = "network"
provider = "mdns"
required = true
role = "discovery"

[[components]]
id = "network.tailscale"
kind = "network"
provider = "tailscale"
required = false
role = "remote"

[[components]]
id = "network.avena-overlay"
kind = "network"
provider = "avena-overlay"
required = false
role = "overlay"
condition = "config.overlay_enabled"
requires = ["network.mdns"]
```

`condition` makes the overlay **conditional**. The flavor ships it, but each machine decides
whether to run it.

## 2. Turn on the overlay for one machine

Machine configuration lives in `/etc/avena/`. It's small, explicit, and survives updates.

```toml
# /etc/avena/config.toml
overlay_enabled = true
```

or remotely:

```bash
avena -t station-01 config set overlay_enabled true
avena -t station-02 config set overlay_enabled true
```

The agent re-evaluates conditions and reconciles:

```console
$ avena -t station-01 network providers
PROVIDER         DESIRED   STATE     ROLE
ethernet         running   healthy   underlay
mdns             running   healthy   discovery
tailscale        stopped   disabled  remote
avena-overlay    running   healthy   overlay
```

## 3. Peers and routes

```console
$ avena -t station-01 network peers
PEER          PROVIDER        ADDRESS                 VIA            STATE
station-02    mdns            192.168.4.31            eth0           seen 3s
station-02    avena-overlay   fd7a:a7e0::3f12:9c01    wg-avena       handshake 14s
station-03    avena-overlay   fd7a:a7e0::88a0:1d42    station-02     routed (2 hops)
```

Each peer relationship says **which provider produced it**. Overlay addresses are deterministic,
derived from device identity, so `station-02` keeps the same overlay address forever.

```console
$ avena -t station-01 network routes
DESTINATION              VIA            PROVIDER        METRIC
fd7a:a7e0::3f12:9c01/128 wg-avena       avena-overlay   10
fd7a:a7e0::88a0:1d42/128 station-02     avena-overlay   20
default                  192.168.4.1    ethernet        100
```

## 4. Add Tailscale for remote access

Tailscale is optional in this flavor, so turn it on:

```bash
avena -t station-01 component start network.tailscale
```

The agent provisions the machine's Tailscale identity from the enrollment material it holds
(Tailscale, Trellis, and Avena identities are **related but separate** credentials) and brings
it up:

```console
$ avena -t station-01 network provider tailscale status
State:      healthy
Tailnet:    oats-field.ts.net
Hostname:   station-01
Addresses:  100.71.3.20, fd7a:115c:a1e0::1a01:314
Peers:      42 online
```

## 5. Degrade gracefully

Pull the Ethernet cable from `station-02`. On `station-01`:

```console
$ avena -t station-01 status
…
Networking
  Ethernet:    healthy
  Tailscale:   healthy
  Overlay:     degraded (1 of 2 peers unreachable)
```

A provider failure is reported as an explicit **degraded** state. It doesn't stop the agent.
Whether that makes the whole device unhealthy depends on the flavor's `required` flags.
