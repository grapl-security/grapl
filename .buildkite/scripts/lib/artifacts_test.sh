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

test__merge_artifact_files_impl() {
    echo '{"first": 1}' > first.json
    echo '{"second": 2}' > second.json
    echo '{"third": 3}' > third.json
    echo '{"first": "overwritten!"}' > fourth.json

    actual="$(_merge_artifact_files_impl .)"

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
