#!/usr/bin/env bash

oneTimeSetUp() {
    source .buildkite/scripts/lib/artifacts.sh
}

test_artifact_json_small() {
    actual="$(artifact_json "1.0.0" "artifact_one")"
    expected=$(
        cat << EOF
{
  "artifact_one": "1.0.0"
}
EOF
    )

    assertEquals "Failed to generate expected JSON" \
        "${expected}" \
        "${actual}"
}

test_artifact_json_large() {
    actual="$(artifact_json 1.0.0 artifact_one artifact_two artifact_three artifact_four)"
    expected=$(
        cat << EOF
{
  "artifact_one": "1.0.0",
  "artifact_two": "1.0.0",
  "artifact_three": "1.0.0",
  "artifact_four": "1.0.0"
}
EOF
    )

    assertEquals "Failed to generate expected JSON" \
        "${expected}" \
        "${actual}"
}

test_merge_artifact_files() {
    echo '{"first": 1}' > "first.${ARTIFACTS_FILE_EXTENSION}"
    echo '{"second": 2}' > "second.${ARTIFACTS_FILE_EXTENSION}"
    echo '{"third": 3}' > "third.${ARTIFACTS_FILE_EXTENSION}"
    echo '{"first": "overwritten!"}' > "fourth.${ARTIFACTS_FILE_EXTENSION}"

    actual="$(merge_artifact_files)"

    expected=$(
        cat << EOF
{
  "first": "overwritten!",
  "second": 2,
  "third": 3
}
EOF
    )

    assertEquals "Failed to generate expected JSON" \
        "${expected}" \
        "${actual}"
}

test_artifacts_file_for() {
    actual=$(
        export BUILDKITE_JOB_ID="aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee"
        artifacts_file_for stuff
    )

    expected="stuff-aaaaaaaa-bbbb-cccc-dddd-eeeeeeeeeeee.grapl-artifacts.json"
    assertEquals "Failed to generate expected artifacts file name!" \
        "${expected}" \
        "${actual}"
}
