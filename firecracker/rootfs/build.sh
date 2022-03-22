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
source .buildkite/scripts/lib/version.sh
INPUTS_SHA="$(sha256_of_dir firecracker/rootfs)"
readonly INPUTS_SHA_SHORT="${INPUTS_SHA:0:16}" # cloudsmith version field must be under 128 chars
VERSION="$(timestamp_and_sha_version)-input-sha256-${INPUTS_SHA_SHORT}"
readonly VERSION
readonly ARTIFACT_PATH="${GRAPL_ROOT}/dist/${IMAGE_NAME}.tar.gz"
ARTIFACT_METADATA_PATH="$(artifact_metadata_path "${ARTIFACT_PATH}")"
readonly ARTIFACT_METADATA_PATH
artifact_metadata_contents "${INPUTS_SHA}" "${VERSION}" > "${ARTIFACT_METADATA_PATH}"
