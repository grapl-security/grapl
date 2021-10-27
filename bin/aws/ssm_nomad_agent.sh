#!/usr/bin/env bash

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

AGENTS=$(aws ec2 describe-instances --filter Name="tag:Name",Values="Nomad Agent")
NOMAD_AGENT_INSTANCE_ID=$(echo "${AGENTS}" | jq -r .Reservations[0].Instances[0].InstanceId)

aws ssm start-session \
    --target "${NOMAD_AGENT_INSTANCE_ID}" \
    --document-name AWS-StartPortForwardingSession \
    --parameters "${SSM_PARAMETERS}"
