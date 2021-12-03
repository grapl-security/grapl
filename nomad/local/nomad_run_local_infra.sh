#!/bin/bash
set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
GRAPL_ROOT="$(git rev-parse --show-toplevel)"
NOMAD_FILE="${GRAPL_ROOT}/nomad/local/grapl-local-infra.nomad"

declare -a NOMAD_VARS=(
    -var "KAFKA_JMX_PORT=${KAFKA_JMX_PORT}"
    -var "KAFKA_BROKER_PORT=${KAFKA_BROKER_PORT}"
    -var "KAFKA_BROKER_PORT_FOR_HOST_OS=${KAFKA_BROKER_PORT_FOR_HOST_OS}"
    -var "LOCALSTACK_PORT=${LOCALSTACK_PORT}"
    -var "ZOOKEEPER_PORT=${ZOOKEEPER_PORT}"
    -var "FAKE_AWS_ACCESS_KEY_ID=${FAKE_AWS_ACCESS_KEY_ID}"
    -var "FAKE_AWS_SECRET_ACCESS_KEY=${FAKE_AWS_SECRET_ACCESS_KEY}"
    -var "localstack_tag=${TAG}"
)

# shellcheck source-path=SCRIPTDIR
source "${THIS_DIR}/../lib/nomad_cli_tools.sh"

echo "--- Deploying Nomad local infrastructure."

# Wait a short period of time before attempting to deploy infrastructure
# shellcheck disable=SC2016
timeout 60 bash -c -- 'while [[ -z $(nomad node status 2>&1 | grep ready) ]]; do printf "Waiting for nomad-agent\n";sleep 1;done'

nomad job validate "${NOMAD_VARS[@]}" "${NOMAD_FILE}"

# Okay, now the Nomad agent is up, but it might not be ready to accept jobs.
# Just loop on `nomad job plan` until it can.
attemptPlan() {
    nomad job plan "${NOMAD_VARS[@]}" "${NOMAD_FILE}" > /dev/null 2>&1
    echo "$?"
}

# fyi: Exit code 1 means "Allocations created or destroyed" and is what we want
while [[ $(attemptPlan) -ne 1 ]]; do
    echo "Waiting to be able to do a nomad-job-run"
    sleep 1
done

nomad job run -verbose "${NOMAD_VARS[@]}" "${NOMAD_FILE}"

echo "Nomad Job Run complete, checking for task failures"

check_for_task_failures_in_job "grapl-local-infra"

echo "--- Nomad local-infra deployed!"
