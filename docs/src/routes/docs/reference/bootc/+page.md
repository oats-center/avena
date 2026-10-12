---
title: "bootc interfaces"
---

Avena relies only on **stable, documented bootc surfaces**, never on bootc internals.

## Commands used by the agent

| Avena | bootc |
|---|---|
| status / release info | `bootc status --json --format-version=1` |
| `system update check` | `bootc upgrade --check` |
| `system update download` | `bootc upgrade --download-only` |
| `system update apply` | `bootc upgrade --from-downloaded --apply` |
| `system switch <image>` | `bootc switch <image>` |
| `system rollback` | `bootc rollback` |
| `system rollback --apply` | `bootc rollback --apply` |
| `dev system usr-overlay` | `bootc usr-overlay` |

## Concepts used

- immutable, image-managed root;
- persistent `/var`;
- persistent (or optionally transient) `/etc`;
- optional transient writable root for development images;
- logically bound container images via `/usr/lib/bootc/bound-images.d`;
- OCI image transport and registry workflows.

Experimental unified/local container storage may speed up local image development later. Avena
doesn't require it.

## Upstream documentation

- [bootc.dev](https://bootc.dev/)
- [bootc API](https://bootc.dev/bootc/bootc-api.7.html)
- [Upgrades](https://bootc.dev/bootc/bootc-upgrades.7.html)
- [Filesystem](https://bootc.dev/bootc/filesystem.html)
- [Logically bound images](https://bootc.dev/bootc/bootc-logically-bound-images.7.html)
- [bootc(8)](https://bootc.dev/bootc/man/bootc.8.html)
