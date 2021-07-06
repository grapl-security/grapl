#!/usr/bin/env bash

oneTimeSetUp() {
    source .buildkite/scripts/lib/lambda.sh
}

test_lambda_name_from_zip() {
    assertEquals \
        "ux-router" \
        "$(lambda_name_from_zip ux-router-lambda.zip)"
}
