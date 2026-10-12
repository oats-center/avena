---
title: "Security model"
---

## Root privilege boundary

`avena-agent` needs significant host privilege. It manages bootc, systemd, networking,
hardware, Podman, and reboots. So it exposes a **narrow, strongly typed Trellis API** rather
than generic command execution.

## No remote shell disguised as an API

Avena does **not** expose any of these as normal Trellis operations:

```text
run arbitrary command as root
write arbitrary file
start arbitrary host unit
mount arbitrary device
```

Every operation is expressed in Avena terms (apply this workload, stage this release, start
this *declared* component) and authorized by [scope](/docs/reference/authorization/).
Development images may expose broader capabilities, but only under the separate
`avena.development` scope.

## Image trust

Production Avena defines:

- **allowed registries**;
- **signature/trust requirements** (Sigstore/cosign signatures, verified before staging);
- **digest pinning policy**;
- **release provenance** (SLSA attestations from CI);
- **credential storage** for private registries;
- **mirror policy**.

Avena won't accept an arbitrary unsigned flavor image just because bootc is technically able
to switch to it. On production flavors, `avena system switch` checks the policy in
`/usr/lib/avena/trust.toml` before staging anything.

## Workload privileges

Container privileges must be explicit in the workload spec. Requests for `NET_ADMIN`, the system
D-Bus, broad device access, host networking, or `privileged` are rejected on production flavors
unless the flavor explicitly allows them for that workload. Such a request is a signal that the
functionality may belong in a host component.

## Secrets

- Device keys live in `/var/lib/avena/identity/keys/` with `0600` root-only permissions,
  TPM-sealed where hardware permits.
- Support bundles **redact** credentials, private keys, tokens, and other secrets by design.
- Avena, Trellis, Tailscale, and overlay identities stay separate credentials.

## Audit

Privileged operations (forced update or reboot, overrides of required components, identity
operations, dev-mode enablement) produce audit events in the device journal and on Trellis.
