#!/bin/bash

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
nomad agent -dev-connect &
consul agent -dev &

# Wait a short period of time before attempting to deploy infrastructure
sleep 5s

echo "Nomad: http://localhost:4646/"
echo "Consul: http://localhost:8500/"

echo "Deploying local infrastructure."
nomad job run nomad/local/grapl-local-infra.nomad

echo "Deployment complete; ctrl + c to terminate".

while true; do read -r; done
