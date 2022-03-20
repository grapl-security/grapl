#!/usr/bin/env bash
set -euo pipefail

################################################################################
# `create_rootfs_image.sh` creates a bootstrapped directory, and then executes
# this file within a chroot on that bootstrapped directory;
# we then do some mutations on the new filesystem.
################################################################################

passwd -d root

echo "grapl-plugin" > /etc/hostname

# Set up a login terminal on the serial console (ttyS0)
# See https://github.com/firecracker-microvm/firecracker/blob/main/docs/rootfs-and-kernel-setup.md
mkdir /etc/systemd/system/serial-getty@ttyS0.service.d/
cat << EOF > /etc/systemd/system/serial-getty@ttyS0.service.d/autologin.conf
[Service]
ExecStart=
ExecStart=-/sbin/agetty --autologin root -o '-p -- \\u' --keep-baud 115200,38400,9600 %I $TERM
EOF

# Disable resolved and ntpd
# See https://github.com/firecracker-microvm/firecracker/blob/f2743bfd2e9f9740dd08e55cdb83ba7b94aabb69/resources/tests/setup_rootfs.sh#L46
rm -f /etc/systemd/system/multi-user.target.wants/systemd-resolved.service
rm -f /etc/systemd/system/dbus-org.freedesktop.resolve1.service
rm -f /etc/systemd/system/sysinit.target.wants/systemd-timesyncd.service
