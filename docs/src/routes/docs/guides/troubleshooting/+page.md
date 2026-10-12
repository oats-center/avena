---
title: "Troubleshooting and diagnostics"
---

## Start here

```bash
avena status
avena diagnostics health
```

## Common situations

### A component is `blocked`

```bash
avena component status <id>
```

The `Reason:` line says what it's waiting for: missing hardware, a failed dependency, or an
unavailable image. Fix the cause, then:

```bash
avena hardware rescan      # if it was hardware
avena flavor reconcile
```

### A workload keeps restarting

```bash
avena workload status <name>
avena workload logs <name> -n 300
```

If it's flavor-bound and started failing after an update, check whether the device is still in
probation (`avena system status`). Avena may be about to roll back.

### The device rolled back on its own

```bash
avena system status
avena diagnostics summary --update-history
```

`Last update: … reason:` gives the failed check. The target digest is now blocked from automatic
reinstall. After fixing the release, publish a **new** digest rather than unblocking the old one.

### An update won't apply

```bash
avena system update status
avena system inhibitors
```

Usually an active inhibitor, or you're outside the maintenance window. Inhibitors that seem
stuck are leases. They expire if the holder stops renewing. A holder that keeps renewing is
telling you that it's busy.

### Trellis is disconnected

```bash
avena network providers
avena network provider tailscale status
avena logs --component core.trellis
```

Local control keeps working. The CLI on the device talks to the local agent.

### "Is this device stock?"

```bash
avena dev status
```

Any active overrides are listed. `avena status` also shows a `DEV OVERRIDES ACTIVE` banner.

## Break glass

If the agent itself is broken, use the underlying tools directly over SSH or the console.
They're the supported recovery path:

```bash
sudo bootc status
sudo bootc rollback --apply
sudo systemctl status avena-agent
sudo journalctl -u avena-agent -b
```

## Support bundles

```bash
avena diagnostics support-bundle -o bundle.tar.zst
```

Contents: Avena/flavor versions and image digest, bootc status, component state, failed systemd
units, networking state, Trellis state, journal excerpts, hardware inventory, update history,
and the most recent rollback reason. Credentials, keys, and tokens are redacted by design.
