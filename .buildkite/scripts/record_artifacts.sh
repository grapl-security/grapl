#!/usr/bin/env bash

set -euo pipefail

source .buildkite/scripts/lib/packer_constants.sh

upload_artifacts_file() {
    # Takes in the name of an image, like "image_name", and expects to find
    # a corresponding packer manifest names "image_name.packer-manifest.json".

    # e.g. "grapl-service"
    readonly root_name=$1
    # e.g. "grapl-service.packer-manifest.json"
    readonly manifest_file="${root_name}${PACKER_MANIFEST_SUFFIX}"
    # e.g. "grapl-service.artifacts.json"
    readonly artifacts_file="${root_name}${ARTIFACTS_FILE_SUFFIX}"

    # Download Packer manifest
    echo -e "--- :buildkite: Download Packer manifest"
    buildkite-agent artifact download "${manifest_file}" .

    # TODO wimax July 2021: The below is not correct for Grapl's AMIs.
    # We have many regions.

    # Extract AMI ID
    echo -e "--- :gear: Extracting AMI ID"
    # The raw artifact ID is like: "us-east-1:ami-0123456789abcdef0";
    # we're just after the ID, not the region (at the moment, everything
    # we do is in the same region).
    ami_id=$(jq -r '.builds[-1].artifact_id' "${manifest_file}" | cut -d ":" -f2)
    echo "${ami_id}"

    # Creating artifacts file
    echo -c "--- :gear: Creating ${artifacts_file} file"
    # TODO: This should also have regions in the near future
    echo "{\"${root_name}-ami\":\"${ami_id}\"}" > "${artifacts_file}"
    jq '.' "${artifacts_file}"

    # Uploading artifacts file
    echo -c "--- :buildkite: Uploading ${artifacts_file} file"
    buildkite-agent artifact upload "${artifacts_file}"
}

for image_name in "${PACKER_IMAGE_NAMES[@]}"; do
    upload_artifacts_file "${image_name}"
done
