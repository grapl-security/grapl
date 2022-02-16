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
cd "${BUILD_DIR}"

########################################
# Get the Firecracker source
########################################
git clone git@github.com:firecracker-microvm/firecracker.git \
    --depth=1 \
    --branch="${FIRECRACKER_RELEASE}"
cd firecracker

########################################
# Generate the kernel.
# Based on https://github.com/firecracker-microvm/firecracker/blob/main/docs/rootfs-and-kernel-setup.md
########################################
./tools/devtool build_kernel --config "${KERNEL_CONFIG}" --nproc 8

########################################
# Copy kernel into dist.
########################################
readonly KERNEL_BIN_FILE="build/kernel/linux-${KERNEL_VERSION}/vmlinux-${KERNEL_VERSION}-x86_64.bin"
readonly DISTRIBUTION="${REPOSITORY_ROOT}/dist/firecracker_kernel.tar.gz"
tar --create --gzip --file="${DISTRIBUTION}" "${KERNEL_BIN_FILE}"
