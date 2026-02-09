#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

HARNESS_DIR="${HARNESS_DIR:-/var/tmp/avena-nspawn}"
MACHINES_DIR="$HARNESS_DIR/machines"
BRIDGE_NAME="avena-br0"
BRIDGE_SUBNET="10.42.42.0/24"
BRIDGE_IP="10.42.42.1"
NATS_IMAGE="docker.io/library/nats:2.10.20"

AVENAD_BIN="${AVENAD_BIN:-$PROJECT_ROOT/target/debug/avenad}"
AVENACTL_BIN="${AVENACTL_BIN:-$PROJECT_ROOT/target/debug/avenactl}"

usage() {
  cat <<EOF
Usage: $0 <command> [args]

Commands:
  bootstrap <name>       Bootstrap a Fedora rootfs for machine <name>
  up <count>            Start <count> devices with auto networking
  down                  Stop all containers and cleanup
  exec <name> [cmd]     Execute command in container (default: bash)
  logs <name>           Show systemd journal for container
  ip <name>             Show IP address of container

Environment:
  HARNESS_DIR           Base directory (default: /var/tmp/avena-nspawn)
  AVENAD_BIN            Path to avenad binary
  AVENACTL_BIN          Path to avenactl binary

Example:
  $0 up 3               # Start 3 devices
  $0 exec dev1          # Open shell in dev1
  $0 exec dev1 avenactl devices list  # Run avenactl command
  $0 down               # Cleanup
EOF
}

require_root() {
  if [[ $EUID -ne 0 ]]; then
    echo "This command requires root. Try: sudo $0 $*" >&2
    exit 1
  fi
}

setup_bridge() {
  if ip link show "$BRIDGE_NAME" &>/dev/null; then
    return
  fi

  echo "Creating bridge $BRIDGE_NAME at $BRIDGE_SUBNET"
  ip link add "$BRIDGE_NAME" type bridge
  ip addr add "$BRIDGE_IP/24" dev "$BRIDGE_NAME"
  ip link set "$BRIDGE_NAME" up

  iptables -t nat -A POSTROUTING -s "$BRIDGE_SUBNET" -j MASQUERADE || true
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

bootstrap_fedora() {
  local name=$1
  local rootfs="$MACHINES_DIR/$name"

  if [[ -d "$rootfs" ]]; then
    echo "Machine $name already exists at $rootfs"
    return
  fi

  echo "Bootstrapping Fedora for $name at $rootfs"
  mkdir -p "$rootfs"

  local image="registry.fedoraproject.org/fedora:43"
  local temp_ctr="avena-bootstrap-$$"

  echo "Pulling Fedora container image..."
  podman pull "$image" >/dev/null

  echo "Extracting rootfs..."
  podman create --name "$temp_ctr" "$image" /bin/bash >/dev/null
  podman export "$temp_ctr" | tar -C "$rootfs" -xf -
  podman rm "$temp_ctr" >/dev/null

  echo "Installing additional packages..."
  systemd-nspawn -D "$rootfs" --quiet \
    dnf install -y --setopt=install_weak_deps=False \
      systemd \
      podman \
      iproute \
      iputils \
      procps-ng \
      findutils \
      vim-minimal

  echo "avena-$name" > "$rootfs/etc/hostname"

  mkdir -p "$rootfs/nats/cfg"
  mkdir -p "$rootfs/var/lib/avena"
  mkdir -p "$rootfs/etc/containers/systemd/nats"

  cp "$AVENAD_BIN" "$rootfs/usr/local/bin/avenad"
  chmod +x "$rootfs/usr/local/bin/avenad"

  cp "$AVENACTL_BIN" "$rootfs/usr/local/bin/avenactl"
  chmod +x "$rootfs/usr/local/bin/avenactl"

  cat > "$rootfs/etc/systemd/system/avenad.service" <<'EOF'
[Unit]
Description=Avena Device Daemon
After=network.target

[Service]
Type=simple
ExecStart=/usr/local/bin/avenad
Restart=always
RestartSec=5
Environment=RUST_LOG=info
Environment=AVENA_SKIP_LOCAL_NATS=0

[Install]
WantedBy=multi-user.target
EOF

  systemd-nspawn -D "$rootfs" --quiet \
    systemctl enable avenad.service

  echo "root:root" | systemd-nspawn -D "$rootfs" --quiet chpasswd

  echo "Bootstrapped $name"
}

start_machine() {
  local name=$1
  local ip_suffix=$2
  local rootfs="$MACHINES_DIR/$name"

  if [[ ! -d "$rootfs" ]]; then
    bootstrap_fedora "$name"
  fi

  if machinectl show "$name" &>/dev/null; then
    echo "Machine $name already running"
    return
  fi

  local ip_addr="10.42.42.$ip_suffix"

  echo "Starting $name at $ip_addr"

  systemd-nspawn \
    --machine="$name" \
    --directory="$rootfs" \
    --boot \
    --network-bridge="$BRIDGE_NAME" \
    --bind=/run/user/1000/podman:/run/podman:norbind \
    &

  sleep 2

  local attempts=0
  while [[ $attempts -lt 30 ]]; do
    if machinectl show "$name" &>/dev/null; then
      break
    fi
    sleep 1
    ((attempts++))
  done

  machinectl shell "$name" /usr/bin/bash -c "ip addr add $ip_addr/24 dev host0 || true"
  machinectl shell "$name" /usr/bin/bash -c "ip route add default via $BRIDGE_IP || true"

  echo "Started $name at $ip_addr"
}

stop_machine() {
  local name=$1

  if ! machinectl show "$name" &>/dev/null; then
    return
  fi

  echo "Stopping $name"
  machinectl terminate "$name" 2>/dev/null || true

  local attempts=0
  while [[ $attempts -lt 10 ]]; do
    if ! machinectl show "$name" &>/dev/null; then
      break
    fi
    sleep 1
    ((attempts++))
  done

  machinectl poweroff "$name" 2>/dev/null || true
}

cmd_bootstrap() {
  require_root
  local name=${1:-}
  if [[ -z "$name" ]]; then
    echo "Error: machine name required" >&2
    usage
    exit 1
  fi

  mkdir -p "$MACHINES_DIR"
  bootstrap_fedora "$name"
}

cmd_up() {
  require_root
  local count=${1:-}
  if [[ -z "$count" ]] || ! [[ "$count" =~ ^[0-9]+$ ]]; then
    echo "Error: valid count required" >&2
    usage
    exit 1
  fi

  if [[ ! -x "$AVENAD_BIN" ]]; then
    echo "Error: avenad binary not found at $AVENAD_BIN" >&2
    echo "Build it first: cargo build" >&2
    exit 1
  fi

  if [[ ! -x "$AVENACTL_BIN" ]]; then
    echo "Error: avenactl binary not found at $AVENACTL_BIN" >&2
    echo "Build it first: cargo build" >&2
    exit 1
  fi

  mkdir -p "$MACHINES_DIR"
  setup_bridge

  for i in $(seq 1 "$count"); do
    local name="dev$i"
    local ip_suffix=$((10 + i))
    start_machine "$name" "$ip_suffix"
  done

  echo ""
  echo "Environment ready. Example usage:"
  echo "  sudo $0 exec dev1"
  echo "  sudo $0 exec dev1 journalctl -u avenad -f"
  echo "  sudo $0 ip dev1"
}

cmd_down() {
  require_root

  for machine in $(machinectl list --no-legend | awk '{print $1}' | grep '^dev[0-9]' || true); do
    stop_machine "$machine"
  done

  teardown_bridge

  echo "Stopped all machines"
}

cmd_exec() {
  require_root
  local name=${1:-}
  if [[ -z "$name" ]]; then
    echo "Error: machine name required" >&2
    usage
    exit 1
  fi
  shift

  local cmd_args=("$@")
  if [[ ${#cmd_args[@]} -eq 0 ]]; then
    cmd_args=("/usr/bin/bash")
  fi

  machinectl shell "$name" "${cmd_args[@]}"
}

cmd_logs() {
  require_root
  local name=${1:-}
  if [[ -z "$name" ]]; then
    echo "Error: machine name required" >&2
    usage
    exit 1
  fi

  machinectl shell "$name" /usr/bin/journalctl -u avenad -f
}

cmd_ip() {
  local name=${1:-}
  if [[ -z "$name" ]]; then
    echo "Error: machine name required" >&2
    usage
    exit 1
  fi

  machinectl show "$name" -p Addresses --value 2>/dev/null || echo "Machine not running"
}

cmd=${1:-}
shift || true

case "$cmd" in
  bootstrap)
    cmd_bootstrap "$@"
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
  ip)
    cmd_ip "$@"
    ;;
  *)
    usage
    exit 1
    ;;
esac
