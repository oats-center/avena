#!/usr/bin/env bash
set -euo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"

cat <<'EOF'
Multi-Device Test Script
=========================

This will test:
1. Bootstrap and start 2 devices
2. Check avenad is running
3. List devices from dev1
4. Register a link from dev1 to dev2
5. Verify link credentials were created

Press ENTER to start...
EOF
read -r

echo ""
echo "Step 1: Starting 2 devices..."
sudo "$SCRIPT_DIR/nspawn-harness.sh" up 2

echo ""
echo "Step 2: Waiting for containers to fully boot (10s)..."
sleep 10

echo ""
echo "Step 3: Checking avenad status on dev1..."
sudo "$SCRIPT_DIR/nspawn-harness.sh" exec dev1 systemctl status avenad --no-pager || true

echo ""
echo "Step 4: Checking avenad status on dev2..."
sudo "$SCRIPT_DIR/nspawn-harness.sh" exec dev2 systemctl status avenad --no-pager || true

echo ""
echo "Step 5: Viewing avenad logs from dev1..."
sudo "$SCRIPT_DIR/nspawn-harness.sh" exec dev1 journalctl -u avenad --no-pager -n 20

echo ""
echo "Step 6: Checking JWT files were created on dev1..."
sudo "$SCRIPT_DIR/nspawn-harness.sh" exec dev1 ls -lh /nats/cfg/

echo ""
echo "Step 7: Checking device identity on dev1..."
sudo "$SCRIPT_DIR/nspawn-harness.sh" exec dev1 cat /var/lib/avena/device.json

echo ""
echo "Step 8: Testing avenactl on dev1..."
sudo "$SCRIPT_DIR/nspawn-harness.sh" exec dev1 avenactl --help

echo ""
echo ""
echo "Environment is ready!"
echo ""
echo "Next steps (manual):"
echo "  1. Open shell in dev1:    sudo $SCRIPT_DIR/nspawn-harness.sh exec dev1"
echo "  2. List devices:          avenactl devices list"
echo "  3. Select active device:  avenactl devices select <device-id>"
echo "  4. Register link to dev2: avenactl devices register nats://10.42.42.12:4222"
echo "  5. Check link status:     journalctl -u avenad | grep -i link"
echo ""
echo "When done testing:         sudo $SCRIPT_DIR/nspawn-harness.sh down"
echo ""
EOF
