#!/bin/bash

set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
GRAPL_ROOT="$(git rev-parse --show-toplevel)"
cd "${THIS_DIR}"

# Ensure script is being run with `local-grapl.env` variables
# via `make start-nomad-dev`
if [[ ! -v LOCALSTACK_PORT ]]; then
    echo "!!! Run this with 'make start-nomad-dev'"
    exit 1
fi

# This guard is strictly informative. nomad agent -dev-connect cannot run without root
if [[ $EUID -ne 0 ]]; then
    echo "This script must be run as root"
    exit 1
fi

# ensure that all dependencies are available
if [[ -z $(command -v nomad) ]]; then
    echo "Nomad must be installed. Please follow the install instructions at https://www.nomadproject.io/downloads"
    exit 2
fi

if [[ -z $(command -v consul) ]]; then
    echo "Consul must be installed. Please follow the install instructions at https://www.consul.io/downloads"
    exit 2
fi

trap 'kill $(jobs -p)' EXIT

echo "Starting nomad and consul locally."
nomad agent -config="nomad-agent-conf.nomad" -dev-connect &
consul agent -dev &
# Potential improvement: Wait 5s and then check the PIDs, see they're still running

# Wait a short period of time before attempting to deploy infrastructure
# shellcheck disable=SC2016
timeout 30 bash -c -- 'while [[ -z $(nomad status | grep running) ]]; do printf "Nomad is not ready";sleep 1;done'

echo "Nomad: http://localhost:4646/"
echo "Consul: http://localhost:8500/"

echo "Deploying local infrastructure."
nomad job run \
    -var "LOCALSTACK_PORT=${LOCALSTACK_PORT}" \
    -var "LOCALSTACK_HOST=${LOCALSTACK_HOST}" \
    -var "FAKE_AWS_ACCESS_KEY_ID=${FAKE_AWS_ACCESS_KEY_ID}" \
    -var "FAKE_AWS_SECRET_ACCESS_KEY=${FAKE_AWS_SECRET_ACCESS_KEY}" \
    "${GRAPL_ROOT}"/nomad/local/grapl-local-infra.nomad

echo "Deployment complete; ctrl + c to terminate".

while true; do read -r; done
