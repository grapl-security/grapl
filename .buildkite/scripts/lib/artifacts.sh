#!/usr/bin/env bash

# The name of the file that we will merge all artifact JSON
# information into.
readonly ALL_ARTIFACTS_JSON_FILE="all_artifacts.json"
export ALL_ARTIFACTS_JSON_FILE

# The extension all our artifact JSON files will have. Such a file
# should contain a single flat JSON object.
#
# The extension includes "grapl" to prevent the (admittedly unlikely)
# scenario of somehow having some _other_ file with an
# "artifacts.json" extension sneaking into a file glob somewhere.
#
# It also makes things _super_ obvious and searchable.
readonly ARTIFACTS_FILE_EXTENSION="grapl-artifacts.json"

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

# Merge all the artifact JSON files in the current directory and send
# the resulting JSON object to standard output.
merge_artifact_files() {
    jq --slurp 'reduce .[] as $item ({}; . * $item)' -- *".${ARTIFACTS_FILE_EXTENSION}"
}

# Generate a file name for an artifacts JSON file. The `slug` is just
# a meaningful name you give to describe the file. The
# `${BUILDKITE_JOB_ID}` is also incorporated into the file name,
# meaning that you don't have to care about naming collisions between
# artifact files that are uploaded by different jobs. All you have to
# do is make sure the `slug` is unique within the job.
#
#     $ artifacts_file_for monkeypants
#     # => monkeypants-e44f9784-e20e-4b93-a21d-f41fd5869db9.grapl-artifacts.json
#
artifacts_file_for() {
    local -r slug="${1}"
    echo "${slug}-${BUILDKITE_JOB_ID}.${ARTIFACTS_FILE_EXTENSION}"
}
