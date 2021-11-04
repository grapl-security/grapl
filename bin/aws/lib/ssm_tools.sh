#!/usr/bin/env bash

# Inputs
# Type
# Remote Port
# Local port
ssm_port_forward() {
    readonly SERVER_TYPE=$1
    readonly REMOTE_PORT=$2
    readonly LOCAL_PORT=$3

    if [ -z "${AWS_PROFILE}" ]; then
        echo "AWS Profile is not set. Please run 'export AWS_PROFILE=foo' and rerun this script"
        exit 1
    fi

    SSM_PARAMETERS=$(
        cat << EOF
{
 "portNumber": ["${REMOTE_PORT}"],
 "localPortNumber": ["${LOCAL_PORT}"]
}
EOF
    )

    echo "Connecting to a ${SERVER_TYPE} in AWS PROFILE: ${AWS_PROFILE} on port ${REMOTE_PORT} and forwarding to ${LOCAL_PORT}"
    echo "To connect to a ${SERVER_TYPE} in a different AWS Account change your AWS_PROFILE environment variable"

    # shellcheck disable=SC2140
    SERVER_INSTANCE_ID=$(
        aws ec2 describe-instances \
            --filter Name=tag:Name,Values="${SERVER_TYPE}" \
            --filter Name="instance-state-name",Values="running" \
            --query="Reservations[0].Instances[0].InstanceId" \
            --output=text
    )

    echo "--- Instance: ${SERVER_INSTANCE_ID}"

    echo "To connect to the ${SERVER_TYPE} UI go to http://localhost:${LOCAL_PORT} in your browser"

    aws ssm start-session \
        --target "${SERVER_INSTANCE_ID}" \
        --document-name AWS-StartPortForwardingSession \
        --parameters "${SSM_PARAMETERS}"
}
