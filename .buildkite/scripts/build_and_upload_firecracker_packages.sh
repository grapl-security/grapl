#!/usr/bin/env bash

set -euo pipefail

################################################################################
# This file is responsible for building and uploading non-Docker packages, like
# the Firecracker microVM kernel and the Firecracker rootfs.
#
# It will not upload or promote anything if the artifact-metadata.json file
# specifies an (artifact_name + input_sha256) pair that already exists in
# the ${UPSTREAM_REGISTRY}.
# (Notably: We store the `input_sha256` in Cloudsmith's "tag" field.)
#
# Every new uploaded package must have a different Version; as such it is
# convenient to include a git SHA or timestamp_and_sha_version().
################################################################################

source .buildkite/scripts/lib/artifacts.sh
source .buildkite/scripts/lib/artifact_metadata.sh

# This is where our images will ultimately be promoted to. It is the
# registry we'll need to query to see if a package with the same tag
# already exists.
readonly UPSTREAM_REGISTRY="${UPSTREAM_REGISTRY:-grapl/testing}"
readonly UPLOAD_TO_REGISTRY="${UPLOAD_TO_REGISTRY:-grapl/raw}"

# Get the json query result of a named package.
# Usage:
#  query_package --query="name:^firecracker_kernel.tar.gz$""
cloudsmith_query_package() {
    local -r queries="${1}"
    cloudsmith ls packages \
        "${UPSTREAM_REGISTRY}" \
        "${queries}" \
        --output-format=pretty_json |
        jq ".data"
}

# Check if a package with this name and tag exists in ${UPSTREAM_REGISTRY}.
# We use the 'tag' field to store the input_sha256.
# Returns 0 if it is present; 1 if not.
present_upstream() {
    cloudsmith_query_package --query="name:^${1}$ AND tag:^${2}$" |
        jq --exit-status ". | length != 0"
}

input_sha_as_tag() {
    local -r input_sha256="${1}"
    echo "input-sha256-${input_sha256}"
}

########################################
# Main logic
########################################
# These will be uploaded to Cloudsmith with just their basename.
readonly PACKAGES=(
    dist/firecracker_kernel.tar.gz
    dist/firecracker_rootfs.tar.gz
)

# This is the list of packages that actually have different shas
new_packages=()

echo "--- Building packages"
make "${PACKAGES[@]}"

echo "--- :cloudsmith::sleuth_or_spy: Checking upstream repository to determine what to promote"

for artifact_path in "${PACKAGES[@]}"; do
    artifact_name=$(basename "${artifact_path}")
    echo "--- :cloudsmith: Should we update '${artifact_name}' in '${UPSTREAM_REGISTRY}'?"
    version="$(get_version_from_artifact_metadata "${artifact_path}")"
    input_sha256="$(get_input_sha256_from_artifact_metadata "${artifact_path}")"
    tag="$(input_sha_as_tag "${input_sha256}")"

    echo "Checking if a package generated with the same inputs exists upstream. SHA: ${input_sha256}"
    if ! present_upstream "${artifact_name}" "${tag}"; then
        echo "Package not present upstream; will promote '${artifact_name}'"
        new_packages+=("${artifact_path}")
    else
        echo "Package with this tag exists upstream; no change needed"
    fi
done

echo "--- :cloudsmith::up: Uploading new packages to Cloudsmith"
for artifact_path in "${new_packages[@]}"; do
    artifact_name=$(basename "${artifact_path}")
    version="$(get_version_from_artifact_metadata "${artifact_path}")"
    input_sha256="$(get_input_sha256_from_artifact_metadata "${artifact_path}")"
    tag="$(input_sha_as_tag "${input_sha256}")"

    cloudsmith upload raw "${UPLOAD_TO_REGISTRY}" \
        "${artifact_path}" \
        --name "${artifact_name}" \
        --version "${version}" \
        --tags "${tag}"

    # This generates an artifact_json file for each artifact, since we have differing versions
    # between each.
    artifact_file="$(artifacts_file_for "${artifact_name}")"
    artifact_json "${version}" "${artifact_name}" > "${artifact_file}"
    echo "Wrote results to ${artifact_file}"
done
