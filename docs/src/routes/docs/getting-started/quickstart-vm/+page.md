---
title: "Quickstart: boot Avena in a VM"
---

In about ten minutes you'll boot the **`lab`** flavor in a local virtual machine, check its
status, and look at the components the agent is reconciling. The `lab` flavor is a small
general-purpose composition meant for testbeds and learning. It needs no special hardware.

:::note[Note]
You need a Linux host with KVM (`/dev/kvm`), `qemu-system-x86_64` or `virt-install`, and about
4 GiB of free RAM.
:::

## 1. Download the VM image

Every flavor release publishes installer artifacts built from the *same* OCI image that devices
update from: an ISO, a raw disk, and a qcow2 VM image.

```bash
curl -LO https://github.com/oats-center/avena-images/releases/latest/download/avena-lab-x86_64.qcow2
```

:::tip[Tip]
You can also build the disk yourself from the OCI image with
[bootc-image-builder](https://github.com/osbuild/bootc-image-builder):

```bash
sudo podman run --rm --privileged \
    -v ./output:/output \
    -v /var/lib/containers/storage:/var/lib/containers/storage \
    quay.io/centos-bootc/bootc-image-builder:latest \
    --type qcow2 ghcr.io/oats-center/avena-lab:latest
```
:::

## 2. Provide provisioning material

On first boot Avena looks for provisioning material: an SSH key for break-glass access, and
optionally Trellis enrollment credentials. For a VM, the easiest way to pass it in is a
cloud-init style seed drive:

```bash
mkdir -p seed
cat > seed/avena-provision.toml <<'EOF'
[identity]
name = "lab-01"

[access]
ssh_authorized_keys = ["ssh-ed25519 AAAA... you@laptop"]

# Leave [trellis] out to run standalone; the agent still serves
# Trellis locally so `avena` works on the device itself.
EOF

genisoimage -output seed.iso -volid AVENA-PROVISION -joliet -rock seed/
```

## 3. Boot it

```bash
virt-install \
  --name avena-lab-01 \
  --memory 4096 --vcpus 2 \
  --import --disk avena-lab-x86_64.qcow2 \
  --disk seed.iso,device=cdrom \
  --os-variant fedora-unknown \
  --network user,model=virtio,portForward=2222:22 \
  --graphics none
```

Watch the console. After systemd starts, the agent runs first-boot provisioning:

```text
avena-agent[812]: flavor lab 2026.10.1 (schema 1) loaded from /usr/lib/avena/flavor.toml
avena-agent[812]: identity: created device id av1q7m2…  name=lab-01
avena-agent[812]: hardware: 0 required, 2 detected
avena-agent[812]: network: ethernet healthy (underlay)
avena-agent[812]: trellis: local service online
avena-agent[812]: reconcile: 7 components, 7 actions, 0 blocked
avena-agent[812]: health: first boot, marking deployment known-good
avena-agent[812]: ready
```

## 4. Log in and look around

```bash
ssh -p 2222 core@localhost
```

```console
$ avena status
Avena Lab — lab-01

System
  Avena:       1.4.0
  Flavor:      lab 2026.10.1
  Image:       sha256:3ac1…
  Channel:     production
  Boot state:  healthy

Control
  Trellis:     local only
  Identity:    lab-01

Networking
  Ethernet:    healthy
  Tailscale:   disabled
  Overlay:     disabled

Components
  7 running
  0 degraded
  0 failed

Workloads
  lab-hello    running

Update
  current

Inhibitors
  none
```

List what the agent is managing:

```console
$ avena component list
ID                      TYPE      DESIRED   ACTUAL    HEALTH
core.trellis            systemd   running   running   healthy
core.journald-upload    systemd   stopped   stopped   n/a
network.ethernet        network   running   running   healthy
network.mdns            systemd   running   running   healthy
network.tailscale       network   stopped   stopped   n/a
network.avena-overlay   network   stopped   stopped   n/a
workload.lab-hello      quadlet   running   running   healthy
```

## 5. Try reconciliation

Stop a component behind Avena's back and see what the agent does:

```console
$ sudo systemctl stop lab-hello.service
$ avena component status workload.lab-hello
ID:        workload.lab-hello
Desired:   required-running
Actual:    running
Health:    healthy
Events:
  12s ago  actual state changed: running → stopped (external)
  12s ago  reconcile: start lab-hello.service
  11s ago  actual state changed: stopped → running
```

The flavor says `lab-hello` is required, so the agent started it again. Avena doesn't fight
you over components that are *optional*. Those it leaves in whatever state you put them in.

## What you just saw

- The VM booted an OCI image. Nothing was installed or configured by hand.
- The agent read `/usr/lib/avena/flavor.toml` and brought the machine to that state.
- The `avena` CLI talked to the agent over Trellis, even though this machine isn't connected
  to any fleet.
