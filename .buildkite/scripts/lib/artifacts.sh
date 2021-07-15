#!/usr/bin/env bash

readonly ARTIFACT_FILE_DIRECTORY=all_artifacts_files

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

# Download a given artifact file from Buildkite into
# `$ARTIFACT_FILE_DIRECTORY`.
#
# Does not fail if the file was not generated and uploaded during the
# current Buildkite pipeline (which is a legitimate scenario - for example,
# we only *sometimes* generate new AMI IDs.)
download_artifact_file() {
    local -r artifacts_file="${1}"
    mkdir -p "${ARTIFACT_FILE_DIRECTORY}"
    echo -e "--- :buildkite: Download '${artifacts_file}' artifacts file"
    if ! (buildkite-agent artifact download "${artifacts_file}" "${ARTIFACT_FILE_DIRECTORY}"); then
        echo "^^^ +++" # Un-collapses this section in Buildkite, making it more obvious we couldn't download
        echo "No file found"
    fi
    # TODO: Would be nice to validate the artifacts. Right now there are some restrictions:
    # - json file must be a flat associative array of key -> primitive (no nested maps, arrays)
    #     bad: {"im": {"nested": true}}
    #     good: {"im.nested": true}
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
