---
title: "Make data migrations rollback-safe"
---

**OS rollback is not application-data rollback.** `/var` is shared by every deployment, so a
rolled-back application can find data written by a newer version of itself.

```text
release N+1 migrates DB to schema v8
             │
       probation fails → rollback
             │
release N starts, finds schema v8   ← your problem
```

## Prefer, in order

1. **Backwards-compatible formats.** Add fields and tables; don't repurpose or drop them in
   the same release. Old code ignores what it doesn't know.
2. **Expand/contract across releases.** Release N+1 writes both old and new shapes. Release
   N+2, once N+1 is known-good fleet-wide, stops writing the old shape.
3. **Reversible migrations.** Ship the down-migration in release N+1 and run it from release N
   when it sees a newer schema version it knows how to reverse.
4. **Versioned state directories.** Write `/var/lib/myapp/v8/` next to `v7/`. Rollback simply
   uses `v7/`. This trades disk space for safety.
5. **Btrfs snapshots** for products where the data volume justifies it. Snapshot before the
   migration, and restore on rollback via a product hook.

## When you can't make it safe

Declare it. In the flavor release metadata:

```toml
# flavor.toml
[release]
rollback_unsafe = true
rollback_unsafe_reason = "tsd-processing migrates survey index to v8 irreversibly"
```

Effects:

- `avena system rollback` from this release warns and requires `--yes`.
- Automatic rollback **after** the migration has run is suppressed. The device stays
  `degraded` and reports, rather than silently booting software that will corrupt data.
- Fleet tooling flags the release so that canary exposure is planned accordingly.

## Checklist

- [ ] New version reads old data.
- [ ] Old version tolerates new data, *or* the release is marked `rollback_unsafe`.
- [ ] Migration is idempotent (it may run again after a crash).
- [ ] Migration runs after probation starts, not in the image build.
- [ ] Tested with: upgrade → rollback → upgrade.
