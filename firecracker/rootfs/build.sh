#!/usr/bin/env bash
set -euo pipefail

########################################
# Generate the rootfs in Packer.
########################################
readonly IMAGE_NAME="firecracker_rootfs"

packer init -upgrade firecracker/rootfs/build-rootfs.pkr.hcl
packer build \
    -var dist_folder="${GRAPL_ROOT}/dist" \
    -var image_name="${IMAGE_NAME}" \
    firecracker/rootfs/build-rootfs.pkr.hcl

########################################
# Write a manifest file containing the version.
# The version should dedupe based on input file checksums.
# (This may be replaced with Pants-based file diffs in the future.)
########################################
source .buildkite/scripts/lib/manifest.sh
source .buildkite/scripts/lib/version.sh
INPUTS_SHA="$(sha256_of_dir firecracker/rootfs)"
readonly INPUTS_SHA_SHORT="${INPUTS_SHA:0:16}" # cloudsmith version field must be under 128 chars
VERSION="$(timestamp_and_sha_version)-input-sha256-${INPUTS_SHA_SHORT}"
readonly VERSION
readonly MANIFEST_PATH="${GRAPL_ROOT}/dist/${IMAGE_NAME}.tar.gz.manifest"
manifest_contents "${INPUTS_SHA}" "${VERSION}" > "${MANIFEST_PATH}"
