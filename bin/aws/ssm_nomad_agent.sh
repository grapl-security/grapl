#!/usr/bin/env bash

set -euo pipefail

# the 4946 is an arbitrary choice that is indentifiably related to nomad's 4646

AGENTS=$(aws ec2 describe-instances --filter Name="tag:Name",Values="Nomad Agent")
NOMAD_AGENT_INSTANCE_ID=$(echo "${AGENTS}" | jq -r .Reservations[0].Instances[0].InstanceId)

aws ssm start-session \
    --target "${NOMAD_AGENT_INSTANCE_ID}" \
    --document-name AWS-StartPortForwardingSession \
    --parameters '{"portNumber":["1234"], "localPortNumber": ["4946"]}'
