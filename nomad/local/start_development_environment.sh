#!/bin/bash
if [[ $EUID -ne 0 ]]; then
    echo "This script must be run as root"
    exit 1
fi

trap 'kill $(jobs -p)' EXIT

echo "Starting nomad and consul locally."
nomad agent -dev-connect > /dev/null 2>&1 &
consul agent -dev > /dev/null 2>&1 &

# Wait a short period of time before attempting to deploy infrastructure
sleep 5s

echo "Nomad: http://localhost:4646/"
echo "Consul: http://localhost:8500/"

echo "Deploying local infrastructure."
nomad job run nomad/local/grapl-local-infra.nomad

echo "Deployment complete; ctrl + c to terminate".

while true; do read -r; done
