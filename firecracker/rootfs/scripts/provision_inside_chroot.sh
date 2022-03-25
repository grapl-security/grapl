#!/usr/bin/env bash
set -euo pipefail
set -o xtrace

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")

################################################################################
# `create_rootfs_image.sh` creates a bootstrapped directory, and then executes
# this file within a chroot on that bootstrapped directory;
# we then do some mutations on the new filesystem.
################################################################################

passwd -d root

echo "grapl-plugin" > /etc/hostname

# Set up a login terminal on the serial console (ttyS0)
# See https://github.com/firecracker-microvm/firecracker/blob/main/docs/rootfs-and-kernel-setup.md
mkdir -p /etc/systemd/system/serial-getty@ttyS0.service.d/
cp "${THIS_DIR}/ttyS0_autologin.conf" \
    /etc/systemd/system/serial-getty@ttyS0.service.d/autologin.conf

# Disable resolved and ntpd
# See https://github.com/firecracker-microvm/firecracker/blob/f2743bfd2e9f9740dd08e55cdb83ba7b94aabb69/resources/tests/setup_rootfs.sh#L46
rm -f /etc/systemd/system/multi-user.target.wants/systemd-resolved.service
rm -f /etc/systemd/system/dbus-org.freedesktop.resolve1.service
rm -f /etc/systemd/system/sysinit.target.wants/systemd-timesyncd.service

# Enable our new services
systemctl enable grapl-plugin-bootstrap-init.service
systemctl enable grapl-plugin.service

# Finally: Clean up the apt cache a bit
apt-get clean
