#!/usr/bin/env bash
set -euo pipefail

# Results in a single sha256sum of all files in directory
sha256_of_dir() {
    local -r dir_path="${1}"
    find "${dir_path}" -type f -exec sha256sum {} \; | sha256sum | awk '{print $1;}'
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