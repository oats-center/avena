---
title: "Boot health and rollback"
---

## bootc rollback is necessary but not sufficient

bootc keeps a rollback deployment, but it can't tell whether *your product* works on the new
one. **Avena owns the health policy.**

## The update transaction

Before applying a new release, the agent records `/var/lib/avena/updates/pending.json`:

- current known-good deployment;
- target digest;
- flavor version;
- expected bound workload digests;
- update start time;
- required health checks;
- probation timeout.

After reboot the agent sees that it's booting a pending deployment and enters **probation**.

## Health checks

Typical requirements for a flavor:

- `avena-agent` is running;
- the flavor declaration is valid;
- required host components are healthy;
- required hardware is present or explicitly waived;
- required workloads are running;
- the expected networking provider is working;
- Trellis works locally;
- remote Trellis is reachable, if policy requires it;
- storage is mounted;
- no critical systemd units have failed;
- product-specific self-tests pass.

When all required checks **stay** healthy for the whole probation interval
(`boot_probation_seconds`), the release is marked **known-good** and recorded in
`last-good.json`.

## Failure handling

If the new deployment fails health policy, the agent will:

1. record the failure and diagnostics;
2. request rollback;
3. reboot into the previous deployment when it's safe to;
4. report the failed target once connectivity is back;
5. block automatic re-upgrade to that target until policy explicitly clears it.

The last step prevents rollback loops. Automatic rollback is tested against each supported
bootc backend and bootloader combination.

## Manual rollback

```bash
avena system rollback          # stage the rollback deployment
avena system rollback --apply  # …and reboot into it
```

If the release you're leaving declared an incompatible state migration, Avena warns you before
rolling back.

## Rollback is not a data backup

`/var` persists across deployments, so this can happen:

```text
new application writes schema v8
             │
          rollback
             │
old application sees schema v8
```

Workloads need a migration policy that tolerates this:

- backwards-compatible data formats;
- reversible migrations where possible;
- versioned state;
- product-specific Btrfs snapshots where justified;
- explicit **rollback-unsafe** release metadata for irreversible migrations.

Avena never implies that bootc rollback restores application data. See
[Make data migrations rollback-safe](/docs/guides/data-migrations/).
