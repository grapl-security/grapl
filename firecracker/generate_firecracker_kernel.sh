#!/usr/bin/env bash
set -euo pipefail

################################################################################
# Usage: Call this script from the Makefile, which prevents unnecessarily
#   recompiling the kernel (takes 4-5m on our Chromebooks).
################################################################################
readonly FIRECRACKER_RELEASE="v1.0.0"
readonly KERNEL_VERSION="5.10.0"  # make sure in-sync with below
readonly KERNEL="x86_64-5.10"     # make sure in-sync with above
readonly KERNEL_CONFIG="resources/guest_configs/microvm-kernel-${KERNEL}.config"

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

########################################
# Copy kernel into dist.
########################################
results=$(cat <<EOF
{
    "firecracker_release": "${FIRECRACKER_RELEASE}",
    "kernel_version": "${KERNEL_VERSION}",
    "distribution": "${DISTRIBUTION}"
}
EOF
)
echo "${results}" | jq .