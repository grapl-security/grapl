#!/usr/bin/env bash

set -euo pipefail

flatten_json() {
    # The purpose of this module is to convert something like the following json:
    # {
    #     "some-amis": {
    #         "us-east-1": "ami-111",
    #     }
    # }
    # into { "some-amis.us-east-1": "ami-111" }

    local -r input_json="${1}"
    # https://stackoverflow.com/a/37557003
    jq -r '
        . as $in
        | reduce paths(scalars) as $path (
            {};
            . + { ($path | map(tostring) | join(".")): $in | getpath($path) }
        )
    ' <<< "${input_json}"
}
