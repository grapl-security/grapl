#!/usr/bin/env bash

# no unset variables please
set -eu

# TODO https://github.com/grapl-security/issue-tracker/issues/393
# Remove this in favor of something like `graplctl dev upload_builtin_analyzers`

ANALYZER_UPLOAD_SCRIPT_PATH=$(dirname "$(readlink -f "$0")")
LOCAL_GRAPL_DIR="${ANALYZER_UPLOAD_SCRIPT_PATH}/../local_grapl"

aws s3 cp \
    "${LOCAL_GRAPL_DIR}/suspicious_svchost/main.py" \
    "s3://${DEPLOYMENT_NAME}-analyzers-bucket/analyzers/suspicious_svchost/main.py"

aws s3 cp \
    "${LOCAL_GRAPL_DIR}/unique_cmd_parent/main.py" \
    "s3://${DEPLOYMENT_NAME}-analyzers-bucket/analyzers/unique_cmd_parent/main.py"
