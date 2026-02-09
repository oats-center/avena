# Avena Testing Guide

This guide covers the two-tier testing infrastructure for Avena.

## Testing Tiers

| Tier | Tool | Speed | Use Case |
|------|------|-------|----------|
| **Unit/Integration** | `cargo test` | Fast | HLC, message routing, cluster behavior |
| **Tier 1 (Dev)** | `dev-harness.sh` | Fast | Podman pods with Toxiproxy chaos |
| **Tier 2 (System)** | `nspawn-harness.sh` / `bootc-harness.sh` | Slow | Full systemd, production-like |

## Quick Start

### Unit & Integration Tests

```bash
# Run all tests (requires nats-server in PATH)
cargo test --workspace

# Run with chaos features
cargo test --workspace --features avena-test/chaos
```

Tests automatically skip if `nats-server` is not installed.

### Tier 1: Dev Harness (Fast)

```bash
cargo build
./scripts/dev-harness.sh up 3
./scripts/dev-harness.sh status
./scripts/dev-harness.sh down
```

Exposes:
- NATS hub at `localhost:4222`
- Device NATS at `localhost:14222`, `14223`, etc.
- Toxiproxy API at `localhost:8474`

### Tier 2: System Harness (Production-like)

```bash
cargo build
sudo ./scripts/nspawn-harness.sh up 3
sudo ./scripts/nspawn-harness.sh exec dev1
sudo ./scripts/nspawn-harness.sh down
```

See [BOOTC-TESTING.md](BOOTC-TESTING.md) for bootc-based testing.

---

## avena-test Crate

The `avena-test` crate provides test infrastructure:

```rust
use avena_test::cluster::TestCluster;

#[tokio::test]
async fn my_test() {
    // Independent nodes (no connectivity)
    let cluster = TestCluster::new(3).unwrap();

    // Hub-spoke topology (nodes communicate via hub)
    let cluster = TestCluster::with_hub(3).unwrap();

    // Connect clients
    let nc = cluster.connect_nats("node1").await.unwrap();
    let avena = cluster.connect_avena("node1").await.unwrap();
}
```

### Chaos Testing (Toxiproxy)

Enable the `chaos` feature:

```toml
[dev-dependencies]
avena-test = { path = "../avena-test", features = ["chaos"] }
```

```rust
use avena_test::chaos::{Toxiproxy, Proxy, Toxic, Direction};

let proxy = Toxiproxy::localhost();

// Create proxy between components
proxy.create_proxy(&Proxy::new("nats-proxy", "localhost:5555", "localhost:4222")).await?;

// Inject latency
proxy.add_toxic("nats-proxy", Toxic::latency(100, 20, Direction::Downstream)).await?;

// Simulate partition (infinite timeout)
proxy.add_toxic("nats-proxy", Toxic::timeout(0, Direction::Upstream)).await?;

// Reset all
proxy.reset().await?;
```

---

## Chaos Script (Tier 2)

For nspawn/bootc harnesses, use `chaos.sh`:

```bash
# Network partitions
sudo ./scripts/chaos.sh partition dev1 dev2
sudo ./scripts/chaos.sh heal dev1 dev2
sudo ./scripts/chaos.sh isolate dev1
sudo ./scripts/chaos.sh unisolate dev1

# Network degradation
sudo ./scripts/chaos.sh latency dev1 100 20    # 100ms ± 20ms
sudo ./scripts/chaos.sh loss dev1 10           # 10% packet loss
sudo ./scripts/chaos.sh bandwidth dev1 1000    # 1000 kbps limit

# Reset all
sudo ./scripts/chaos.sh reset
sudo ./scripts/chaos.sh status
```

---

## Test Categories

### Unit Tests (`avena/src/`)

- HLC ordering and merging (`hlc.rs`)
- Message serialization

### Integration Tests (`avenad/tests/`)

| File | Tests |
|------|-------|
| `ping_status.rs` | Basic handler round-trip |
| `cluster_discovery.rs` | Pub/sub routing via hub |
| `hlc_distributed.rs` | HLC sync across nodes |
| `jetstream_kv.rs` | KV replication |
| `broadcast_discovery.rs` | Device discovery |
| `chaos_scenarios.rs` | Network fault tolerance |

### Running Specific Tests

```bash
# Single test file
cargo test --package avenad --test hlc_distributed

# Single test function
cargo test --package avenad test_hlc_sync_across_nodes

# With output
cargo test --package avenad -- --nocapture
```

---

## Multi-Device Testing (Tier 2)

### Using nspawn-harness

Each container:
- Runs Fedora 43 minimal
- Has avenad as systemd service
- Manages its own NATS server with JWT auth
- Connected via bridge `avena-br0` (10.42.42.0/24)
- IPs: dev1=10.42.42.11, dev2=10.42.42.12, etc.

```bash
# Start environment
sudo ./scripts/nspawn-harness.sh up 3

# Execute commands
sudo ./scripts/nspawn-harness.sh exec dev1 avenactl devices list
sudo ./scripts/nspawn-harness.sh logs dev1

# Test link registration
sudo ./scripts/nspawn-harness.sh exec dev1 \
  avenactl link add --to nats://10.42.42.12:4222

# Simulate partition during test
sudo ./scripts/chaos.sh partition dev1 dev2
# ... observe behavior ...
sudo ./scripts/chaos.sh heal dev1 dev2

# Cleanup
sudo ./scripts/nspawn-harness.sh down
```

### Directory Structure

```
/var/tmp/avena-nspawn/
└── machines/
    ├── dev1/
    │   ├── nats/cfg/         # JWT credentials
    │   └── var/lib/avena/    # Device identity
    ├── dev2/
    └── dev3/
```

---

## Troubleshooting

**Tests skip with "nats-server not found":**
```bash
# Install nats-server
# Fedora/RHEL
dnf install nats-server

# Or download binary
curl -L https://github.com/nats-io/nats-server/releases/download/v2.10.20/nats-server-v2.10.20-linux-amd64.tar.gz | tar xz
sudo mv nats-server-*/nats-server /usr/local/bin/
```

**Toxiproxy tests skip:**
```bash
# Start toxiproxy
podman run -d -p 8474:8474 ghcr.io/shopify/toxiproxy:latest
```

**Container won't start:**
```bash
machinectl list
sudo machinectl terminate dev1
```

**Network issues:**
```bash
ip addr show avena-br0
sudo ./scripts/nspawn-harness.sh down
sudo ./scripts/nspawn-harness.sh up 3
```
