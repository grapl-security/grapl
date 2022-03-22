#!/usr/bin/env bash
set -euo pipefail

################################################################################
# A Manifest represents metadata about a single, built artifact.
# It is consumed by `build_and_upload_firecracker_packages.sh` to inform
# Cloudsmith which version and tags to use.
# It is *mostly unrelated* to grapl-artifacts.sh and artifacts.sh, which instead
# are about propagating artifact tags into `origin/rc`
################################################################################

# Generates a SHA256 checksum of the sorted output of the SHA256
# checksums of all the files in this directory (recursively).
#
# $ find . -type f | sort | xargs sha256sum
# 123veryLongSha  ./BUILD
# 456veryLongSha  ./build_and_upload_firecracker_packages.sh
# 789veryLongSha  ./build_and_upload_images.sh
# veryLongSha012  ./ensure_regenerated_constraints.sh
# veryLongSha345  ./extract_artifacts.sh
#
#
# Taking the SHA256 checksum of this output yields the final
# checksum:
#
# $ find . -type f | sort | xargs sha256sum | sha256sum
# 5b611bf839fb19c58000TheSumOfAllTheShas00072cb772a5342b569ec  -
sha256_of_dir() {
    local -r dir_path="${1}"
    find "${dir_path}" -type f | sort | xargs sha256sum | sha256sum | awk '{print $1;}'
}

artifact_metadata_contents() {
    local -r version="${1}"
    local -r input_sha256="${2}"
    jq --null-input \
        --arg version "${version}" \
        --arg input_sha256 "${input_sha256}" \
        '{"version": $version, "input_sha256": $input_sha256}'
}

artifact_metadata_path() {
    local -r artifact_path="${1}"
    echo "${artifact_path}.artifact-metadata.json"
}

get_version_from_artifact_metadata() {
    local -r artifact_path="${1}"
    local -r artifact_metadata_path="$(path_for_artifact_metadata "${artifact_path}")"
    jq -r ".version" "${artifact_metadata_path}"
}

get_input_sha_from_artifact_metadata() {
    local -r artifact_path="${1}"
    local -r artifact_metadata_path="$(path_for_artifact_metadata "${artifact_path}")"
    jq -r ".input_sha256" "${artifact_metadata_path}"
}
