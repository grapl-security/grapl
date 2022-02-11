#!/usr/bin/env bash
set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
cd "${THIS_DIR}"

################################################################################
# Get the Firecracker source
################################################################################
FIRECRACKER_RELEASE="v1.0.0"
git clone git@github.com:firecracker-microvm/firecracker.git
cd firecracker
git checkout "${FIRECRACKER_RELEASE}"

################################################################################
# Generate the kernel.
# Based on https://github.com/firecracker-microvm/firecracker/blob/main/docs/rootfs-and-kernel-setup.md
################################################################################
KERNEL_CONFIG="resources/guest_configs/microvm-kernel-x86_64-4.14.config"
./tools/devtool build_kernel -c "${KERNEL_CONFIG}" -n 8

################################################################################
# Copy kernel into dist.
################################################################################
# rm -rf "${REPOSITORY_ROOT}/src/rust/${CRATE_NAME}"
# cp -r "${OUTPUT_DIR}" "${REPOSITORY_ROOT}/src/rust/${CRATE_NAME}"
