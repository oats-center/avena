---
title: "Networking"
---

## Networking is composable

Avena networking isn't tied to a single transport. A flavor may use any combination of:

- Ethernet
- Wi-Fi
- cellular
- Tailscale
- the Avena overlay
- mDNS neighborhood discovery
- C-V2X
- an application-specific overlay

The agent reconciles the network components the flavor and the current machine policy call
for. Each one is a [component](/docs/concepts/components/) of `kind = "network"`. Its health includes
**connectivity and topology**, not just whether a process is running.

## Bootstrap ordering

Avena must not need remote Trellis connectivity to bring up the connectivity Trellis needs:

```text
kernel
  │
systemd
  │
avena-agent
  │
  ├── load flavor and local state
  ├── establish required local networking
  ├── start required providers
  ├── start Trellis participation
  └── reconcile the remaining flavor
```

After bootstrap, Trellis is the normal control and communication path.

## Providers

### Avena overlay

An optional provider for authenticated device-to-device meshes:

- addresses derived deterministically from device identity;
- mDNS and static peer discovery;
- authenticated peer handshakes;
- WireGuard tunnel management;
- distributed routing and topology.

It's a natural fit for roadside clusters and machine-to-machine field work. It isn't assumed on
every flavor. Because it needs `NET_ADMIN` and `/dev/net/tun`, it runs as a **host service**,
not a privileged container.

### Tailscale

A provider and capability, not the definition of Avena networking. The agent can:

- start or stop Tailscale based on the flavor;
- provision or rotate the machine's Tailscale identity;
- establish links according to Avena policy;
- observe Tailscale health and expose it through Trellis;
- coordinate Tailscale with the overlay or local neighborhood networking.

### Others

Ethernet, Wi-Fi, and cellular (via NetworkManager/ModemManager), mDNS (Avahi), and
product-specific networks like C-V2X are all providers with the same status surface.

## Provider-neutral status

```console
$ avena network providers
PROVIDER         DESIRED   STATE     ROLE
ethernet         running   healthy   underlay
cellular         running   healthy   uplink
tailscale        running   healthy   remote
avena-overlay    stopped   disabled  overlay
```

Peer output always says which provider produced the relationship. Provider-specific detail is
available when you need it:

```bash
avena network provider tailscale status
avena network provider avena-overlay status
```

## Trellis is not the network

Trellis runs **over** whatever connectivity Avena sets up. It doesn't create links. Changing a
flavor from Tailscale to the overlay (or both, or neither) doesn't change how Avena is
controlled.

See [Tutorial 7](/docs/tutorials/07-networking/).
