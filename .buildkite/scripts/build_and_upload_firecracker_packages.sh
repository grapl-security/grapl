#!/usr/bin/env bash

set -euo pipefail

################################################################################
# This file is responsible for building and uploading non-Docker packages, like
# the Firecracker microVM kernel and the Firecracker rootfs.
#
# It will not upload or promote anything if the artifact-metadata.json file
# specifies the same tag found in ${UPSTREAM_REGISTRY}.
#
# Every new uploaded package must have a different Version; as such it is
# convenient to include a git SHA or timestamp_and_sha_version().
################################################################################

source .buildkite/scripts/lib/artifacts.sh

# This is where our images will ultimately be promoted to. It is the
# registry we'll need to query to see if file with the same version
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
# Returns 0 if it is present; 1 if not.
present_upstream() {
    cloudsmith_query_package --query="name:^${1}$ AND tag:^${2}$" |
        jq --exit-status ". | length != 0"
}

########################################
# Main logic
########################################
# These will be uploaded to Cloudsmith with just their basename.
readonly PACKAGES=(
    dist/firecracker_kernel.tar.gz
)

# This is the list of packages that actually have different shas
new_packages=()

echo "--- Building packages"
make dist/firecracker_kernel.tar.gz

echo "--- :cloudsmith::sleuth_or_spy: Checking upstream repository to determine what to promote"

for artifact_path in "${PACKAGES[@]}"; do
    artifact_name=$(basename "${artifact_path}")
    echo "--- :cloudsmith: Should we update '${artifact_name}' in '${UPSTREAM_REGISTRY}'?"
    version="$(get_version_from_artifact_metadata "${artifact_path}")"
    tag="$(get_tag_from_artifact_metadata "${artifact_path}")"

    echo "Checking if a package with tag ${tag} exists upstream..."
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
    version="$(get_version_for_artifact "${artifact_path}")"
    tag="$(get_tag_from_artifact_metadata "${artifact_path}")"
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
