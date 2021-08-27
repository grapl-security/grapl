#!/bin/bash
set -euo pipefail

GRAPL_ROOT="$(git rev-parse --show-toplevel)"

echo "Deploying local infrastructure."

# Wait a short period of time before attempting to deploy infrastructure
# shellcheck disable=SC2016
timeout 30 bash -c -- 'while [[ -z $(nomad status| grep running) ]]; do printf "Waiting for nomad-agent\n";sleep 1;done'

# Before we deploy Localstack, we need to ensure the Docker network exists
docker network create grapl-network || true

nomad job run \
    -var "LOCALSTACK_PORT=${LOCALSTACK_PORT}" \
    -var "LOCALSTACK_HOST=${LOCALSTACK_HOST}" \
    -var "FAKE_AWS_ACCESS_KEY_ID=${FAKE_AWS_ACCESS_KEY_ID}" \
    -var "FAKE_AWS_SECRET_ACCESS_KEY=${FAKE_AWS_SECRET_ACCESS_KEY}" \
    "${GRAPL_ROOT}"/nomad/local/grapl-local-infra.nomad

echo "--- Nomad jobs deployed!"