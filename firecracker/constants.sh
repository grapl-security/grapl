#!/usr/bin/env bash
set -euo pipefail

export readonly FIRECRACKER_RELEASE="v1.0.0"
export readonly KERNEL_VERSION="4.14.174" # make sure in-sync with below
export readonly KERNEL="x86_64-4.14"      # make sure in-sync with above
export readonly KERNEL_CONFIG="resources/guest_configs/microvm-kernel-${KERNEL}.config"
