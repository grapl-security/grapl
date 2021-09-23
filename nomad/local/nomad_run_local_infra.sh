#!/bin/bash
set -euo pipefail

GRAPL_ROOT="$(git rev-parse --show-toplevel)"

echo "Deploying local infrastructure."

# Wait a short period of time before attempting to deploy infrastructure
# shellcheck disable=SC2016
timeout 120 bash -c -- 'while [[ -z $(nomad status 2>&1 | grep running) ]]; do printf "Waiting for nomad-agent\n";sleep 1;done'

# Before we deploy Localstack, we need to ensure the Docker network exists
docker network create grapl-network || true

nomad job run \
    -var "KAFKA_JMX_PORT=${KAFKA_JMX_PORT}" \
    -var "KAFKA_BROKER_PORT=${KAFKA_BROKER_PORT}" \
    -var "KAFKA_BROKER_PORT_FOR_HOST_OS=${KAFKA_BROKER_PORT_FOR_HOST_OS}" \
    -var "LOCALSTACK_PORT=${LOCALSTACK_PORT}" \
    -var "ZOOKEEPER_PORT=${ZOOKEEPER_PORT}" \
    -var "FAKE_AWS_ACCESS_KEY_ID=${FAKE_AWS_ACCESS_KEY_ID}" \
    -var "FAKE_AWS_SECRET_ACCESS_KEY=${FAKE_AWS_SECRET_ACCESS_KEY}" \
    "${GRAPL_ROOT}"/nomad/local/grapl-local-infra.nomad

echo -e "\n--- Nomad local-infra deployed!"
