#!/usr/bin/env bash
set -euo pipefail

########################################
# Generate the rootfs in Packer.
########################################
readonly IMAGE_NAME="firecracker_rootfs"

packer init -upgrade firecracker/rootfs/build-rootfs.pkr.hcl
packer build \
    -var dist_dir="${DIST_DIR}" \
    -var plugin_bootstrap_init_artifacts_dir="${DIST_DIR}/plugin-bootstrap-init" \
    -var image_name="${IMAGE_NAME}" \
    firecracker/rootfs/build-rootfs.pkr.hcl

########################################
# Write a .artifact-metadata.json file
########################################
source .buildkite/scripts/lib/artifact_metadata.sh
readonly ARTIFACT_PATH="${DIST_DIR}/${IMAGE_NAME}.gz"
ARTIFACT_METADATA_PATH="$(artifact_metadata_path "${ARTIFACT_PATH}")"
readonly ARTIFACT_METADATA_PATH

source .buildkite/scripts/lib/version.sh
VERSION="$(timestamp_and_sha_version)"
readonly VERSION

INPUT_SHA256="$(sha256_of_dir firecracker/rootfs)"
readonly INPUT_SHA256

artifact_metadata_contents "${VERSION}" "${INPUT_SHA256}" > "${ARTIFACT_METADATA_PATH}"
