---
title: "Release and promote a flavor"
---

## Avena base release

```text
source
  │
  ▼
build Rust binaries
  │
  ▼
build Avena bootc image
  ├── bootc container lint
  ├── static/security checks
  ├── boot integration test
  └── agent/Trellis functional tests
  ▼
sign/publish image
  ▼
Avena base digest
```

Flavor builds pin the base **by digest**.

## Flavor release

```text
Avena base digest + flavor source + bound app image digests
      │
      ▼
build flavor bootc image
      ├── boot in VM
      ├── verify flavor reconciliation
      ├── verify Trellis control
      ├── verify required workloads
      ├── simulate hardware where practical
      ├── test update from previous release
      └── test rollback
      ▼
publish/sign
```

## A flavor CI workflow

Flavor repositories use the reusable Avena workflow:

```yaml
# .github/workflows/release.yml
name: release
on:
  push:
    tags: ["tsd-*"]

jobs:
  flavor:
    uses: oats-center/avena-images/.github/workflows/flavor.yml@v1
    with:
      flavor: tsd
      containerfile: flavors/tsd/Containerfile
      previous-release: production       # upgrade-from test source
      hardware-sim: flavors/tsd/sim.toml  # simulated GNSS/CAN for VM tests
    secrets: inherit
```

The workflow builds, lints, boots the image in a VM, runs reconciliation, update, and rollback
tests, signs the image, produces installer artifacts (ISO, raw, qcow2), and publishes to the
`candidate` channel.

## Promotion

```text
development ──► candidate ──► canary ──► production
```

Promote the **already-tested digest**. Never silently rebuild "the same version" between
stages.

```bash
avena release promote tsd 2026.10.4 --from candidate --to canary
avena release promote tsd 2026.10.4 --from canary --to production
```

`release promote` moves channel metadata to point at the existing digest and records who
promoted it and when. Devices following that channel find it on their next
`update check`.

## Versioning

- **Avena base:** semver (`1.4.0`).
- **Flavors:** calendar (`2026.10.3`), with the base version recorded in the image labels.
- **Bound workloads:** whatever the application uses, always pinned by digest in the flavor.
