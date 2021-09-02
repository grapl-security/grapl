#!/bin/bash
set -euo pipefail

GRAPL_ROOT="$(git rev-parse --show-toplevel)"
THIS_DIR=$(dirname "${BASH_SOURCE[0]}")

# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/nomad_cli_tools.sh"

echo "--- Deploying integration tests"

# Wait a short period of time before attempting to deploy infrastructure
# shellcheck disable=SC2016
timeout 30 bash -c -- 'while [[ -z $(nomad status 2>&1 | grep running) ]]; do printf "Waiting for nomad-agent\n";sleep 1;done'

nomad job run \
    -var "aws_region=${AWS_REGION}" \
    -var "deployment_name=${DEPLOYMENT_NAME}" \
    -var "aws_access_key_id=${FAKE_AWS_ACCESS_KEY_ID}" \
    -var "aws_access_key_secret=${FAKE_AWS_SECRET_ACCESS_KEY}" \
    -var "aws_endpoint=${GRAPL_AWS_ENDPOINT}" \
    -var "redis_endpoint=${REDIS_ENDPOINT}" \
    "${GRAPL_ROOT}"/nomad/local/integration-tests.nomad

echo "--- Integration tests deployed!"

# Now we have to actually dispatch a job; the above command simply uploaded
# the jobspec, since it's a Parameterized Batch Job.

echo "--- Dispatching integration tests"

DISPATCH_JOB_ID=$(nomad_dispatch integration-tests)
echo "${DISPATCH_JOB_ID}"
