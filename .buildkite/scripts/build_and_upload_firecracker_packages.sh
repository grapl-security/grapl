#!/usr/bin/env bash

set -euo pipefail

################################################################################
# This file is responsible for building and uploading non-Docker packages, like
# the Firecracker microVM kernel and the Firecracker rootfs.
# It will not upload anything if a file of that name and SHA already exist
# upstream.
################################################################################

source .buildkite/scripts/lib/artifacts.sh
source .buildkite/scripts/lib/version.sh
source firecracker/kernel/constants.sh

DEFAULT_VERSION="$(timestamp_and_sha_version)"

get_version() {
    # Special case for Firecracker kernel - we just label it with the
    # Firecracker release/kernel vm built

    if [[ "${1}" == "firecracker_kernel.tar.gz" ]]; then
        echo "firecracker-${FIRECRACKER_RELEASE}-kernel-${KERNEL_VERSION}"
        return 0
    fi
    echo "${DEFAULT_VERSION}"
}

(
    echo "--- Building packages"
    make dist/firecracker_kernel.tar.gz
)

# These will be uploaded to Cloudsmith with just their basename.
readonly PACKAGES=(
    dist/firecracker_kernel.tar.gz
)

# This is where our images will ultimately be promoted to. It is the
# registry we'll need to query to see if file with the same contents
# already exists.
readonly UPSTREAM_REGISTRY="${UPSTREAM_REGISTRY:-grapl/testing}"
readonly UPLOAD_TO_REGISTRY="${UPLOAD_TO_REGISTRY:-grapl/raw}"

# This is the list of packages that actually have different shas
new_packages=()

# Returns a string like `2de4f0c...`
sha256_of_file() {
    sha256sum "${1}" | awk '{print $1;}'
}

# Get the json query result of a named package.
# Usage:
#  query_package --query="name:^firecracker_kernel.tar.gz$""
cloudsmith_query_package() {
    queries="${1}"
    cloudsmith ls packages \
        "${UPSTREAM_REGISTRY}" \
        "${queries}" \
        --output-format=pretty_json |
        jq ".data"
}

# Returns 0 if it is present; 1 if not.
present_upstream() {
    cloudsmith_query_package --query="name:^${1}$ AND checksum:^${2}$" |
        jq --exit-status ". | length != 0"
}

echo "--- :cloudsmith::sleuth_or_spy: Checking upstream repository to determine what to promote"

for artifact_path in "${PACKAGES[@]}"; do
    artifact_name=$(basename "${artifact_path}")
    echo "--- :cloudsmith: Checking '${artifact_name}' in '${UPSTREAM_REGISTRY}'"
    incoming_sha256="$(sha256_of_file "${artifact_path}")"

    echo "Checking if a version with SHA ${incoming_sha256} exists upstream..."
    if ! present_upstream "${artifact_name}" "${incoming_sha256}"; then
        echo "Package not present upstream; will promote '${artifact_name}'"
        new_packages+=("${artifact_path}")
    else
        echo "Package with this SHA exists upstream; no change needed"
    fi
done

echo "--- :cloudsmith::up: Uploading new packages to Cloudsmith"
for artifact_path in "${new_packages[@]}"; do
    artifact_name=$(basename "${artifact_path}")
    version="$(get_version "${artifact_name}")"
    cloudsmith upload raw "${UPLOAD_TO_REGISTRY}" \
        "${artifact_path}" \
        --name "${artifact_name}" \
        --version "${version}"

    # This generates an artifact_json file for each artifact, since we have differing versions
    # between each.
    artifact_file="$(artifacts_file_for "${artifact_name}")"
    artifact_json "${version}" "${artifact_name}" > "${artifact_file}"
    echo "Wrote results to ${artifact_file}"
done
