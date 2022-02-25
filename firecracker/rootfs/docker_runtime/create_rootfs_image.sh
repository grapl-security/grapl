#!/usr/bin/env bash
set -euo pipefail
set -o xtrace

readonly IMAGE="/tmp/rootfs.ext4"
readonly MOUNT_POINT="/tmp/rootfs-mount"
readonly DIST_FILE="/dist/firecracker_rootfs.tar.gz"

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
mount -t fuse-ext2 -o rw+ "${IMAGE}" "${MOUNT_POINT}"

########################################
# Mutate image
########################################
(
    cd "${MOUNT_POINT}"
    echo "hello :)" > hello.txt
)

########################################
# Copy rootfs into dist.
########################################
tar --create --gzip --file="${DIST_FILE}" "${IMAGE}"
# -u is unmount; there is no -- version
fusermount -u "${MOUNT_POINT}"