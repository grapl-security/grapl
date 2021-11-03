#!/usr/bin/env bash

# This script sets up port forwarding between an arbitrary nomad server and your local computer.
# This is a temporary workaround until we set up a VPN
# This will allow you to view the nomad UI and interact with nomad in the cli. In particular, this is used by pulumi to deploy Nomad jobs
#
# Input (optional): Local port to forward to. Defaults to port 4646

set -euo pipefail

LOCAL_PORT_TO_FORWARD_TO="${1:-4646}"

SSM_PARAMETERS=$(
    cat << EOF
{
  "portNumber": ["4646"],
  "localPortNumber": ["${LOCAL_PORT_TO_FORWARD_TO}"]
}
EOF
)

echo "Connecting to a nomad server in AWS PROFILE: ${AWS_PROFILE:-"No AWS Profile is set, please run export AWS_PROFILE=foo"} on port 4646 and forwarding to ${LOCAL_PORT_TO_FORWARD_TO}"
echo "To connect to a nomad server in a different AWS Account change your AWS_PROFILE environment variable"

NOMAD_SERVER_INSTANCE_ID=$(
    aws ec2 describe-instances \
        --filter Name=tag:Name,Values="Nomad Server" \
        --query="Reservations[0].Instances[0].InstanceId" \
        --output=text
)

echo "--- Instance: ${NOMAD_SERVER_INSTANCE_ID}"
echo "To connect to the nomad UI go to http://localhost:${LOCAL_PORT_TO_FORWARD_TO} in your browser"

aws ssm start-session \
    --target "${NOMAD_SERVER_INSTANCE_ID}" \
    --document-name AWS-StartPortForwardingSession \
    --parameters "${SSM_PARAMETERS}"
