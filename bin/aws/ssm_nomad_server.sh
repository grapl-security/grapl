#!/usr/bin/env bash

set -euo pipefail

# the 4846 is an arbitrary choice that is indentifiably related to nomad's 4646

SERVERS=$(aws ec2 describe-instances --filter Name="tag:Name",Values="Nomad Server")
NOMAD_SERVER_INSTANCE_ID=$(echo "${SERVERS}" | jq -r .Reservations[0].Instances[0].InstanceId)

aws ssm start-session \
    --target "${NOMAD_SERVER_INSTANCE_ID}" \
    --document-name AWS-StartPortForwardingSession \
    --parameters '{"portNumber":["4646"], "localPortNumber": ["4846"]}'
