#!/usr/bin/env bash

_get_server_instance() {
    # shellcheck disable=SC2140
    SERVER_INSTANCE_ID=$(
        aws ec2 describe-instances \
            --filters Name=tag:Name,Values="${SERVER_TYPE}" Name="instance-state-name",Values="running" \
            --query="Reservations[0].Instances[0].InstanceId" \
            --output=text
    )
    echo "${SERVER_INSTANCE_ID}"
}

# Inputs
# Server Type
# Remote Port
# Local port
ssm_port_forward() {
    readonly SERVER_TYPE=$1
    readonly REMOTE_PORT=$2
    readonly LOCAL_PORT=$3

    declare -A UI_TYPE_ARRAY=(["Consul Server"]="Consul Server" ["Nomad Agent"]="Grapl Web" ["Nomad Server"]="Nomad Server")

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

    SERVER_INSTANCE_ID=$(_get_server_instance)

    echo "--- Instance: ${SERVER_INSTANCE_ID}"

    echo "To connect to the ${UI_TYPE_ARRAY[${SERVER_TYPE}]} UI go to http://localhost:${LOCAL_PORT} in your browser"

    aws ssm start-session \
        --target "${SERVER_INSTANCE_ID}" \
        --document-name AWS-StartPortForwardingSession \
        --parameters "${SSM_PARAMETERS}"
}

ssm() {
    readonly SERVER_TYPE=$1

    if [ -z "${AWS_PROFILE}" ]; then
        echo "AWS Profile is not set. Please run 'export AWS_PROFILE=foo' and rerun this script"
        exit 1
    fi

    echo "Connecting to a ${SERVER_TYPE} in AWS PROFILE: ${AWS_PROFILE}"
    echo "To connect to a ${SERVER_TYPE} in a different AWS Account change your AWS_PROFILE environment variable"

    SERVER_INSTANCE_ID=$(_get_server_instance)

    echo "--- Instance: ${SERVER_INSTANCE_ID}"

    aws ssm start-session --target "${SERVER_INSTANCE_ID}"
}
