#!/usr/bin/env bash
set -euo pipefail

# Feed in a list of files.
# Results in a single sha256sum of all those files.
sha256_of_input_files() {
    local -ra inputs=("${@}")
    echo "${inputs[@]}" | sha256sum | awk '{print $1;}'
}

sha256_of_dir() {
    local -r dir_path="${1}"
    sha256_of_input_files "$(find "${dir_path}" -type f)"
}

sha_manifest_contents() {
    local -r sha256="${1}"
    jq --null-input \
        --arg sha256 "${sha256}" \
        '{"input_files_sha256": $sha256}'
}
