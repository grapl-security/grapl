#!/usr/bin/env bash

# This script sets up port forwarding between an arbitrary consul server and your local computer.
# This is a temporary workaround until we set up a VPN
# This will allow you to view the consul UI and interact with consul in the cli
#
# Input (optional): Local port to forward to. Defaults to port 8500

set -euo pipefail

if [ -z "${AWS_PROFILE}" ]; then
    echo "AWS Profile is not set. Please run 'export AWS_PROFILE=foo' and rerun this script"
    exit 1
fi

LOCAL_PORT_TO_FORWARD_TO="${1:-8500}"

SSM_PARAMETERS=$(
    cat << EOF
{
  "portNumber": ["8500"],
  "localPortNumber": ["${LOCAL_PORT_TO_FORWARD_TO}"]
}
EOF
)

echo "Connecting to a consul server in AWS PROFILE: ${AWS_PROFILE} on port 8500 and forwarding to ${LOCAL_PORT_TO_FORWARD_TO}"
echo "To connect to a consul server in a different AWS Account change your AWS_PROFILE environment variable"

CONSUL_SERVER_INSTANCE_ID=$(
    aws ec2 describe-instances \
        --filter Name=tag:Name,Values="Consul Server" \
        --query="Reservations[0].Instances[0].InstanceId" \
        --output=text
)

echo "--- Instance: ${CONSUL_SERVER_INSTANCE_ID}"

echo "To connect to the consul UI go to http://localhost:${LOCAL_PORT_TO_FORWARD_TO} in your browser"

aws ssm start-session \
    --target "${CONSUL_SERVER_INSTANCE_ID}" \
    --document-name AWS-StartPortForwardingSession \
    --parameters "${SSM_PARAMETERS}"
