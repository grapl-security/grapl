#!/usr/bin/env bash

set -eu -o pipefail -o xtrace

################################################################################
# This script is not meant to be run on a local workstation; instead, it's meant
# to be invoked by Packer on a remote EC2 machine.
################################################################################

SCRIPTS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly SCRIPTS_DIR

readonly OUTPUT_DIR="${HOME}/output"
mkdir -p "${OUTPUT_DIR}"

readonly BUILD_DIR="${HOME}/rootfs_build_dir"
mkdir -p "${BUILD_DIR}"

readonly IMAGE="${BUILD_DIR}/rootfs.ext4"
readonly MOUNT_POINT="${BUILD_DIR}/mount_point"
mkdir -p "${MOUNT_POINT}"

# Declare that we need these env vars later
readonly SIZE_MB
readonly PLUGIN_BOOTSTRAP_INIT_ARTIFACTS_DIR

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
# Bootstrap Debian at the rootfs root
sudo debootstrap --include apt,nano "${DEBIAN_VERSION}" \
    "${MOUNT_POINT}"

# Copy in the Plugin Bootstrap binary and associated systemd services
(
    # Destination must match what's in grapl-plugin-bootstrap-init.service
    sudo cp "${PLUGIN_BOOTSTRAP_INIT_ARTIFACTS_DIR}"/plugin-bootstrap-init \
        "${MOUNT_POINT}/usr/local/bin/"
    readonly PLUGIN_SERVICE_DIR="${MOUNT_POINT}/etc/systemd/system/"
    sudo mkdir -p "${PLUGIN_SERVICE_DIR}"
    sudo cp "${PLUGIN_BOOTSTRAP_INIT_ARTIFACTS_DIR}"/grapl-plugin.service \
        "${PLUGIN_SERVICE_DIR}/grapl-plugin.service"
    sudo cp "${PLUGIN_BOOTSTRAP_INIT_ARTIFACTS_DIR}"/grapl-plugin-bootstrap-init.service \
        "${PLUGIN_SERVICE_DIR}/grapl-plugin-bootstrap-init.service"
)

# Run the Provision script
(
    # Make these scripts available inside the chroot
    SCRIPTS_MOUNT_POINT="${MOUNT_POINT}/mnt/scripts"
    sudo mkdir -p "${SCRIPTS_MOUNT_POINT}"
    sudo mount --bind "${SCRIPTS_DIR}" "${SCRIPTS_MOUNT_POINT}"
    # Run the provision_inside_chroot script
    sudo chroot "${MOUNT_POINT}" "/mnt/scripts/provision_inside_chroot.sh"

    sudo umount "${SCRIPTS_MOUNT_POINT}"
)

########################################
# Tar the image into $OUTPUT_DIR; this will be scp'd
# back to the Packer Host OS by Packer
########################################
sudo umount "${MOUNT_POINT}"

# NOTE about tar: If you specify the full path of the image,
#   the tar will contain that full nested path of directories.
#   Hence the basename.
tar \
    --directory "${BUILD_DIR}" \
    --file="${OUTPUT_DIR}/${IMAGE_ARCHIVE_NAME}" \
    --create \
    --gzip \
    "$(basename "${IMAGE}")"
