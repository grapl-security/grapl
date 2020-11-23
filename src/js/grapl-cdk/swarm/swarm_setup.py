"""Usage: python3 swarm_setup.py $GRAPL_DEPLOY_NAME"""
import json
import logging
import os
import sys
import time

import boto3
import botocore

from typing import Any, Iterator, Tuple

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel("INFO")
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

IN_PROGRESS_STATUSES = {
    "Pending",
    "In Progress",
    "Delayed",
}


def _swarm_instances(ec2: Any) -> Iterator[Tuple[str, str]]:
    """Yields tuples of (instance_id, private_ip) for all the instances in
    the SwarmASG.

    """
    result = ec2.describe_instances(
        Filters=[{"Name": "tag:Name", "Values": ["Grapl/swarm/SwarmCluster/SwarmASG"]}]
    )
    for reservation in result["Reservations"]:
        for instance in reservation["Instances"]:
            if instance["State"]["Name"] != "terminated":
                yield instance["InstanceId"], instance["PrivateIpAddress"]


def _get_command_result(ssm: Any, command_id: str, instance_id: str) -> str:
    """Poll until the command result is available for the given
    command_id. Returns the command result.

    """
    invocation = None
    while invocation is None:
        try:
            invocation = ssm.get_command_invocation(
                CommandId=command_id,
                InstanceId=instance_id,
                #PluginName="aws:runShellScript",
            )
        except botocore.exceptions.ClientError as e:
            if e.response["Error"]["Code"] == "InvocationDoesNotExist":
                time.sleep(2)
                continue
            else:
                raise e

    while invocation["Status"] in IN_PROGRESS_STATUSES:
        time.sleep(2)
        try:
            invocation = ssm.get_command_invocation(
                CommandId=command_id,
                InstanceId=instance_id,
                #PluginName="aws:runShellScript",
            )
        except botocore.exceptions.ClientError as e:
            if e.response["Error"]["Code"] == "InvocationDoesNotExist":
                continue
            else:
                raise e
    if invocation["Status"] == "Success":
        return invocation["StandardOutputContent"]
    else:
        raise Exception(f"SSM Command failed with Status: \"{invocation['Status']}\"")


def _init_docker_swarm(
    ec2: Any, ssm: Any, prefix: str, manager_id: str, manager_ip: str
) -> str:
    """Initialize the docker swarm manager. Returns the join token
    necessary to attach workers to the swarm.

    """
    command = ssm.send_command(
        # Targets=[{"Key": "tag:Name", "Values": ["Grapl/swarm/SwarmCluster/SwarmASG"]}],
        InstanceIds=[manager_id],
        DocumentName="AWS-RunRemoteScript",
        Parameters={
            "sourceType": ["S3"],
            "sourceInfo": [
                json.dumps(
                    {
                        "path": f"https://s3.amazonaws.com/{prefix.lower()}-swarm-config-bucket/swarm_init.py"
                    }
                )
            ],
            "commandLine": ["python3 swarm_init.py"],
        },
    )
    command_id = command["Command"]["CommandId"]
    result = _get_command_result(ssm, command_id, manager_id)
    ec2.create_tags(
        Resources=[manager_id],
        Tags=[{"Key": "grapl-swarm-role", "Value": "swarm-manager"}],
    )
    LOGGER.info(
        f"Instance {manager_id} with IP {manager_ip} is the docker swarm cluster manager"
    )
    return result


def _join_worker_nodes(
    ec2: Any,
    ssm: Any,
    prefix: str,
    instance_ids: Iterator[str],
    join_token: str,
    manager_ip: str,
) -> None:
    """Join worker nodes to the swarm cluster."""
    for instance_id, _ in instance_ids:
        command = ssm.send_command(
            # Targets=[{"Key": "tag:Name", "Values": ["Grapl/swarm/SwarmCluster/SwarmASG"]}],
            InstanceIds=[instance_id],
            DocumentName="AWS-RunRemoteScript",
            Parameters={
                "sourceType": ["S3"],
                "sourceInfo": [
                    json.dumps(
                        {
                            "path": f"https://s3.amazonaws.com/{prefix.lower()}-swarm-config-bucket/swarm_join.py"
                        }
                    )
                ],
                "commandLine": [f"python3 swarm_join.py {join_token} {manager_ip}"],
            },
        )
        command_id = command["Command"]["CommandId"]
        _get_command_result(ssm, command_id, instance_id)
        ec2.create_tags(
            Resources=[instance_id],
            Tags=[{"Key": "grapl-swarm-role", "Value": "swarm-worker"}],
        )
        LOGGER.info(f"Joined worker instance {instance_id} to the docker swarm cluster")


def main(prefix: str) -> None:
    ec2 = boto3.client("ec2")

    LOGGER.info("Retrieving instance IDs")
    instances = _swarm_instances(ec2)

    ssm = boto3.client("ssm")

    LOGGER.info("Initializing swarm manager")
    manager_id, manager_ip = next(instances)
    join_token = _init_docker_swarm(ec2, ssm, prefix, manager_id, manager_ip)

    LOGGER.info("Joining worker instances")
    _join_worker_nodes(ec2, ssm, prefix, instances, join_token, manager_ip)

    LOGGER.info("Docker swarm cluster setup complete")


if __name__ == "__main__":
    main(prefix=sys.argv[1])
