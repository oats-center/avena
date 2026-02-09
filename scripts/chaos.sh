#!/usr/bin/env bash
set -euo pipefail

# Network chaos injection for nspawn/bootc harness using tc/iptables.
# Requires root privileges.
#
# This script operates on the bridge interface used by the harnesses.
# Devices are assigned IPs in the 10.42.42.0/24 range:
#   - dev1 = 10.42.42.11
#   - dev2 = 10.42.42.12
#   - etc.

BRIDGE="${AVENA_BRIDGE:-avena-br0}"

usage() {
  cat <<EOF
Usage: $0 <command> [args]

Network chaos injection for nspawn/bootc harness.
Operates on bridge: $BRIDGE (override with AVENA_BRIDGE env var)

Commands:
  partition <dev1> <dev2>      Block traffic between two devices
  heal <dev1> <dev2>           Restore traffic between two devices
  isolate <dev>                Block all traffic to/from a device
  unisolate <dev>              Restore all traffic to/from a device

  latency <dev> <ms> [jitter]  Add latency to traffic TO a device
  loss <dev> <percent>         Add packet loss to traffic TO a device
  corrupt <dev> <percent>      Add packet corruption to traffic TO a device
  bandwidth <dev> <kbps>       Limit bandwidth to a device

  reset                        Clear all chaos rules (tc + iptables)
  status                       Show current chaos rules

Examples:
  $0 partition dev1 dev2       # dev1 and dev2 can't communicate
  $0 heal dev1 dev2            # restore communication
  $0 latency dev1 100 20       # 100ms ± 20ms latency to dev1
  $0 loss dev1 10              # 10% packet loss to dev1
  $0 reset                     # clear all rules

EOF
  exit 1
}

get_dev_ip() {
  local dev=$1
  local idx=${dev#dev}
  if [[ ! "$idx" =~ ^[0-9]+$ ]]; then
    echo "Invalid device name: $dev (expected devN)" >&2
    exit 1
  fi
  echo "10.42.42.$((10 + idx))"
}

require_root() {
  if [[ $EUID -ne 0 ]]; then
    echo "This script requires root privileges" >&2
    exit 1
  fi
}

check_bridge() {
  if ! ip link show "$BRIDGE" >/dev/null 2>&1; then
    echo "Bridge $BRIDGE does not exist. Is the harness running?" >&2
    exit 1
  fi
}

cmd_partition() {
  local dev1=$1 dev2=$2
  local ip1=$(get_dev_ip "$dev1")
  local ip2=$(get_dev_ip "$dev2")

  echo "Partitioning $dev1 ($ip1) <-> $dev2 ($ip2)"
  iptables -I FORWARD -s "$ip1" -d "$ip2" -j DROP 2>/dev/null || true
  iptables -I FORWARD -s "$ip2" -d "$ip1" -j DROP 2>/dev/null || true
}

cmd_heal() {
  local dev1=$1 dev2=$2
  local ip1=$(get_dev_ip "$dev1")
  local ip2=$(get_dev_ip "$dev2")

  echo "Healing partition $dev1 ($ip1) <-> $dev2 ($ip2)"
  iptables -D FORWARD -s "$ip1" -d "$ip2" -j DROP 2>/dev/null || true
  iptables -D FORWARD -s "$ip2" -d "$ip1" -j DROP 2>/dev/null || true
}

cmd_isolate() {
  local dev=$1
  local ip=$(get_dev_ip "$dev")

  echo "Isolating $dev ($ip)"
  iptables -I FORWARD -s "$ip" -j DROP 2>/dev/null || true
  iptables -I FORWARD -d "$ip" -j DROP 2>/dev/null || true
}

cmd_unisolate() {
  local dev=$1
  local ip=$(get_dev_ip "$dev")

  echo "Unisolating $dev ($ip)"
  iptables -D FORWARD -s "$ip" -j DROP 2>/dev/null || true
  iptables -D FORWARD -d "$ip" -j DROP 2>/dev/null || true
}

setup_tc_root() {
  if ! tc qdisc show dev "$BRIDGE" | grep -q "qdisc prio 1:"; then
    tc qdisc add dev "$BRIDGE" root handle 1: prio bands 16 2>/dev/null || true
  fi
}

get_band_for_ip() {
  local ip=$1
  local last_octet=${ip##*.}
  local band=$((last_octet - 10))
  if [[ $band -lt 1 || $band -gt 15 ]]; then
    band=1
  fi
  echo "$band"
}

cmd_latency() {
  local dev=$1 ms=$2 jitter=${3:-0}
  local ip=$(get_dev_ip "$dev")
  local band=$(get_band_for_ip "$ip")

  echo "Adding ${ms}ms (±${jitter}ms) latency to $dev ($ip)"
  setup_tc_root

  tc qdisc del dev "$BRIDGE" parent 1:$band 2>/dev/null || true
  tc qdisc add dev "$BRIDGE" parent 1:$band handle $((band * 10)): netem delay "${ms}ms" "${jitter}ms"
  tc filter add dev "$BRIDGE" protocol ip parent 1:0 prio 1 u32 match ip dst "$ip" flowid 1:$band 2>/dev/null || true
}

cmd_loss() {
  local dev=$1 percent=$2
  local ip=$(get_dev_ip "$dev")
  local band=$(get_band_for_ip "$ip")

  echo "Adding ${percent}% packet loss to $dev ($ip)"
  setup_tc_root

  tc qdisc del dev "$BRIDGE" parent 1:$band 2>/dev/null || true
  tc qdisc add dev "$BRIDGE" parent 1:$band handle $((band * 10)): netem loss "${percent}%"
  tc filter add dev "$BRIDGE" protocol ip parent 1:0 prio 1 u32 match ip dst "$ip" flowid 1:$band 2>/dev/null || true
}

cmd_corrupt() {
  local dev=$1 percent=$2
  local ip=$(get_dev_ip "$dev")
  local band=$(get_band_for_ip "$ip")

  echo "Adding ${percent}% packet corruption to $dev ($ip)"
  setup_tc_root

  tc qdisc del dev "$BRIDGE" parent 1:$band 2>/dev/null || true
  tc qdisc add dev "$BRIDGE" parent 1:$band handle $((band * 10)): netem corrupt "${percent}%"
  tc filter add dev "$BRIDGE" protocol ip parent 1:0 prio 1 u32 match ip dst "$ip" flowid 1:$band 2>/dev/null || true
}

cmd_bandwidth() {
  local dev=$1 kbps=$2
  local ip=$(get_dev_ip "$dev")
  local band=$(get_band_for_ip "$ip")

  echo "Limiting bandwidth to $dev ($ip) to ${kbps}kbps"
  setup_tc_root

  tc qdisc del dev "$BRIDGE" parent 1:$band 2>/dev/null || true
  tc qdisc add dev "$BRIDGE" parent 1:$band handle $((band * 10)): tbf rate "${kbps}kbit" burst 32kbit latency 400ms
  tc filter add dev "$BRIDGE" protocol ip parent 1:0 prio 1 u32 match ip dst "$ip" flowid 1:$band 2>/dev/null || true
}

cmd_reset() {
  echo "Resetting all chaos rules..."

  tc qdisc del dev "$BRIDGE" root 2>/dev/null || true

  while iptables -D FORWARD -j DROP 2>/dev/null; do :; done

  echo "Done"
}

cmd_status() {
  echo "=== iptables FORWARD rules ==="
  iptables -L FORWARD -n -v 2>/dev/null | grep -E "DROP|10\.42\.42\." || echo "(no rules)"
  echo ""
  echo "=== tc qdisc on $BRIDGE ==="
  tc qdisc show dev "$BRIDGE" 2>/dev/null || echo "(no qdiscs)"
  echo ""
  echo "=== tc filters on $BRIDGE ==="
  tc filter show dev "$BRIDGE" 2>/dev/null || echo "(no filters)"
}

if [[ $# -lt 1 ]]; then
  usage
fi

cmd=$1
shift

case "$cmd" in
  partition)
    [[ $# -lt 2 ]] && usage
    require_root
    check_bridge
    cmd_partition "$1" "$2"
    ;;
  heal)
    [[ $# -lt 2 ]] && usage
    require_root
    cmd_heal "$1" "$2"
    ;;
  isolate)
    [[ $# -lt 1 ]] && usage
    require_root
    check_bridge
    cmd_isolate "$1"
    ;;
  unisolate)
    [[ $# -lt 1 ]] && usage
    require_root
    cmd_unisolate "$1"
    ;;
  latency)
    [[ $# -lt 2 ]] && usage
    require_root
    check_bridge
    cmd_latency "$1" "$2" "${3:-0}"
    ;;
  loss)
    [[ $# -lt 2 ]] && usage
    require_root
    check_bridge
    cmd_loss "$1" "$2"
    ;;
  corrupt)
    [[ $# -lt 2 ]] && usage
    require_root
    check_bridge
    cmd_corrupt "$1" "$2"
    ;;
  bandwidth)
    [[ $# -lt 2 ]] && usage
    require_root
    check_bridge
    cmd_bandwidth "$1" "$2"
    ;;
  reset)
    require_root
    cmd_reset
    ;;
  status)
    cmd_status
    ;;
  *)
    usage
    ;;
esac
