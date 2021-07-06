#!/usr/local/env bash

# Uploads a given file, as a "raw" file (e.g., a ZIP file, not any
# other file that has a custom Cloudsmith repository type, like a
# Python wheel, Rust crate, etc.), to a specific repository in our
# 'grapl' organization in Cloudsmith.
upload_raw_file_to_cloudsmith() {
    local -r file_path="${1}"
    local -r version="${2}"
    local -r repository="${3}"

    cloudsmith upload \
        raw \
        "grapl/${repository}" \
        "${file_path}" \
        --version="${version}" \
        --verbose
}
