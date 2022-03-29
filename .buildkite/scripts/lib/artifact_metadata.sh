#!/usr/bin/env bash
set -euo pipefail

################################################################################
# A Manifest represents metadata about a single, built artifact.
# It is consumed by `upload_firecracker_packages.sh` to inform
# Cloudsmith of:
# * the artifact's version
# * the SHA256sum of all files used to generate the artifact
# The input_sha256 is used to prevent duplicate uploads - it represents the
# identity of an artifact.
#
# NOTE: It is *mostly unrelated* to grapl-artifacts.sh and artifacts.sh, which
# instead are about propagating artifact tags into `origin/rc`
################################################################################

# Generates a SHA256 checksum of the sorted output of the SHA256
# checksums of all the files in this directory (recursively).
#
# $ find . -type f | sort | xargs sha256sum
# 123veryLongSha  ./some_file
# 456veryLongSha  ./coolfile.sh
# 789veryLongSha  ./coolfile2.sh
# veryLongSha012  ./coolfile3.sh
# veryLongSha345  ./coolfile4.py
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
    local -r artifact_metadata_path="$(artifact_metadata_path "${artifact_path}")"
    jq -r ".version" "${artifact_metadata_path}"
}

get_input_sha256_from_artifact_metadata() {
    local -r artifact_path="${1}"
    local -r artifact_metadata_path="$(artifact_metadata_path "${artifact_path}")"
    jq -r ".input_sha256" "${artifact_metadata_path}"
}
