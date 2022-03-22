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
# Write a .artifact-metadata.json file
########################################
source .buildkite/scripts/lib/artifact_metadata.sh
readonly ARTIFACT_PATH="${GRAPL_ROOT}/dist/${IMAGE_NAME}.tar.gz"
ARTIFACT_METADATA_PATH="$(artifact_metadata_path "${ARTIFACT_PATH}")"
readonly ARTIFACT_METADATA_PATH

source .buildkite/scripts/lib/version.sh
TAG="inputsha-$(sha256_of_dir firecracker/rootfs)"
readonly TAG
VERSION="$(timestamp_and_sha_version)"
readonly VERSION
artifact_metadata_contents "${VERSION}" "${TAG}" > "${ARTIFACT_METADATA_PATH}"
