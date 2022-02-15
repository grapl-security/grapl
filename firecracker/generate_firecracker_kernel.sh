#!/usr/bin/env bash
set -euo pipefail

################################################################################
# Usage: Call this script from the Makefile, which prevents unnecessarily
#   recompiling the kernel (takes 4-5m on our Chromebooks).
################################################################################
readonly FIRECRACKER_RELEASE="v1.0.0"
readonly KERNEL_CONFIG="resources/guest_configs/microvm-kernel-x86_64-4.14.config"

REPOSITORY_ROOT=$(git rev-parse --show-toplevel)
readonly REPOSITORY_ROOT

readonly BUILD_DIR="/tmp/firecracker_kernel_build"
mkdir -p "${BUILD_DIR}"
cd "${BUILD_DIR}"

########################################
# Get the Firecracker source
########################################
if [ ! -d "./firecracker" ]; then
    git clone git@github.com:firecracker-microvm/firecracker.git
fi
cd firecracker
git fetch
git checkout tags/"${FIRECRACKER_RELEASE}"

########################################
# Generate the kernel.
# Based on https://github.com/firecracker-microvm/firecracker/blob/main/docs/rootfs-and-kernel-setup.md
########################################
rm -rf build/kernel
./tools/devtool build_kernel -c "${KERNEL_CONFIG}" -n 8

########################################
# Copy kernel into dist.
########################################
KERNEL_BIN_FILE="$(ls build/kernel/**/*.bin)"
gzip --stdout "${KERNEL_BIN_FILE}" > "${REPOSITORY_ROOT}/dist/firecracker_kernel.tar.gz"