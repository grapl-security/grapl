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
dd if=/dev/zero of="${IMAGE}" bs=1M count=100

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

    # This is basically what Firecracker's `devtool build_rootfs` does:
    # Copy requisite dirs from a very minimal Debian build.
    readonly dirs_to_copy=(/bin /etc /home /lib /lib64 /opt /root /sbin /usr)
    cp -r ${dirs_to_copy[@]} "${MOUNT_POINT}/"

    echo "hello :)" > hello.txt
)

########################################
# Do some Firecracker-specific mutations that help it interact with tty
########################################
"${THIS_DIR}/prepare_rootfs.sh" "${MOUNT_POINT}"

########################################
# Copy rootfs into dist.
########################################
(
    cd "/tmp"
    tar --create --gzip --file "${DIST_FILE}" --directory "/tmp" "rootfs.ext4"
    # -u is unmount; there is no -- version
)
fusermount -u "${MOUNT_POINT}"