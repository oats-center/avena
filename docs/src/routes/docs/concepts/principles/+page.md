---
title: "Design principles"
---

Judge every Avena change against these.

### One platform, many products
TSD, ISOBlue, roadside, and future systems are flavors of Avena, not separately maintained
Linux distributions.

### Reproducible system state
The host OS and platform are delivered as OCI images through bootc.

### Local autonomy
The agent can bring up and maintain the machine from its local flavor and state while remote
connectivity is down.

### Trellis is the control plane
Once communication is available, Avena is controlled and observed through Trellis, not a
parallel custom RPC stack.

### Networking is composable
Tailscale, the Avena overlay, local networking, and product-specific networking are components
a flavor selects.

### Containers are a tool, not a religion
Use Quadlet/Podman by default for application software. Use host services when hardware or
system ownership makes that cleaner.

### The agent reconciles; it does not imperatively script the world
Desired and actual state are explicit, and reconciliation is idempotent.

### Production is immutable; development is intentionally temporary
Developers can override binaries, workloads, and `/usr` and still return to the published
system.

### Updates are policy-aware
Downloading an update is separate from deciding when it's safe to reboot.

### Rollback is a system feature, not a data backup
Persistent application state has its own compatibility requirements.

### Build once, promote by digest
The artifact tested in candidate and canary is the artifact promoted to production.

---

## What Avena deliberately is not

- Kubernetes
- a generic multi-node scheduler
- a replacement for systemd
- a replacement for Podman
- a replacement for Trellis
- a second Trellis-like RPC framework
- a universal Linux package manager
- a giant privileged container
- a configuration-management system that keeps mutating the root filesystem
- a networking stack that assumes every deployment uses the same provider

Avena's value is the integration and policy layer that gives these field systems one coherent
way of operating.
