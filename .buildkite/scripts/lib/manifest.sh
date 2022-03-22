#!/usr/bin/env bash
set -euo pipefail

################################################################################
# A Manifest represents metadata about a built artifact.
# It is consumed by `build_and_upload_firecracker_packages.sh` to inform
# Cloudsmith which version and tags to use.
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

manifest_contents() {
    local -r version="${1}"
    jq --null-input \
        --arg version "${version}" \
        '{"version": $version}'
}

get_version_from_manifest() {
    local -r manifest_path="${1}"
    jq -r ".version" "${manifest_path}"
}