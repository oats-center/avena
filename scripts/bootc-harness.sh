#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "$SCRIPT_DIR/.." && pwd)"

HARNESS_DIR="${HARNESS_DIR:-/var/tmp/avena-bootc}"
IMAGE_DIR="$HARNESS_DIR/images"
ROOTFS_DIR="$HARNESS_DIR/rootfs"
MACHINES_DIR="$HARNESS_DIR/machines"
BRIDGE_NAME="avena-br0"
BRIDGE_SUBNET="10.42.42.0/24"
BRIDGE_IP="10.42.42.1"

IMAGE_NAME="${IMAGE_NAME:-localhost/avena-device:latest}"
DISK_IMAGE="$IMAGE_DIR/disk.raw"
BUILD_PROFILE="${BUILD_PROFILE:-debug}"

usage() {
  cat <<EOF
Usage: $0 <command> [args]

Commands:
  build                 Build bootc image and extract rootfs (run once)
  rebuild               Rebuild container and extract rootfs (fast iteration)
  up <count>            Start <count> devices with auto networking
  down                  Stop all containers and cleanup
  exec <name> [cmd]     Execute command in container (default: bash)
  logs <name>           Show systemd journal for container
  ip <name>             Show IP address of container

Environment:
  HARNESS_DIR           Base directory (default: /var/tmp/avena-bootc)
  IMAGE_NAME            Container image name (default: localhost/avena-device:latest)
  BUILD_PROFILE         Cargo profile to use (default: debug)

Example:
  $0 build              # Build bootc image (run once)
  $0 up 3               # Start 3 devices

  # Make code changes:
  cargo build
  $0 rebuild            # Rebuild container & extract rootfs
  $0 down && $0 up 3    # Restart machines

  $0 exec dev1          # Open shell in dev1
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

cmd_build() {
  require_root

  local bin_dir="$PROJECT_ROOT/target/$BUILD_PROFILE"

  if [[ ! -f "$bin_dir/avenad" ]] || [[ ! -f "$bin_dir/avenactl" ]]; then
    echo "Error: Binaries not found in $bin_dir" >&2
    echo "Run 'cargo build' first (or 'cargo build --release' for BUILD_PROFILE=release)" >&2
    exit 1
  fi

  echo "Building bootc container image: $IMAGE_NAME (profile: $BUILD_PROFILE)"
  podman build -t "$IMAGE_NAME" \
    --no-cache \
    --build-arg BUILD_PROFILE="$BUILD_PROFILE" \
    -f "$PROJECT_ROOT/Containerfile" \
    "$PROJECT_ROOT"

  mkdir -p "$IMAGE_DIR"

  echo "Creating bootable disk image: $DISK_IMAGE"
  rm -rf "$IMAGE_DIR/output"

  podman run --rm \
    --privileged \
    --pull=newer \
    --security-opt label=type:unconfined_t \
    -v "$IMAGE_DIR:/output:Z" \
    -v /var/lib/containers/storage:/var/lib/containers/storage \
    quay.io/centos-bootc/bootc-image-builder:latest \
    --type raw \
    --rootfs xfs \
    "$IMAGE_NAME"

  mv "$IMAGE_DIR/image/disk.raw" "$IMAGE_DIR/disk.raw"
  rm -rf "$IMAGE_DIR/image" "$IMAGE_DIR/manifest.json"

  echo "Extracting rootfs from disk image..."
  rm -rf "$ROOTFS_DIR"
  mkdir -p "$ROOTFS_DIR"

  local loop_dev=$(losetup -f)
  losetup -P "$loop_dev" "$IMAGE_DIR/disk.raw"

  local root_part="${loop_dev}p4"
  if [[ ! -e "$root_part" ]]; then
    root_part="${loop_dev}p3"
  fi

  mount "$root_part" /mnt
  cp -a /mnt/ostree/deploy/*/deploy/*/. "$ROOTFS_DIR/" 2>/dev/null || cp -a /mnt/. "$ROOTFS_DIR/"
  umount /mnt
  losetup -d "$loop_dev"

  echo "Build complete: $ROOTFS_DIR"
}

cmd_rebuild() {
  require_root

  local bin_dir="$PROJECT_ROOT/target/$BUILD_PROFILE"

  if [[ ! -f "$bin_dir/avenad" ]] || [[ ! -f "$bin_dir/avenactl" ]]; then
    echo "Error: Binaries not found in $bin_dir" >&2
    echo "Run 'cargo build' first" >&2
    exit 1
  fi

  echo "Rebuilding container image: $IMAGE_NAME (profile: $BUILD_PROFILE)"
  podman build -t "$IMAGE_NAME" \
    --build-arg BUILD_PROFILE="$BUILD_PROFILE" \
    -f "$PROJECT_ROOT/Containerfile" \
    "$PROJECT_ROOT"

  local temp_ctr="avena-extract-$$"
  podman create --name "$temp_ctr" "$IMAGE_NAME" /bin/bash

  echo "Re-extracting rootfs..."
  rm -rf "$ROOTFS_DIR"
  mkdir -p "$ROOTFS_DIR"
  podman export "$temp_ctr" | tar -C "$ROOTFS_DIR" -xf -
  podman rm "$temp_ctr"

  echo "Rebuild complete. Restart machines:"
  echo "  sudo $0 down && sudo $0 up <count>"
}

start_machine() {
  local name=$1
  local ip_suffix=$2
  local machine_dir="$MACHINES_DIR/$name"

  if machinectl show "$name" &>/dev/null; then
    echo "Machine $name already running"
    return
  fi

  if [[ ! -d "$machine_dir" ]] || [[ ! -f "$machine_dir/etc/os-release" ]]; then
    echo "Creating machine directory for $name"
    rm -rf "$machine_dir"
    mkdir -p "$machine_dir"
    cp -a "$ROOTFS_DIR/." "$machine_dir/"
    echo "avena-$name" > "$machine_dir/etc/hostname"
  fi

  local ip_addr="10.42.42.$ip_suffix"

  echo "Starting $name at $ip_addr"

  systemd-run \
    --unit="nspawn-$name" \
    --property=KillMode=mixed \
    --setenv=SYSTEMD_NSPAWN_LOCK=0 \
    systemd-nspawn \
      --machine="$name" \
      --directory="$machine_dir" \
      --boot \
      --network-bridge="$BRIDGE_NAME"

  sleep 5

  local attempts=0
  while [[ $attempts -lt 60 ]]; do
    if machinectl show "$name" &>/dev/null 2>&1; then
      if machinectl shell "$name" /usr/bin/true &>/dev/null 2>&1; then
        break
      fi
    fi
    sleep 1
    ((attempts++))
  done

  machinectl shell "$name" /usr/bin/bash -c "ip addr add $ip_addr/24 dev host0 || true" 2>/dev/null || true
  machinectl shell "$name" /usr/bin/bash -c "ip route add default via $BRIDGE_IP || true" 2>/dev/null || true

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

cmd_up() {
  require_root
  local count=${1:-}
  if [[ -z "$count" ]] || ! [[ "$count" =~ ^[0-9]+$ ]]; then
    echo "Error: valid count required" >&2
    usage
    exit 1
  fi

  if [[ ! -d "$ROOTFS_DIR" ]]; then
    echo "Error: Rootfs not found at $ROOTFS_DIR" >&2
    echo "Build it first: sudo $0 build" >&2
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
  build)
    cmd_build "$@"
    ;;
  rebuild)
    cmd_rebuild "$@"
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
