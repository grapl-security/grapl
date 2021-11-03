#!/usr/bin/env bash

set -euo pipefail

SERVERS=$(aws ec2 describe-instances --filter Name=tag:Name,Values="Consul Server")
CONSUL_SERVER_INSTANCE_ID=$(echo "${SERVERS}" | jq -r .Reservations[0].Instances[0].InstanceId)

echo "--- Instance: ${CONSUL_SERVER_INSTANCE_ID}"

aws ssm start-session \
    --target "${CONSUL_SERVER_INSTANCE_ID}" \
    --document-name AWS-StartPortForwardingSession \
    --parameters '{"portNumber":["8500"], "localPortNumber": ["8500"]}'
