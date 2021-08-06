#!/usr/bin/env bash

readonly ARTIFACT_FILE_DIRECTORY=artifact_manifests

# The name of the file that we will merge all artifact JSON
# information into.
readonly ALL_ARTIFACTS_JSON_FILE="all_artifacts.json"
export ALL_ARTIFACTS_JSON_FILE

# Generate a flat JSON object for a number of artifacts that have the
# same version.
#
# The first argument is the version; all others are artifact
# names.
#
# The JSON object is emitted on standard output.
artifact_json() {
    local -r version="${1}"

    # All other arguments are artifact names
    shift
    local -ra artifacts=("${@}")

    for artifact in "${artifacts[@]}"; do
        jq --null-input \
            --arg key "${artifact}" \
            --arg value "${version}" \
            '{"key": $key, "value": $value}'
    done | jq --null-input '[inputs] | from_entries'
}

# Given a directory of JSON files (assumed to represent JSON objects),
# merge them into a single JSON object, sent to standard output.
#
# Clients should favor `merge_artifact_files` over calling this
# function directly. (The logic is implemented this way to facilitate
# testing.)
_merge_artifact_files_impl() {
    local -r directory="${1}"
    jq --slurp 'reduce .[] as $item ({}; . * $item)' "${directory}/"*.json
}

# Merge all the artifact files in `${ARTIFACT_FILE_DIRECTORY}` and
# send the resulting JSON object to standard output.
merge_artifact_files() {
    _merge_artifact_files_impl "${ARTIFACT_FILE_DIRECTORY}"
}

# Generate a file name for an artifacts.json file. The `slug` is just
# a meaningful name you give to describe the file. The
# `${BUILDKITE_JOB_ID}` is also incorporated into the file name, meaning
# that you don't have to care about naming collisions between artifact
# files that are uploaded by different jobs. All you have to do is
# make sure the `slug` is unique within the job.
#
# Additionally, these paths are all inside the
# `${ARTIFACT_FILE_DIRECTORY}`. As a convenience, this function call
# also creates that directory if it does not already exist, ensuring
# that you can use this path without any additional work.
#
#     $ artifacts_file_for monkeypants
#     # => artifact_manifests/monkeypants-e44f9784-e20e-4b93-a21d-f41fd5869db9.artifacts.json
#
artifacts_file_for() {
    local -r slug="${1}"

    mkdir -p "${ARTIFACT_FILE_DIRECTORY}"
    echo "${ARTIFACT_FILE_DIRECTORY}/${slug}-${BUILDKITE_JOB_ID}.artifacts.json"
}
