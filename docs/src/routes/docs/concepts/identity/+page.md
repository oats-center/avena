---
title: "Identity and provisioning"
---

## Device identity

Every Avena machine has a device identity stored in `/var/lib/avena/identity/`. It is
**independent of the bootc deployment** and survives:

- OS updates;
- rollback;
- flavor rebuilds and flavor switches;
- application redeployments;
- `avena dev reset --state`.

```console
$ avena identity show
Device ID:    av1q7m2k9x…
Name:         tsd-02
Created:      2026-03-18T10:42:00Z
Key:          ed25519, TPM-sealed
Trellis:      tsd-02 (enrolled)
Tailscale:    tsd-02.oats-field.ts.net (node key rotates 90d)
Overlay:      fd7a:a7e0::3f12:9c01
```

Avena identity, Trellis identity, Tailscale identity, and other network-provider identities
are **related but never merged into one credential**. The agent knows how they all map to the
same physical machine.

Resetting or rotating identity requires explicit recovery/admin authorization and isn't
casually exposed.

## First boot

A newly installed flavor runs a minimal, deterministic first-boot sequence:

```text
boot
 │
 ▼
avena-agent
 ├── verify flavor declaration
 ├── create/load device identity
 ├── load provisioning material
 ├── detect hardware
 ├── establish base network
 ├── enroll/activate Trellis identity
 ├── start flavor networking providers
 ├── reconcile required components
 ├── validate health
 └── announce ready through Trellis
```

## Provisioning modes

**Connected enrollment.** The device boots with a short-lived enrollment token and exchanges
it for long-lived Trellis credentials the first time it reaches the fleet.

**Pre-provisioned / disconnected.** Credentials are generated ahead of time and delivered
on provisioning media, so the device is fully functional before it ever sees a network.

Provisioning material is read from a volume labelled `AVENA-PROVISION` or from
`/etc/avena/provision.toml`. After use, it's moved into `/var/lib/avena/identity/enrollment/`.

```toml
# avena-provision.toml
[identity]
name = "roadside-047"

[trellis]
url = "wss://trellis.oats.example.org"
enrollment_token = "…"

[access]
ssh_authorized_keys = ["ssh-ed25519 AAAA… ops@oats"]
```
