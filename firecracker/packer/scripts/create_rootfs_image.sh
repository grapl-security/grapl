#!/usr/bin/env bash

set -euo pipefail
set -o xtrace

################################################################################
# This script is not meant to be run on a local workstation; instead, it's meant
# to be invoked by Packer on a remote EC2 machine.
################################################################################

readonly OUTPUT_DIR="${HOME}/output"
mkdir -p "${OUTPUT_DIR}"

readonly BUILD_DIR="${HOME}/rootfs_build_dir"
mkdir -p "${BUILD_DIR}"

readonly IMAGE="${BUILD_DIR}/${IMAGE_NAME}.ext4"
readonly MOUNT_POINT="${BUILD_DIR}/mount_point"
mkdir -p "${MOUNT_POINT}"

readonly SIZE_MB="${SIZE_MB}"

########################################
# Create image and mount it.
########################################
# make a $SIZE_MB empty file
dd if=/dev/zero of="${IMAGE}" bs=1M count="${SIZE_MB}"

# format that filesystem
# `-F` is 'force'; needed to bypass prompt:
# "<file> is not a block special device. Proceed anyway? (y,n))"
/sbin/mkfs.ext4 -F "${IMAGE}"

# Mount the image at mountpoint
sudo mount -t ext4 \
    -o loop,rw \
    "${IMAGE}" "${MOUNT_POINT}"

# Let anybody write to the mount point.
sudo chmod 777 "${MOUNT_POINT}"

########################################
# Mutate image
########################################
(
    # Bootstrap Ubuntu at the rootfs root
    sudo debootstrap --include apt,nano "${DEBIAN_VERSION}" \
        "${MOUNT_POINT}"

    # Do some Firecracker-specific mutations, primarily the agetty thing.
    # See https://github.com/firecracker-microvm/firecracker/blob/main/docs/rootfs-and-kernel-setup.md
    sudo chroot "${MOUNT_POINT}" /bin/bash -x << ENDCHROOT
        set -euo pipefail

        echo "grapl-plugin" > /etc/hostname
        passwd -d root

        # Set up a login terminal on the serial console (ttyS0)
        mkdir /etc/systemd/system/serial-getty@ttyS0.service.d/
        cat <<EOF > /etc/systemd/system/serial-getty@ttyS0.service.d/autologin.conf
        [Service]
        ExecStart=
        ExecStart=-/sbin/agetty --autologin root -o '-p -- \\u' --keep-baud 115200,38400,9600 %I $TERM
EOF
ENDCHROOT
)

########################################
# Tar the image into $OUTPUT_DIR; this will be scp'd
# back to the Packer Host OS by Packer
########################################
sudo umount "${MOUNT_POINT}"

# NOTE about tar: If you specify the full path of the image,
# the tar will contain that full nested path of directories.
# Hence the basename.
tar \
    --directory "${BUILD_DIR}" \
    --file="${OUTPUT_DIR}/${IMAGE_ARCHIVE_NAME}" \
    --create \
    --gzip \
    "$(basename "${IMAGE}")"
