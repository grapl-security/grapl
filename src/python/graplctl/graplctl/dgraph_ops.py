from __future__ import annotations

import json
import logging
import os
import sys

from typing import Iterator, List, Tuple, Optional

from botocore.client import ClientError

from mypy_boto3_cloudwatch.client import CloudWatchClient
from mypy_boto3_cloudwatch.type_defs import MetricTypeDef
from mypy_boto3_sns.client import SNSClient
from mypy_boto3_ssm import SSMClient
from mypy_boto3_route53 import Route53Client

from . import common

get_command_results = common.get_command_results
Tag = common.Tag
Ec2Instance = common.Ec2Instance

LOGGER = logging.getLogger(__name__)
LOGGER.setLevel(os.getenv("GRAPL_LOG_LEVEL", "INFO"))
LOGGER.addHandler(logging.StreamHandler(stream=sys.stdout))

CW_NAMESPACE = "CWAgent"
CW_DISK_USAGE_METRIC_NAME = "disk_used_percent"


def _seems_like_the_desired_arn(deployment_name: str, arn: str) -> bool:
    # see CDK class OperationalAlarms
    # note: the deployment_name should *not* be lower-ized
    return f"{deployment_name}-operational-alarms-sink" in arn


def _find_operational_alarms_arn(sns: SNSClient, deployment_name: str) -> str:
    topics_raw = sns.list_topics()
    all_topic_arns = [d["TopicArn"] for d in topics_raw["Topics"]]
    arn = next(
        (
            arn
            for arn in all_topic_arns
            if _seems_like_the_desired_arn(deployment_name, arn)
        ),
        None,
    )
    if not arn:
        raise Exception(f"Couldn't find a good candidate arn among {all_topic_arns}")
    return arn


def _find_metric_for_instance(
    cloudwatch: CloudWatchClient,
    instance_id: str,
    path: str,
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
            {
                "Name": "path",
                "Value": path,
            },
            {
                "Name": "InstanceId",
                "Value": instance_id,
            },
            {
                "Name": "AutoScalingGroupName",
            },
            {
                "Name": "ImageId",
            },
            {
                "Name": "InstanceType",
            },
            {
                "Name": "device",
            },
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


def create_disk_usage_alarms(
    cloudwatch: CloudWatchClient,
    sns: SNSClient,
    instance_id: str,
    deployment_name: str,
) -> None:
    """Create disk usage alarms for the / and /dgraph partitions"""
    ops_alarm_action = _find_operational_alarms_arn(sns, deployment_name)
    root_metric = _find_metric_for_instance(cloudwatch, instance_id, path="/")
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


def _dns_ip_addresses(
    route53: Route53Client,
    dns_name: str,
    ip_address: Optional[str],
    hosted_zone_id: str,
) -> Iterator[str]:
    for rrset in route53.list_resource_record_sets(
        HostedZoneId=hosted_zone_id,
        StartRecordName=dns_name,
    )["ResourceRecordSets"]:
        if rrset["Type"] == "A":
            for rrecord in rrset["ResourceRecords"]:
                yield rrecord["Value"]
    if ip_address is not None:
        yield ip_address


def remove_dns_ip(
    route53: Route53Client,
    dns_name: str,
    ip_address: str,
    hosted_zone_id: str,
) -> None:
    ip_addresses = [
        ip
        for ip in _dns_ip_addresses(route53, dns_name, None, hosted_zone_id)
        if ip != ip_address
    ]

    change = {
        "Action": "DELETE",  # delete the A record if this is the last address
        "ResourceRecordSet": {
            "Name": dns_name,
            "Type": "A",
            "TTL": 300,
            "ResourceRecords": [{"Value": ip_address}],
        },
    }
    if len(ip_addresses) > 0:
        change["Action"] = "UPSERT"
        change["ResourceRecordSet"]["ResourceRecords"] = [
            {"Value": ip} for ip in ip_addresses
        ]

    try:
        comment = f"Removed {ip_address} from {dns_name} DNS A Record"
        route53.change_resource_record_sets(
            HostedZoneId=hosted_zone_id,
            ChangeBatch={
                "Changes": [change],
                "Comment": comment,
            },
        )
        LOGGER.info(comment)
    except ClientError as e:
        if e.response["Error"]["Code"] == "InvalidChangeBatch":
            LOGGER.warn(f"DNS record does not exist for {ip_address}")
        else:
            raise e


def insert_dns_ip(
    route53: Route53Client,
    dns_name: str,
    ip_address: str,
    hosted_zone_id: str,
) -> None:
    comment = f"Inserted {ip_address} into {dns_name} DNS A Record"
    route53.change_resource_record_sets(
        HostedZoneId=hosted_zone_id,
        ChangeBatch={
            "Changes": [
                {
                    "Action": "UPSERT",
                    "ResourceRecordSet": {
                        "Name": dns_name,
                        "Type": "A",
                        "TTL": 300,
                        "ResourceRecords": [
                            {"Value": ip}
                            for ip in _dns_ip_addresses(
                                route53, dns_name, ip_address, hosted_zone_id
                            )
                        ],
                    },
                },
            ],
            "Comment": comment,
        },
    )
    LOGGER.info(comment)


def init_dgraph(
    ssm: SSMClient,
    deployment_name: str,
    instances: List[Ec2Instance],
) -> None:
    """configure the docker swarm cluster instances for dgraph"""
    instance_ids = [instance.instance_id for instance in instances]
    command = ssm.send_command(
        InstanceIds=instance_ids,
        DocumentName="AWS-RunRemoteScript",
        Parameters={
            "sourceType": ["S3"],
            "sourceInfo": [
                json.dumps(
                    {
                        "path": f"https://s3.amazonaws.com/{deployment_name.lower()}-dgraph-config-bucket/dgraph_init.py"
                    }
                )
            ],
            "commandLine": ["/usr/bin/python3 dgraph_init.py"],
        },
    )
    command_id = command["Command"]["CommandId"]
    for instance_id, result in get_command_results(ssm, command_id, instance_ids):
        LOGGER.info(f"command {command_id} instance {instance_id}: {result}")


def deploy_dgraph(
    ssm: SSMClient,
    deployment_name: str,
    manager_instance: Ec2Instance,
    worker_instances: Tuple[Ec2Instance, Ec2Instance],
) -> None:
    """deploy dgraph on the docker swarm cluster"""
    command = ssm.send_command(
        InstanceIds=[manager_instance.instance_id],
        DocumentName="AWS-RunRemoteScript",
        Parameters={
            "sourceType": ["S3"],
            "sourceInfo": [
                json.dumps(
                    {
                        "path": f"https://s3.amazonaws.com/{deployment_name.lower()}-dgraph-config-bucket/dgraph_deploy.py"
                    }
                )
            ],
            "commandLine": [
                f"/usr/bin/python3 dgraph_deploy.py {deployment_name.lower()} {manager_instance.private_dns_name} {worker_instances[0].private_dns_name} {worker_instances[1].private_dns_name}"
            ],
        },
    )
    command_id = command["Command"]["CommandId"]
    instance_id, result = next(
        get_command_results(ssm, command_id, [manager_instance.instance_id])
    )
    LOGGER.info(f"command {command_id} instance {instance_id}: {result}")
