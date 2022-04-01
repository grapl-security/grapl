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

# Move kernel to a stable name, so that Nomad knows which file in the
# untarred archive to use as the kernel.
readonly KERNEL_FILENAME="firecracker_kernel"
readonly ARTIFACT_PATH="${REPOSITORY_ROOT}/dist/${KERNEL_FILENAME}.tar.gz"
mv "${KERNEL_BIN_DIR}/${KERNEL_BIN_FILE}" "./${KERNEL_FILENAME}"

tar \
    --file="${ARTIFACT_PATH}" \
    --create \
    --gzip \
    "${KERNEL_FILENAME}"

########################################
# Write a .artifact-metadata.json file
########################################
source .buildkite/scripts/lib/artifact_metadata.sh
ARTIFACT_METADATA_PATH="$(artifact_metadata_path "${ARTIFACT_PATH}")"
readonly ARTIFACT_METADATA_PATH

source .buildkite/scripts/lib/version.sh
# Note: in Cloudsmith, versions must be len <=128, hence we use a short git sha.
VERSION="firecracker-${FIRECRACKER_RELEASE}-kernel-${KERNEL_VERSION}-$(sha_version)"
readonly VERSION

INPUT_SHA256="$(sha256_of_dir firecracker/kernel)"
readonly INPUT_SHA256

artifact_metadata_contents "${VERSION}" "${INPUT_SHA256}" > "${ARTIFACT_METADATA_PATH}"
