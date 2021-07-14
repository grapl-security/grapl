#!/usr/bin/env bash

set -euo pipefail

source .buildkite/scripts/lib/packer_constants.sh
readonly jq_filter_path=".buildkite/scripts/lib/extract_ami_id_dict.jq"

upload_artifacts_file() {
    # Takes in the name of an image, like "image_name", and expects to find
    # a corresponding packer manifest names "image_name.packer-manifest.json".

    # e.g. "grapl-service"
    local -r packer_image_name=$1
    # e.g. "grapl-service.packer-manifest.json"
    local -r manifest_file="${packer_image_name}${PACKER_MANIFEST_SUFFIX}"
    # e.g. "grapl-service.artifacts.json"
    local -r artifacts_file="${packer_image_name}${ARTIFACTS_FILE_SUFFIX}"

    # Download Packer manifest
    echo -e "--- :buildkite: Download Packer manifest"
    buildkite-agent artifact download "${manifest_file}" .

    # TODO wimax July 2021: The below is not correct for Grapl's AMIs.
    # We have many regions.

    # Extract AMI ID
    echo -e "--- :gear: Extracting AMI ID"
    # The raw artifact ID is like: "us-east-1:ami-0123456789abcdef0";

    # Creates a dict that looks like
    # { "imagename.us-east-1": "ami-111", "imagename.us-east-2": "ami-222", ...}
    local -r ami_ids_dict=$(jq --raw-output --arg IMAGE_NAME "${packer_image_name}" --from-file "${jq_filter_path}" "${manifest_file}")
    echo "${ami_ids_dict}"

    # Creating artifacts file
    echo -c "--- :gear: Creating ${artifacts_file} file"
    echo "${ami_ids_dict}" > "${artifacts_file}"

    # Uploading artifacts file
    echo -c "--- :buildkite: Uploading ${artifacts_file} file"
    buildkite-agent artifact upload "${artifacts_file}"
    # This artifact then gets picked up by the "Merge artifacts files" step in Buildkite
}

for image_name in "${PACKER_IMAGE_NAMES[@]}"; do
    upload_artifacts_file "${image_name}"
done
