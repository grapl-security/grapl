#!/bin/bash

set -euo pipefail

THIS_DIR=$(dirname "${BASH_SOURCE[0]}")
cd "${THIS_DIR}"

# Ensure script is being run with `local-grapl.env` variables
# via `make start-nomad-ci`
if [[ ! -v DOCKER_REGISTRY ]]; then
    echo "!!! Run this with 'make start-nomad-ci'"
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

NOMAD_LOGS_DEST=/tmp/nomad-agent.log
CONSUL_LOGS_DEST=/tmp/consul-agent.log
echo "Starting nomad and consul locally. Logs @ ${NOMAD_LOGS_DEST} and ${CONSUL_LOGS_DEST}."
# These will run forever until `make stop-nomad-ci` is invoked."
nomad agent -config="nomad-agent-conf.nomad" -dev-connect > "${NOMAD_LOGS_DEST}" &
consul agent -dev > "${CONSUL_LOGS_DEST}" &

./nomad_run_local_infra.sh
echo "Deployment complete"
