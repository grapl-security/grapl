#!/usr/bin/env bash

set -euo pipefail

source .buildkite/scripts/lib/packer_constants.sh

# Download Packer manifest
echo -e "--- :buildkite: Download Packer manifest"
buildkite-agent artifact download "${PACKER_MANIFEST_FILE}" .

# Extract AMI ID
echo -e "--- :gear: Extracting AMI ID"
# The raw artifact ID is like: "us-east-1:ami-0123456789abcdef0";
# we're just after the ID, not the region (at the moment, everything
# we do is in the same region).
ami_id=$(jq -r '.builds[-1].artifact_id' "${PACKER_MANIFEST_FILE}" | cut -d ":" -f2)
echo "${ami_id}"

# Creating artifacts file
echo -c "--- :gear: Creating ${ARTIFACTS_FILE} file"
echo "{\"ami\":\"${ami_id}\"}" > "${ARTIFACTS_FILE}"
jq '.' "${ARTIFACTS_FILE}"

# Uploading artifacts file
echo -c "--- :buildkite: Uploading ${ARTIFACTS_FILE} file"
buildkite-agent artifact upload "${ARTIFACTS_FILE}"
