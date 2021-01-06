"""Usage: python3 swarm_setup.py $GRAPL_DEPLOY_NAME"""
from __future__ import annotations

import json
import logging
import sys
import time
from typing import TYPE_CHECKING, Any, Iterator, List, NamedTuple, Optional, Sequence

import boto3

if TYPE_CHECKING:
    from mypy_boto3_cloudwatch.client import CloudWatchClient
    from mypy_boto3_cloudwatch.type_defs import MetricTypeDef
    from mypy_boto3_sns.client import SNSClient

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel("INFO")
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

IN_PROGRESS_STATUSES = {
    "Pending",
    "InProgress",
    "Delayed",
}

InstanceTuple = NamedTuple(
    "InstanceTuple",
    (("instance_id", str), ("private_ip_address", str), ("private_dns_name", str),),
)


def _swarm_instances(ec2: Any) -> Iterator[InstanceTuple]:
    """Yields tuples of (instance_id, private_ip, hostname) for all the
    instances in the SwarmASG.

    """
    result = ec2.describe_instances(
        Filters=[{"Name": "tag:Name", "Values": ["Grapl/swarm/SwarmCluster/SwarmASG"]}]
    )
    for reservation in result["Reservations"]:
        for instance in reservation["Instances"]:
            if instance["State"]["Name"] != "terminated":
                yield InstanceTuple(
                    instance_id=instance["InstanceId"],
                    private_ip_address=instance["PrivateIpAddress"],
                    private_dns_name=instance["PrivateDnsName"],
                )


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
        CommandId=command_id, InstanceId=instance_id, PluginName="runShellScript",
    )

    if invocation["Status"] == "Success":
        return invocation["StandardOutputContent"].strip()
    else:
        raise Exception(f"SSM Command failed with Status: \"{invocation['Status']}\"")


CW_NAMESPACE = "CWAgent"
CW_DISK_USAGE_METRIC_NAME = "disk_used_percent"


def _find_operational_alarms_arn(prefix: str, sns: Optional[SNSClient] = None,) -> str:
    sns = sns or boto3.client("sns")
    topics_raw = sns.list_topics()
    all_topic_arns = [d["TopicArn"] for d in topics_raw["Topics"]]

    def seems_like_the_desired_arn(arn: str) -> bool:
        # see CDK class OperationalAlarms
        # note: the prefix should *not* be lower-ized
        return f"{prefix}-operational-alarms-sink" in arn

    arn = next((arn for arn in all_topic_arns if seems_like_the_desired_arn(arn)), None)
    if not arn:
        raise Exception(f"Couldn't find a good candidate arn among {all_topic_arns}")
    return arn


def _find_metric_for_instance(
    cloudwatch: CloudWatchClient, instance_id: str, path: str,
) -> MetricTypeDef:
    """
    To define a Cloudwatch Alarm, one must specify *all* the dimensions complete with values.
    (The dimension names are part of the identity of the metric. Gross!)
    So, before we define an alarm, we do a quick query on path + instance id to get the
    other dimensions.
    """
    metrics_result = cloudwatch.list_metrics(
        Namespace=CW_NAMESPACE,
        MetricName=CW_DISK_USAGE_METRIC_NAME,
        Dimensions=[
            {"Name": "path", "Value": path,},
            {"Name": "InstanceId", "Value": instance_id,},
            {"Name": "AutoScalingGroupName",},
            {"Name": "ImageId",},
            {"Name": "InstanceType",},
            {"Name": "device",},
            {"Name": "fstype"},
        ],
    )
    metrics = metrics_result["Metrics"]
    if len(metrics) != 1:
        raise Exception(
            f"Tried querying for disk metrics in path {path} on {instance_id} - expected 1, got {metrics}\n"
            "(Try waiting ~5m after provisioning an Autoscaling Group for the expected metric to show up.)"
        )
    return metrics[0]


def _create_disk_usage_alarms(
    cloudwatch: CloudWatchClient, instance_id: str, prefix: str,
) -> None:
    ops_alarm_action = _find_operational_alarms_arn(prefix)

    root_metric = _find_metric_for_instance(cloudwatch, instance_id, path="/")
    """Create disk usage alarms for the / and /dgraph partitions"""
    cloudwatch.put_metric_alarm(
        AlarmActions=[ops_alarm_action],
        AlarmName=f"/ disk_used_percent ({instance_id})",
        AlarmDescription=f"Root volume disk usage percent threshold exceeded on {instance_id}",
        ActionsEnabled=False,
        MetricName=root_metric["MetricName"],
        Namespace=root_metric["Namespace"],
        Statistic="Maximum",
        Period=300,
        EvaluationPeriods=1,
        ComparisonOperator="GreaterThanOrEqualToThreshold",
        Threshold=95.0,
        Unit="Percent",
        Dimensions=root_metric["Dimensions"],
    )

    dgraph_partition_metric = _find_metric_for_instance(
        cloudwatch, instance_id, path="/dgraph"
    )
    cloudwatch.put_metric_alarm(
        AlarmActions=[ops_alarm_action],
        AlarmName=f"/dgraph disk_used_percent ({instance_id})",
        AlarmDescription=f"DGraph volume disk usage percent threshold exceeded on {instance_id}",
        ActionsEnabled=False,
        MetricName=dgraph_partition_metric["MetricName"],
        Namespace=dgraph_partition_metric["Namespace"],
        Statistic="Maximum",
        Period=300,
        EvaluationPeriods=1,
        ComparisonOperator="GreaterThanOrEqualToThreshold",
        Threshold=95.0,
        Unit="Percent",
        Dimensions=dgraph_partition_metric["Dimensions"],
    )


def _init_docker_swarm(
    ec2: Any,
    ssm: Any,
    cloudwatch: CloudWatchClient,
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
    LOGGER.info(
        f"Instance {manager_id} with IP {manager_ip} and hostname {manager_hostname} is the docker swarm cluster manager"
    )
    return result


def _join_worker_nodes(
    ec2: Any,
    ssm: Any,
    cloudwatch: CloudWatchClient,
    prefix: str,
    instances: Sequence[InstanceTuple],
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
        LOGGER.info(
            f"Joined worker instance {instance_id} with hostname {hostname} to the docker swarm cluster"
        )
        hostnames.append(hostname)
    return hostnames


def main(prefix: str) -> None:
    ec2 = boto3.client("ec2")

    LOGGER.info("Retrieving instance IDs")
    instances = list(_swarm_instances(ec2))
    manager_instance = instances[0]
    worker_instances = instances[1:]

    ssm = boto3.client("ssm")
    cloudwatch: CloudWatchClient = boto3.client("cloudwatch")

    LOGGER.info("Creating disk usage alarms")
    for instance_id, _, _ in instances:
        _create_disk_usage_alarms(cloudwatch, instance_id, prefix)

    LOGGER.info("Initializing swarm manager")
    manager_id, manager_ip, manager_hostname = manager_instance
    join_token = _init_docker_swarm(
        ec2, ssm, cloudwatch, prefix, manager_id, manager_ip, manager_hostname
    )

    LOGGER.info("Joining worker instances")
    worker_hostnames = _join_worker_nodes(
        ec2, ssm, cloudwatch, prefix, worker_instances, join_token, manager_ip
    )

    LOGGER.info("Docker swarm cluster setup complete")

    print(
        f"""
##
# Paste the following into an SSM shell on the swarm
# manager ({manager_id}) to deploy dgraph to
# the swarm cluster.
#
# With awscli:
# aws ssm start-session --target {manager_id}
##

sudo su ec2-user
cd $HOME

# Write the following exports to a file so they can be re-sourced in future
cat >all_exports <<DELIM
export GRAPL_DEPLOY_NAME="{prefix}"
export AWS_LOGS_GROUP="{prefix}-grapl-dgraph"
export AWS01_NAME="{manager_hostname}"
export AWS02_NAME="{worker_hostnames[0]}"
export AWS03_NAME="{worker_hostnames[1]}"
DELIM

source ./all_exports

aws s3 cp s3://${{GRAPL_DEPLOY_NAME,,}}-dgraph-config-bucket/docker-compose-dgraph.yml .
aws s3 cp s3://${{GRAPL_DEPLOY_NAME,,}}-dgraph-config-bucket/envoy.yaml .

docker stack deploy -c docker-compose-dgraph.yml dgraph

docker service ls

"""
    )


if __name__ == "__main__":
    main(prefix=sys.argv[1])
