#!/bin/bash

set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
GRAPL_ROOT="$(git rev-parse --show-toplevel)"
cd "${THIS_DIR}"

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
NOMAD_AGENT_PID=$!
consul agent -dev &
CONSUL_AGENT_PID=$!

# Wait a short period of time before attempting to deploy infrastructure
sleep 10s
echo "Checking health"

# TODO: Use the PIDs to ask, are the agents still alive?

echo "Nomad: http://localhost:4646/"
echo "Consul: http://localhost:8500/"

# Import environment variables from `local-grapl.env`
set -o allexport; . ${GRAPL_ROOT}/local-grapl.env; set +o allexport;

GRAPL_LOCAL_VARS="
-var LOCALSTACK_PORT=${LOCALSTACK_PORT}
-var LOCALSTACK_HOST=${LOCALSTACK_HOST}
-var FAKE_AWS_ACCESS_KEY_ID=${FAKE_AWS_ACCESS_KEY_ID}
-var FAKE_AWS_SECRET_ACCESS_KEY=${FAKE_AWS_SECRET_ACCESS_KEY}
"

echo "Deploying local infrastructure."
nomad job run \
    ${GRAPL_LOCAL_VARS} \
    ${GRAPL_ROOT}/nomad/local/grapl-local-infra.nomad

echo "Deployment complete; ctrl + c to terminate".

while true; do read -r; done
