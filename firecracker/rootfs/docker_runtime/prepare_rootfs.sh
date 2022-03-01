#!/usr/bin/env bash
set -euo pipefail

# Mostly copied verbatim from resources/tests/setup_rootfs.sh in the Firecracker repo.
# (which is Apache 2.0)

prepare_fc_rootfs() {
    BUILD_DIR="$1"
    SSH_DIR="$BUILD_DIR/ssh"

    packages="udev systemd-sysv openssh-server iproute2"
    apt-get update
    apt-get install -y --no-install-recommends $packages

    # Set a hostname.
    echo "ubuntu-fc-uvm" > "/etc/hostname"

    # The serial getty service hooks up the login prompt to the kernel console at
    # ttyS0 (where Firecracker connects its serial console).
    # We'll set it up for autologin to avoid the login prompt.
    mkdir "/etc/systemd/system/serial-getty@ttyS0.service.d/"
cat <<EOF > "/etc/systemd/system/serial-getty@ttyS0.service.d/autologin.conf"
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin root -o '-p -- \\u' --keep-baud 115200,38400,9600 %I $TERM
EOF

    # Disable resolved and ntpd
    #
    rm -f /etc/systemd/system/multi-user.target.wants/systemd-resolved.service
    rm -f /etc/systemd/system/dbus-org.freedesktop.resolve1.service
    rm -f /etc/systemd/system/sysinit.target.wants/systemd-timesyncd.service

    # Generate key for ssh access from host
    if [ ! -f "$SSH_DIR/id_rsa" ]; then
        mkdir -p "$SSH_DIR"
        ssh-keygen -f "$SSH_DIR/id_rsa" -N ""
    fi
    mkdir -m 0600 -p "/root/.ssh/"
    cp "$SSH_DIR/id_rsa.pub" "/root/.ssh/authorized_keys"
}