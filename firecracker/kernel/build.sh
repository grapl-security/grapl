#!/usr/bin/env bash
set -euo pipefail

################################################################################
# Usage: Call this script from the Makefile, which prevents unnecessarily
#   recompiling the kernel (takes 4-5m on our Chromebooks).
################################################################################
# shellcheck source-path=SCRIPTDIR
source "$(dirname "${BASH_SOURCE[0]}")/constants.sh"

REPOSITORY_ROOT=$(git rev-parse --show-toplevel)
readonly REPOSITORY_ROOT

BUILD_DIR="$(mktemp --directory)"
readonly BUILD_DIR

########################################
# Get the Firecracker source
########################################
(
    cd "${BUILD_DIR}"
    git clone git@github.com:firecracker-microvm/firecracker.git \
        --depth=1 \
        --branch="${FIRECRACKER_RELEASE}"
    cd firecracker

    ########################################
    # Generate the kernel.
    # Based on https://github.com/firecracker-microvm/firecracker/blob/main/docs/rootfs-and-kernel-setup.md
    ########################################
    ./tools/devtool build_kernel --config "${KERNEL_CONFIG}" --nproc 8
)

########################################
# Copy kernel into dist.
########################################
readonly KERNEL_BIN_DIR="${BUILD_DIR}/firecracker/build/kernel/linux-${KERNEL_VERSION}/"
readonly KERNEL_BIN_FILE="vmlinux-${KERNEL_VERSION}-x86_64.bin"
readonly DISTRIBUTION="${REPOSITORY_ROOT}/dist/firecracker_kernel.tar.gz"

# NOTE about tar: If you specify the full path of the thing-to-be-tar'd,
#   the tar will contain that full nested path of directories.
#   Hence the --directory, and basename for KERNEL_BIN_FILE
tar \
    --directory "${KERNEL_BIN_DIR}" \
    --file="${DISTRIBUTION}" \
    --create \
    --gzip \
    "${KERNEL_BIN_FILE}"

########################################
# Write a manifest file containing the version.
# The version should dedupe based on input file checksums.
# (This may be replaced with Pants-based file diffs in the future.)
########################################
source build-support/create_manifest.sh
INPUTS_SHA="$(sha256_of_dir firecracker/kernel)"
readonly INPUTS_SHA_SHORT="${INPUTS_SHA:0:16}" # cloudsmith version field must be under 128 chars
readonly MANIFEST_PATH="${DISTRIBUTION}.manifest"
readonly VERSION="firecracker-${FIRECRACKER_RELEASE}-kernel-${KERNEL_VERSION}-input-sha256-${INPUTS_SHA_SHORT}"
manifest_contents "${VERSION}" > "${MANIFEST_PATH}"
