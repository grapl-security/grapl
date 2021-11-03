#!/usr/bin/env bash

# This script sets up port forwarding between an arbitrary nomad agent server and your local computer.
# This is a temporary workaround until we set up a VPN
# This will allow you to view the UI of a container on the server and/or interact with the container via the cli
#
# Input (optional): Port to forward to. This will be both the port on the server to forward and the local port that is forwarded to. Defaults to port 1234

set -euo pipefail

PORT_TO_FORWARD="${1:-1234}"

SSM_PARAMETERS=$(
    cat << EOF
{
  "portNumber": ["${PORT_TO_FORWARD}"],
  "localPortNumber": ["${PORT_TO_FORWARD}"]
}
EOF
)

echo "Connecting to a nomad agent server in AWS PROFILE: ${AWS_PROFILE:-"No AWS Profile is set, please run export AWS_PROFILE=foo"} on port ${PORT_TO_FORWARD} and forwarding to ${PORT_TO_FORWARD}"
echo "To connect to a nomad agent server in a different AWS Account change your AWS_PROFILE environment variable"

NOMAD_AGENT_INSTANCE_ID=$(
    aws ec2 describe-instances \
        --filter Name=tag:Name,Values="Nomad Agent" \
        --query="Reservations[0].Instances[0].InstanceId" \
        --output=text
)

echo "--- Instance: ${NOMAD_AGENT_INSTANCE_ID}"
echo "To connect to the UI go to http://localhost:${LOCAL_PORT_TO_FORWARD_TO} in your browser"

aws ssm start-session \
    --target "${NOMAD_AGENT_INSTANCE_ID}" \
    --document-name AWS-StartPortForwardingSession \
    --parameters "${SSM_PARAMETERS}"
