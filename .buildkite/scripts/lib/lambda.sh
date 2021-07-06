#!/usr/bin/env bash

# Helper functions for dealing with our lambda ZIP files.

# All our Lambda ZIP files end in this suffix, e.g.,
# "ux-router-lambda.zip"
readonly LAMBDA_SUFFIX="-lambda.zip"

# This file contains a single flat JSON object describing
# artifact/version pairs for adding to the `artifacts` object in our
# Pulumi stack configuration files.
#
# When a pipeline generates artifacts, it should record this in a file
# of this name and upload it as a Buildkite artifact for consumption
# in other jobs.
#
# (This is only for Lambda zip files)
readonly LAMBDA_ARTIFACTS_FILE="lambda_artifacts.json"
export LAMBDA_ARTIFACTS_FILE

# Extracts the name of a lambda function from a file name based on our
# naming convention.
#
#     lambda_name_from_zip "ux-router-lambda.zip"
#     # => "ux-router"
#
lambda_name_from_zip() {
    local -r zip_file="${1}"
    basename "${zip_file}" "${LAMBDA_SUFFIX}"
}
