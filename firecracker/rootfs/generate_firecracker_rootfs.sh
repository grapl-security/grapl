#!/usr/bin/env bash
set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
readonly THIS_DIR

REPOSITORY_ROOT=$(git rev-parse --show-toplevel)
readonly REPOSITORY_ROOT

(
    cd "${THIS_DIR}"
    docker buildx bake -f bake.hcl rootfs-build
)

########################################
# Generate the rootfs.
# We use Fuse to mount inside Docker.
########################################
docker run \
    --rm \
    --device /dev/fuse \
    --cap-add SYS_ADMIN \
    --volume "${REPOSITORY_ROOT}/dist:/dist" \
    rootfs-build:dev