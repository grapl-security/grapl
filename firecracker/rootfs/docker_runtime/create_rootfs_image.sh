#!/usr/bin/env bash
set -euo pipefail
set -o xtrace

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly THIS_DIR

readonly IMAGE="/tmp/rootfs.ext4"
readonly MOUNT_POINT="/tmp/rootfs-mount"
readonly DIST_FILE="/dist/firecracker_rootfs.ext4.tar.gz"

########################################
# Create image and mount it.
########################################
# make a 50mb empty file
dd if=/dev/zero of="${IMAGE}" bs=1M count=50

# format that filesystem
/sbin/mkfs.ext4 "${IMAGE}"

# make a mount-point
mkdir "${MOUNT_POINT}"

# Mount
mount -t fuse-ext2 \
    -o rw+ \
    "${IMAGE}" "${MOUNT_POINT}"

########################################
# Mutate image
########################################
(
    # Bootstrap Debian at the rootfs root
    debootstrap --include openssh-server,unzip,rsync,apt,netplan.io,nano focal \
        "${MOUNT_POINT}" \
        http://archive.ubuntu.com/ubuntu/

    # Copy provision script into the image
    # (this could be done with a mount - too hard to follow)
    mkdir -p "${MOUNT_POINT}"
    cp "${THIS_DIR}/provision.sh" "${MOUNT_POINT}/setup"
    
    # Run the provision script as if it were chroot'd into the root
    chroot "${MOUNT_POINT}" /bin/bash "/setup/provision.sh"
    
    cd "${MOUNT_POINT}"
    echo "hello :)" > hello.txt
)

########################################
# Copy rootfs into dist.
########################################
(
    cd /tmp
    tar --create --gzip --file "${DIST_FILE}" --directory "/tmp" "rootfs.ext4"
)
# -u is unmount; there is no -- version
fusermount -u "${MOUNT_POINT}"
