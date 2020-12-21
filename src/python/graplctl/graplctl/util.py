from __future__ import annotations

import json
import logging
import sys
import time

from typing import TYPE_CHECKING, Any, Iterator, List, NamedTuple, Optional

import boto3

if TYPE_CHECKING:
    from mypy_boto3_ec2 import EC2ServiceResource
    from mypy_boto3_cloudwatch.client import CloudWatchClient
    from mypy_boto3_cloudwatch.type_defs import MetricTypeDef
    from mypy_boto3_sns.client import SNSClient

def _get_command_result(ssm: Any, command_id: str, instance_id: str) -> str:
    """Poll until the command result is available for the given
    command_id. Returns the command result.

    """
    while 1:
        commands = ssm.list_commands(CommandId=command_id)
        if commands["Commands"][0]["Status"] in IN_PROGRESS_STATUSES:
            LOGGER.info(f"Waiting for SSM command {command_id} to complete")
            time.sleep(2)
        else:
            break

    invocation = ssm.get_command_invocation(
        CommandId=command_id,
        InstanceId=instance_id,
        PluginName="runShellScript",
    )

    if invocation["Status"] == "Success":
        return invocation["StandardOutputContent"].strip()
    else:
        raise Exception(f"SSM Command failed with Status: \"{invocation['Status']}\"")


def main(prefix: str) -> None:
    ec2 = boto3.client("ec2")

    LOGGER.info("Retrieving instance IDs")
    instances = _swarm_instances(ec2)

    ssm = boto3.client("ssm")
    cloudwatch: CloudWatchClient = boto3.client("cloudwatch")

    LOGGER.info("Initializing swarm manager")
    manager_id, manager_ip, manager_hostname = next(instances)
    join_token = _init_docker_swarm(
        ec2, ssm, cloudwatch, prefix, manager_id, manager_ip, manager_hostname
    )

    LOGGER.info("Joining worker instances")
    worker_hostnames = _join_worker_nodes(
        ec2, ssm, cloudwatch, prefix, instances, join_token, manager_ip
    )

    LOGGER.info("Docker swarm cluster setup complete")

    print(
        f"""
##
# Paste the following into an SSM shell on the swarm
# manager ({manager_id}) to deploy dgraph to
# the swarm cluster:
##

sudo su ec2-user
cd $HOME

export GRAPL_DEPLOY_NAME="{prefix}"
export AWS_LOGS_GROUP="{prefix}-grapl-dgraph"
export AWS01_NAME="{manager_hostname}"
export AWS02_NAME="{worker_hostnames[0]}"
export AWS03_NAME="{worker_hostnames[1]}"

aws s3 cp s3://${{GRAPL_DEPLOY_NAME,,}}-dgraph-config-bucket/docker-compose-dgraph.yml .
aws s3 cp s3://${{GRAPL_DEPLOY_NAME,,}}-dgraph-config-bucket/envoy.yaml .

docker stack deploy -c docker-compose-dgraph.yml dgraph

docker service ls

"""
    )
