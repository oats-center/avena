# Avena Design Document

## Overview

Avena is a distributed device management system for edge computing environments. It enables fleet management of devices (e.g., agricultural equipment, remote sensors) that may operate in disconnected or intermittently connected scenarios.

## Core Components

### avenad (Device Daemon)

The daemon runs on each managed device and is responsible for:

1. **Local NATS Server Management**
   - Runs a NATS server as a Podman container via systemd quadlet
   - Manages NATS JWT authentication (operator/account/user credentials)
   - Configures leaf node connections to peer devices

2. **Device Identity**
   - Generates and persists device keypair (ed25519)
   - Signs messages for authentication during link handshakes
   - Stores identity at `~/.local/share/avena/device.json`

3. **Workload Reconciliation**
   - Watches JetStream KV for desired workload state
   - Generates quadlet container files
   - Manages systemd units via D-Bus

4. **Device Discovery**
   - Broadcasts announce messages periodically
   - Listens for peer announcements
   - Maintains device registry in JetStream KV

### avenactl (CLI Tool)

The CLI tool for operators to manage the device fleet:

1. **Context Management** - Multiple NATS connection profiles
2. **Device Operations** - Ping, status, list devices
3. **Workload Management** - Apply, delete, start/stop, logs, history
4. **Link Management** - Connect devices into mesh networks

### avena (Library)

Shared library providing:
- NATS client wrapper with JetStream support
- Message type definitions
- Hybrid Logical Clock (HLC) implementation
- Device API methods

## Architecture

```
┌─────────────┐     ┌─────────────┐     ┌─────────────┐
│   Device A  │     │   Device B  │     │   Device C  │
│  (avenad)   │────▶│  (avenad)   │◀────│  (avenad)   │
│             │     │             │     │             │
│ ┌─────────┐ │     │ ┌─────────┐ │     │ ┌─────────┐ │
│ │  NATS   │ │     │ │  NATS   │ │     │ │  NATS   │ │
│ │ Server  │ │     │ │ Server  │ │     │ │ Server  │ │
│ └─────────┘ │     │ └─────────┘ │     │ └─────────┘ │
└─────────────┘     └─────────────┘     └─────────────┘
       │                  │
       │    NATS Leaf     │
       │    Connections   │
       └──────────────────┘
              │
              ▼
        ┌───────────┐
        │ avenactl  │
        └───────────┘
```

## Key Design Decisions

### NATS for Communication

- **Why NATS**: Lightweight, supports pub/sub and request/reply, built-in clustering via leaf nodes
- **JetStream KV**: Provides durable storage with watch capability for reconciliation
- **JWT Auth**: Each device runs in operator mode with its own credentials

### Hybrid Logical Clock (HLC)

HLC provides causally-ordered timestamps across distributed nodes:

```rust
HybridTimestamp {
    wall_time_ms: u64,  // Wall clock (or max seen)
    counter: u32,       // Increments when wall clock hasn't advanced
    node_id: String,    // Tie-breaker for concurrent events
}
```

**Sync Mechanism:**
- Every NATS message includes HLC in `Avena-HLC` header
- On receive: merge remote HLC with local (`max(local, remote) + 1`)
- On send: attach current HLC to outgoing message
- Persistence: Saved to `~/.local/share/avena/hlc.json` every 60s

**Conflict Detection:**
- Workload specs include HLC timestamp and issuer
- Apply rejects if existing spec has newer timestamp
- Use `--force` to override conflict check (records `forced: true` for audit)

### Link System

Devices connect via authenticated handshakes:
1. Device A sends LinkOffer with signed nonce
2. Device B verifies signature, generates user credentials
3. Device B sends LinkAccept with credentials
4. Device A configures NATS leaf node connection
5. Both devices can now communicate via NATS mesh

### Workload Model

Workloads are declarative specs stored in JetStream KV:
- Key pattern: `device/{device-id}/{workload-name}`
- Value: Complete `WorkloadDesiredState` (no partial updates)
- History: Last 10 versions retained for audit

Reconciliation is level-triggered:
1. Watch detects any change to device's workload keys
2. Full reconciliation runs (compare desired vs actual)
3. Deploy/update/remove workloads as needed

## Goals

### Current

- **Simple fleet management**: Ping, status, workload deployment
- **Device meshing**: Connect devices via NATS leaf nodes
- **Offline tolerance**: Devices operate independently, sync when connected
- **Conflict detection**: HLC timestamps prevent silent overwrites

### Future

- **Controller authentication**: Sign workload specs, verify trust chains
- **Wireguard tunnels**: Secure NATS leaf node connections
- **Richer workload model**: Resource limits, health checks, dependencies
- **Fleet-wide queries**: Aggregate status across all devices

## Non-Goals

- Not a full container orchestration platform (no scheduling)
- Not a replacement for k8s (much simpler, edge-focused)
- Not real-time (eventual consistency is acceptable)

## Data Model

### Device Identity
```rust
struct DeviceIdentity {
    id: String,           // UUID
    pubkey: String,       // ed25519 public key
    seed: String,         // ed25519 seed (private)
    network_token: Option<String>,  // Shared secret for network membership
}
```

### Workload Spec
```rust
struct WorkloadDesiredState {
    name: String,
    spec: WorkloadSpec,
    timestamp: Option<HybridTimestamp>,
    issuer: Option<String>,
    forced: bool,
}

struct WorkloadSpec {
    image: String,
    tag: Option<String>,
    cmd: Option<String>,
    args: Vec<String>,
    env: Vec<(String, String)>,
    mounts: Vec<MountSpec>,
    ports: Vec<PortSpec>,
    volumes: Vec<String>,
    // Future: devices, perms (currently unused)
}
```

### Link Entry
```rust
struct LinkEntry {
    url: String,
    creds_path: Option<String>,
    inline_creds: Option<String>,
}
```

## CLI Examples

```bash
# Connect to a context
avenactl context add local --connection localhost:4222
avenactl context use local

# List devices
avenactl devices ls

# Deploy a workload
avenactl devices workload apply dev1 nginx docker.io/nginx:latest \
    --port 80:8080 \
    --mount /data:/var/www:ro

# Check workload history
avenactl devices workload history dev1 nginx

# Link two devices
avenactl link add --from dev1 --to nats://10.0.0.2:4222
```

## Testing

- **Unit tests**: HLC ordering, message serialization
- **Integration tests**: Ping/status with ephemeral NATS server
- **Bootc harness**: Full multi-device simulation with systemd-nspawn

## Open Questions for Discussion

1. **Workload dependencies**: Should workloads be able to declare dependencies on other workloads?
2. **Fleet-wide operations**: How to apply a workload to multiple devices atomically?
3. **Health monitoring**: Should avenad report workload health, or leave that to external systems?
4. **Network partitions**: How long should devices retain workload specs when disconnected?
5. **Upgrade strategy**: How to upgrade avenad itself across the fleet?
