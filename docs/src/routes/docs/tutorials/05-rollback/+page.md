---
title: "Tutorial 5: Break a release, watch it roll back"
---

**You will learn:** what boot probation is, how Avena decides a release is bad, how automatic
rollback works, and how Avena keeps a known-bad release from being reinstalled.

**You need:** the [quickstart VM](/docs/getting-started/quickstart-vm/) and a registry you can
push to. **Don't do this on a production device.**

## 1. Make a deliberately broken release

Start from the `lab` flavor and add a required component that can never become healthy:

```dockerfile
# Containerfile.broken
FROM ghcr.io/oats-center/avena-lab:2026.10.1

COPY broken.service /usr/lib/systemd/system/broken.service
RUN cat >> /usr/lib/avena/flavor.toml <<'EOF'

[[components]]
id = "core.broken"
kind = "systemd"
unit = "broken.service"
required = true
EOF
RUN sed -i 's/^version = .*/version = "2026.10.1-broken"/' /usr/lib/avena/flavor.toml
```

```ini
# broken.service
[Service]
ExecStart=/usr/bin/false
```

```bash
podman build -f Containerfile.broken -t ghcr.io/you/avena-lab:broken .
podman push ghcr.io/you/avena-lab:broken
```

## 2. Install it like a normal update

```console
$ avena system switch ghcr.io/you/avena-lab:broken
Staged lab 2026.10.1-broken. Reboot to activate.
$ avena system reboot
```

Before rebooting, the agent writes an update transaction to
`/var/lib/avena/updates/pending.json`:

```json
{
  "known_good": { "image": "sha256:3ac1…", "flavor_version": "2026.10.1" },
  "target":     { "image": "sha256:e7f0…", "flavor_version": "2026.10.1-broken" },
  "bound_images": { "lab-hello": "sha256:51c9…" },
  "initiated_at": "2026-10-12T16:20:03Z",
  "probation_seconds": 180,
  "required_checks": ["agent", "flavor", "components.required", "storage", "systemd"]
}
```

## 3. Watch probation

Connect to the console as it boots:

```text
avena-agent: pending update boot detected: 2026.10.1 → 2026.10.1-broken
avena-agent: entering boot probation (180 s)
avena-agent: reconcile: start broken.service
avena-agent: component core.broken: failed (exit 1)
avena-agent: reconcile: restart broken.service (attempt 2/5)
…
avena-agent: health: required component core.broken failed
avena-agent: probation FAILED: 1 required check failing
avena-agent: rollback: recording failure, requesting rollback to sha256:3ac1…
avena-agent: rollback: rebooting into previous deployment
```

After the second reboot:

```console
$ avena system status
Booted:
  flavor:       lab
  release:      2026.10.1
  image:        ghcr.io/oats-center/avena-lab@sha256:3ac1…
  state:        healthy

Rollback:
  release:      2026.10.1-broken
  image:        sha256:e7f0…

Last update:
  target:       2026.10.1-broken (sha256:e7f0…)
  result:       rolled back
  reason:       required component core.broken failed during probation
  blocked:      yes — will not auto-update to this digest

$ avena diagnostics summary --update-history
2026-10-12 16:20  2026.10.1 → 2026.10.1-broken   applied
2026-10-12 16:23  2026.10.1-broken               probation failed: core.broken
2026-10-12 16:24  2026.10.1-broken → 2026.10.1   rolled back
```

## 4. Known-bad protection

Avena won't automatically reinstall a target that failed probation. Rolling back to the good
release and then upgrading straight back into the bad one would just loop. To retry
deliberately:

```bash
avena system update unblock sha256:e7f0…
```

## 5. What probation checks

The flavor's `[health]` section and each component's `required` flag decide this. A typical
release must, for the whole probation interval:

- keep `avena-agent` running and the flavor declaration valid;
- keep every required component and required workload healthy;
- have required hardware present (or explicitly waived);
- have its networking provider and local Trellis working;
- reach remote Trellis, *if* `require_trellis = true`;
- have no failed critical systemd units and storage mounted;
- pass any product-specific self-tests.

See [Write flavor health checks](/docs/guides/health-checks/).

## 6. What rollback does **not** do

Rollback switches the **operating system image** and the flavor-bound workloads that came with
it. It does **not** roll back `/var`. If release N+1 migrated a database to schema v8 and then
got rolled back, release N will find schema v8. Read
[Make data migrations rollback-safe](/docs/guides/data-migrations/) before shipping a migration.

## Clean up

```bash
avena system update unblock sha256:e7f0…   # optional
```

Your VM is already back on the published `lab` release.
