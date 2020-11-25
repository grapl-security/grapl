"""Usage: python3 swarm_setup.py $GRAPL_DEPLOY_NAME"""
import json
import logging
import os
import sys
import time

import boto3
import botocore

from typing import Any, Iterator, List, Tuple

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel("INFO")
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

IN_PROGRESS_STATUSES = {
    "Pending",
    "InProgress",
    "Delayed",
}


def _swarm_instances(ec2: Any) -> Iterator[Tuple[str, str, str]]:
    """Yields tuples of (instance_id, private_ip, hostname) for all the
    instances in the SwarmASG.

    """
    result = ec2.describe_instances(
        Filters=[{"Name": "tag:Name", "Values": ["Grapl/swarm/SwarmCluster/SwarmASG"]}]
    )
    for reservation in result["Reservations"]:
        for instance in reservation["Instances"]:
            if instance["State"]["Name"] != "terminated":
                yield instance["InstanceId"], instance["PrivateIpAddress"], instance[
                    "PrivateDnsName"
                ]


def _get_command_result(ssm: Any, command_id: str, instance_id: str) -> str:
    """Poll until the command result is available for the given
    command_id. Returns the command result.

    """
    while 1:
        commands = ssm.list_commands(CommandId=command_id)
        if commands["Commands"][0]["Status"] in IN_PROGRESS_STATUSES:
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


def _create_disk_usage_alarms(cloudwatch: Any, instance_id: str) -> None:
    """Create disk usage alarms for the / and /dgraph partitions"""
    cloudwatch.put_metric_alarm(
        AlarmName=f"/ disk_used_percent ({instance_id})",
        AlarmDescription=f"Root volume disk usage percent threshold exceeded on {instance_id}",
        ActionsEnabled=False,
        MetricName="disk_used_percent",
        Namespace="CWAgent",
        Statistic="Maximum",
        Period=300,
        EvaluationPeriods=1,
        ComparisonOperator="GreaterThanOrEqualToThreshold",
        Threshold=95.0,
        Unit="Percent",
        Dimensions=[
            {"Name": "InstanceId", "Value": instance_id},
            {"Name": "path", "Value": "/"},
        ],
    )
    cloudwatch.put_metric_alarm(
        AlarmName=f"/dgraph disk_used_percent ({instance_id})",
        AlarmDescription=f"DGraph volume disk usage percent threshold exceeded on {instance_id}",
        ActionsEnabled=False,
        MetricName="disk_used_percent",
        Namespace="CWAgent",
        Statistic="Maximum",
        Period=300,
        EvaluationPeriods=1,
        ComparisonOperator="GreaterThanOrEqualToThreshold",
        Threshold=95.0,
        Unit="Percent",
        Dimensions=[
            {"Name": "InstanceId", "Value": instance_id},
            {"Name": "path", "Value": "/dgraph"},
        ],
    )


def _init_docker_swarm(
    ec2: Any,
    ssm: Any,
    cloudwatch: Any,
    prefix: str,
    manager_id: str,
    manager_ip: str,
    manager_hostname: str,
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
    _create_disk_usage_alarms(cloudwatch, manager_id)
    LOGGER.info(
        f"Instance {manager_id} with IP {manager_ip} and hostname {manager_hostname} is the docker swarm cluster manager"
    )
    return result


def _join_worker_nodes(
    ec2: Any,
    ssm: Any,
    cloudwatch: Any,
    prefix: str,
    instances: Iterator[str],
    join_token: str,
    manager_ip: str,
) -> List[str]:
    """Join worker nodes to the swarm cluster. Returns hostnames of the
    worker nodes."""
    hostnames = []
    for instance_id, _, hostname in instances:
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
        _create_disk_usage_alarms(cloudwatch, instance_id)
        LOGGER.info(
            f"Joined worker instance {instance_id} with hostname {hostname} to the docker swarm cluster"
        )
        hostnames.append(hostname)
    return hostnames


def main(prefix: str) -> None:
    ec2 = boto3.client("ec2")

    LOGGER.info("Retrieving instance IDs")
    instances = _swarm_instances(ec2)

    ssm = boto3.client("ssm")
    cloudwatch = boto3.client("cloudwatch")

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

aws s3 cp s3://$GRAPL_DEPLOY_NAME-dgraph-config-bucket/docker-compose-dgraph.yml .
aws s3 cp s3://$GRAPL_DEPLOY_NAME-dgraph-config-bucket/envoy.yaml .

docker stack deploy -c docker-compose-dgraph.yml dgraph

docker service ls

"""
    )


if __name__ == "__main__":
    main(prefix=sys.argv[1])
