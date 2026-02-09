#!/usr/bin/env bash
set -euo pipefail

# Dev harness using systemd-nspawn for proper avenad testing with JWT-authenticated NATS.
# Each device runs avenad normally (managing its own local NATS via systemd).
# Hub NATS acts as central point for device discovery and linking.

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

TMP_DIR="${TMP_DIR:-/tmp/avena-harness}"
MACHINES_DIR="$TMP_DIR/machines"
CREDS_DIR="$TMP_DIR/creds"
HUB_DIR="$TMP_DIR/hub"
ROOTFS_DIR="$TMP_DIR/rootfs"

AVENAD_BIN="${AVENAD_BIN:-$PROJECT_ROOT/target/debug/avenad}"
AVENACTL_BIN="${AVENACTL_BIN:-$PROJECT_ROOT/target/debug/avenactl}"
KEYGEN_BIN="${KEYGEN_BIN:-$PROJECT_ROOT/target/debug/avena-keygen}"

NATS_IMAGE="docker.io/library/nats:2.10"
BASE_IMAGE="registry.fedoraproject.org/fedora:43"

BRIDGE_NAME="avena-dev"
BRIDGE_SUBNET="10.43.43.0/24"
BRIDGE_IP="10.43.43.1"
HUB_IP="10.43.43.2"

usage() {
  cat <<EOF
Usage: $0 <command> [args]

Commands:
  setup                 Build rootfs and generate credentials (run once)
  up <count>            Start hub + <count> device containers
  down                  Stop all containers and cleanup
  exec <name> [cmd]     Execute command in device (default: bash)
  logs <name>           Show avenad logs for device
  hub-logs              Show hub NATS logs
  status                Show running machines
  host-config           Configure avenactl on host to connect to hub

Environment:
  TMP_DIR               Base directory (default: /tmp/avena-harness)
  AVENAD_BIN            Path to avenad binary
  AVENACTL_BIN          Path to avenactl binary
  KEYGEN_BIN            Path to avena-keygen binary

This harness uses systemd-nspawn to run devices with full systemd support,
allowing avenad to manage its own local NATS. Each device's NATS connects
as a leaf node to the hub using JWT authentication.

Example workflow:
  cargo build
  sudo $0 setup
  sudo $0 up 3
  $0 host-config          # Configure avenactl on host
  avenactl devices ls     # Query devices from host
  sudo $0 exec dev1 avenactl link add --to dev2
  sudo $0 down
EOF
}

require_root() {
  if [[ $EUID -ne 0 ]]; then
    echo "This command requires root. Try: sudo $0 $*" >&2
    exit 1
  fi
}

check_binaries() {
  for bin in "$AVENAD_BIN" "$AVENACTL_BIN" "$KEYGEN_BIN"; do
    if [[ ! -x "$bin" ]]; then
      echo "Error: Binary not found: $bin" >&2
      echo "Run 'cargo build' first" >&2
      exit 1
    fi
  done
}

setup_bridge() {
  if ip link show "$BRIDGE_NAME" &>/dev/null; then
    return
  fi

  echo "Creating bridge $BRIDGE_NAME at $BRIDGE_SUBNET"
  ip link add "$BRIDGE_NAME" type bridge
  ip addr add "$BRIDGE_IP/24" dev "$BRIDGE_NAME"
  ip link set "$BRIDGE_NAME" up

  iptables -t nat -A POSTROUTING -s "$BRIDGE_SUBNET" -j MASQUERADE 2>/dev/null || true
  echo 1 > /proc/sys/net/ipv4/ip_forward
}

teardown_bridge() {
  if ! ip link show "$BRIDGE_NAME" &>/dev/null; then
    return
  fi

  echo "Removing bridge $BRIDGE_NAME"
  ip link set "$BRIDGE_NAME" down 2>/dev/null || true
  ip link del "$BRIDGE_NAME" 2>/dev/null || true
  iptables -t nat -D POSTROUTING -s "$BRIDGE_SUBNET" -j MASQUERADE 2>/dev/null || true
}

cmd_setup() {
  require_root
  check_binaries

  echo "=== Setting up dev harness ==="

  # Create directories
  mkdir -p "$TMP_DIR" "$CREDS_DIR" "$HUB_DIR" "$MACHINES_DIR" "$ROOTFS_DIR"

  # Generate hub credentials
  echo "Generating hub NATS credentials..."
  "$KEYGEN_BIN" init --output "$CREDS_DIR"
  "$KEYGEN_BIN" hub-config \
    --creds-dir "$CREDS_DIR" \
    --leaf-port 7422 \
    --client-port 4222 \
    --output "$HUB_DIR/nats.conf"

  # Create Fedora rootfs if not exists
  if [[ ! -f "$ROOTFS_DIR/etc/os-release" ]]; then
    echo "Creating Fedora rootfs..."
    local temp_ctr="avena-rootfs-$$"
    podman create --name "$temp_ctr" "$BASE_IMAGE" /bin/bash
    podman export "$temp_ctr" | tar -C "$ROOTFS_DIR" -xf -
    podman rm "$temp_ctr"

    # Install required packages in rootfs
    systemd-nspawn -D "$ROOTFS_DIR" --pipe dnf install -y podman systemd iproute iputils

    # Enable lingering for user services
    mkdir -p "$ROOTFS_DIR/var/lib/systemd/linger"
    touch "$ROOTFS_DIR/var/lib/systemd/linger/root"
  fi

  echo ""
  echo "Setup complete. Files in $TMP_DIR:"
  echo "  creds/     - Hub NATS credentials"
  echo "  hub/       - Hub NATS config"
  echo "  rootfs/    - Base Fedora rootfs"
  echo ""
  echo "Next: sudo $0 up <count>"
}

start_hub() {
  echo "Starting hub NATS..."

  # Clean up any existing hub resources
  podman rm -f avena-hub 2>/dev/null || true
  ip link del veth-hub-br 2>/dev/null || true
  ip netns del avena-hub 2>/dev/null || true

  # Create network namespace for hub
  ip netns add avena-hub

  # Create veth pair connecting hub to bridge
  ip link add veth-hub type veth peer name veth-hub-br
  ip link set veth-hub netns avena-hub
  ip link set veth-hub-br master "$BRIDGE_NAME"
  ip link set veth-hub-br up

  # Configure hub network namespace
  ip netns exec avena-hub ip addr add "$HUB_IP/24" dev veth-hub
  ip netns exec avena-hub ip link set veth-hub up
  ip netns exec avena-hub ip link set lo up
  ip netns exec avena-hub ip route add default via "$BRIDGE_IP"

  # Create JetStream data directory
  mkdir -p "$HUB_DIR/jetstream"

  # Run hub NATS in the network namespace
  podman run -d \
    --name avena-hub \
    --network ns:/var/run/netns/avena-hub \
    -v "$HUB_DIR/nats.conf:/nats.conf:ro,Z" \
    -v "$CREDS_DIR:/creds:ro,Z" \
    -v "$HUB_DIR/jetstream:/data/jetstream:Z" \
    "$NATS_IMAGE" -c /nats.conf

  echo "Hub NATS started at $HUB_IP:4222 (clients) and $HUB_IP:7422 (leaf nodes)"

  # Also expose on host via port forwarding
  iptables -t nat -A PREROUTING -p tcp --dport 4222 -j DNAT --to-destination "$HUB_IP:4222" 2>/dev/null || true
  iptables -t nat -A PREROUTING -p tcp --dport 7422 -j DNAT --to-destination "$HUB_IP:7422" 2>/dev/null || true
  iptables -A FORWARD -p tcp -d "$HUB_IP" --dport 4222 -j ACCEPT 2>/dev/null || true
  iptables -A FORWARD -p tcp -d "$HUB_IP" --dport 7422 -j ACCEPT 2>/dev/null || true
}

stop_hub() {
  podman rm -f avena-hub 2>/dev/null || true
  ip link del veth-hub-br 2>/dev/null || true
  ip netns del avena-hub 2>/dev/null || true

  # Clean up port forwarding rules
  iptables -t nat -D PREROUTING -p tcp --dport 4222 -j DNAT --to-destination "$HUB_IP:4222" 2>/dev/null || true
  iptables -t nat -D PREROUTING -p tcp --dport 7422 -j DNAT --to-destination "$HUB_IP:7422" 2>/dev/null || true
  iptables -D FORWARD -p tcp -d "$HUB_IP" --dport 4222 -j ACCEPT 2>/dev/null || true
  iptables -D FORWARD -p tcp -d "$HUB_IP" --dport 7422 -j ACCEPT 2>/dev/null || true
}

start_device() {
  local name=$1
  local idx=$2
  local machine_dir="$MACHINES_DIR/$name"
  local ip_addr="10.43.43.$((10 + idx))"

  if machinectl show "$name" &>/dev/null 2>&1; then
    echo "Machine $name already running"
    return
  fi

  echo "Creating device $name..."

  # Create machine directory with overlay from rootfs
  rm -rf "$machine_dir"
  mkdir -p "$machine_dir"
  cp -a "$ROOTFS_DIR/." "$machine_dir/"

  # Set hostname
  echo "avena-$name" > "$machine_dir/etc/hostname"

  # Copy binaries
  cp "$AVENAD_BIN" "$machine_dir/usr/local/bin/avenad"
  cp "$AVENACTL_BIN" "$machine_dir/usr/local/bin/avenactl"
  chmod +x "$machine_dir/usr/local/bin/avenad" "$machine_dir/usr/local/bin/avenactl"

  # Generate device leaf credentials for NATS-to-NATS leaf connection
  "$KEYGEN_BIN" leaf-user \
    --account-dir "$CREDS_DIR" \
    --name "leaf-$name" \
    --output "$machine_dir/etc/avena/hub-leaf.creds"

  # Copy hub admin credentials for avenactl client access
  cp "$CREDS_DIR/avena-admin.creds" "$machine_dir/etc/avena/avena-admin.creds"

  # Create avenad as a user service (needs user systemd for podman/quadlet)
  mkdir -p "$machine_dir/root/.config/systemd/user"
  cat > "$machine_dir/root/.config/systemd/user/avenad.service" <<EOF
[Unit]
Description=Avena Device Daemon
After=default.target

[Service]
Type=simple
Environment=RUST_LOG=info
Environment=AVENA_DEVICE_ID=$name
Environment=AVENA_HUB_URL=nats://$HUB_IP:7422
Environment=AVENA_HUB_CREDS=/etc/avena/hub-leaf.creds
ExecStart=/usr/local/bin/avenad
Restart=on-failure
RestartSec=5

[Install]
WantedBy=default.target
EOF

  # Enable avenad user service
  mkdir -p "$machine_dir/root/.config/systemd/user/default.target.wants"
  ln -sf ../avenad.service \
    "$machine_dir/root/.config/systemd/user/default.target.wants/avenad.service"

  # Enable lingering so user services start at boot
  mkdir -p "$machine_dir/var/lib/systemd/linger"
  touch "$machine_dir/var/lib/systemd/linger/root"

  # Create avenactl config pointing to hub with admin credentials
  mkdir -p "$machine_dir/root/.config/avena"
  cat > "$machine_dir/root/.config/avena/config.toml" <<EOF
active_context = "hub"

[context.hub]
name = "hub"
connection = "nats://$HUB_IP:4222"
creds = "/etc/avena/avena-admin.creds"
js_domain = "avena"
EOF

  echo "Starting $name at $ip_addr..."

  systemd-run \
    --unit="nspawn-$name" \
    --property=KillMode=mixed \
    --setenv=SYSTEMD_NSPAWN_LOCK=0 \
    systemd-nspawn \
      --machine="$name" \
      --directory="$machine_dir" \
      --boot \
      --network-bridge="$BRIDGE_NAME"

  # Wait for machine to be ready
  sleep 5
  local attempts=0
  while [[ $attempts -lt 30 ]]; do
    if systemd-run --machine="$name" --pipe /usr/bin/true </dev/null >/dev/null 2>&1; then
      break
    fi
    sleep 2
    ((attempts++))
  done

  # Configure network
  systemd-run --machine="$name" --pipe /usr/bin/bash -c "ip addr add $ip_addr/24 dev host0 || true" </dev/null >/dev/null 2>&1 || true
  systemd-run --machine="$name" --pipe /usr/bin/bash -c "ip link set host0 up || true" </dev/null >/dev/null 2>&1 || true
  systemd-run --machine="$name" --pipe /usr/bin/bash -c "ip route add default via $BRIDGE_IP || true" </dev/null >/dev/null 2>&1 || true
  systemd-run --machine="$name" --pipe /usr/bin/bash -c "echo 'nameserver 8.8.8.8' > /etc/resolv.conf" </dev/null >/dev/null 2>&1 || true

  echo "Started $name at $ip_addr"
}

stop_device() {
  local name=$1

  if ! machinectl show "$name" &>/dev/null 2>&1; then
    return
  fi

  echo "Stopping $name"
  machinectl terminate "$name" 2>/dev/null || true

  local attempts=0
  while [[ $attempts -lt 10 ]]; do
    if ! machinectl show "$name" &>/dev/null 2>&1; then
      break
    fi
    sleep 1
    ((attempts++))
  done

  machinectl poweroff "$name" 2>/dev/null || true
}

cmd_up() {
  require_root
  local count=${1:-}

  if [[ -z "$count" ]] || ! [[ "$count" =~ ^[0-9]+$ ]]; then
    echo "Error: valid count required" >&2
    usage
    exit 1
  fi

  check_binaries

  if [[ ! -f "$ROOTFS_DIR/etc/os-release" ]]; then
    echo "Error: Rootfs not found. Run 'sudo $0 setup' first" >&2
    exit 1
  fi

  if [[ ! -f "$CREDS_DIR/operator.jwt" ]]; then
    echo "Error: Credentials not found. Run 'sudo $0 setup' first" >&2
    exit 1
  fi

  setup_bridge
  start_hub

  for i in $(seq 1 "$count"); do
    start_device "dev$i" "$i"
  done

  echo ""
  echo "=== Environment ready ==="
  echo "Hub NATS: nats://localhost:4222"
  echo "Devices: dev1 - dev$count"
  echo ""
  echo "Usage:"
  echo "  sudo $0 exec dev1                    # Shell into device"
  echo "  sudo $0 exec dev1 avenactl devices ls  # List devices"
  echo "  sudo $0 logs dev1                    # View avenad logs"
  echo "  sudo $0 hub-logs                     # View hub NATS logs"
}

cmd_down() {
  require_root

  # Stop all device machines
  for machine in $(machinectl list --no-legend 2>/dev/null | awk '{print $1}' | grep '^dev[0-9]' || true); do
    stop_device "$machine"
  done

  stop_hub
  teardown_bridge

  echo "All containers stopped"
}

cmd_exec() {
  require_root
  local name=${1:-}

  if [[ -z "$name" ]]; then
    echo "Error: device name required" >&2
    usage
    exit 1
  fi
  shift

  local cmd_args=("$@")
  if [[ ${#cmd_args[@]} -eq 0 ]]; then
    # Interactive shell - use nsenter
    nsenter -t "$(machinectl show "$name" -p Leader --value)" -a /usr/bin/bash
  else
    # Non-interactive - use systemd-run --pipe
    systemd-run --machine="$name" --pipe "${cmd_args[@]}" </dev/null
  fi
}

cmd_logs() {
  require_root
  local name=${1:-}

  if [[ -z "$name" ]]; then
    echo "Error: device name required" >&2
    usage
    exit 1
  fi

  systemd-run --machine="$name" --pipe /usr/bin/journalctl --user -u avenad -n 50 --no-pager </dev/null
}

cmd_hub_logs() {
  podman logs -f avena-hub
}

cmd_status() {
  echo "=== Hub ==="
  podman ps --filter name=avena-hub --format "table {{.Names}}\t{{.Status}}\t{{.Ports}}"
  echo ""
  echo "=== Devices ==="
  machinectl list 2>/dev/null | grep -E '^dev[0-9]|^MACHINE' || echo "No devices running"
}

cmd_host_config() {
  # Set up avenactl config on host to connect to hub
  local config_dir="${XDG_CONFIG_HOME:-$HOME/.config}/avena"
  mkdir -p "$config_dir/nats"

  if [[ ! -f "$CREDS_DIR/avena-admin.creds" ]]; then
    echo "Error: Credentials not found. Run 'sudo $0 setup' first" >&2
    exit 1
  fi

  # Copy credentials
  cp "$CREDS_DIR/avena-admin.creds" "$config_dir/nats/"

  # Create config
  cat > "$config_dir/config.toml" <<EOF
active_context = "hub"

[context.hub]
name = "hub"
connection = "nats://$HUB_IP:4222"
creds = "$config_dir/nats/avena-admin.creds"
js_domain = "avena"
EOF

  echo "Host avenactl config created at $config_dir/config.toml"
  echo "Test with: avenactl devices ls"
}

# Main
cmd=${1:-}
shift || true

case "$cmd" in
  setup)
    cmd_setup "$@"
    ;;
  up)
    cmd_up "$@"
    ;;
  down)
    cmd_down "$@"
    ;;
  exec)
    cmd_exec "$@"
    ;;
  logs)
    cmd_logs "$@"
    ;;
  hub-logs)
    cmd_hub_logs "$@"
    ;;
  status)
    cmd_status "$@"
    ;;
  host-config)
    cmd_host_config "$@"
    ;;
  *)
    usage
    exit 1
    ;;
esac
